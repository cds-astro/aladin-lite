// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU General Public License version 3.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, version 3 of the License.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    The GNU General Public License is available in COPYING file
//    along with Aladin Lite.
//



/******************************************************************************
 * Aladin Lite project
 * 
 * File HpxImageSurvey
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HpxImageSurvey = (function() {


    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    var HpxImageSurvey = function(id, name, rootUrl, cooFrame, maxOrder, options) {
        this.id = id;
    	this.name = name;
    	if (rootUrl.slice(-1 )=== '/') {
    	    this.rootUrl = rootUrl.substr(0, rootUrl.length-1);
    	}
    	else {
    	    this.rootUrl = rootUrl;
    	}
    	
    	options = options || {};
    	// TODO : support PNG
    	this.imgFormat = options.imgFormat || 'jpg';

        // permet de forcer l'affichage d'un certain niveau
        this.minOrder = options.minOrder || null;

        // TODO : lire depuis fichier properties
        this.cooFrame = CooFrameEnum.fromString(cooFrame, CooFrameEnum.J2000);
        
        // force coo frame for Glimpse 360
        if (this.rootUrl.indexOf('/glimpse360/aladin/data')>=0) {
            this.cooFrame = CooFrameEnum.J2000;
        }
        
        // TODO : lire depuis fichier properties
        this.maxOrder = maxOrder;
    	
    	this.allskyTexture = null;
    	
    	this.alpha = 0.0; // opacity value between 0 and 1 (if this layer is an opacity layer)
    
    	this.allskyTextureSize = 0;
    
        this.lastUpdateDateNeededTiles = 0;

        var found = false;
        for (var k=0; k<HpxImageSurvey.SURVEYS.length; k++) {
            if (HpxImageSurvey.SURVEYS[k].id==this.id) {
                found = true;
            }
        }
        if (! found) {
            HpxImageSurvey.SURVEYS.push({
                 "id": this.id,
                 "url": this.rootUrl,
                 "name": this.name,
                 "maxOrder": this.maxOrder,
                 "frame": this.cooFrame
            });
        }
        HpxImageSurvey.SURVEYS_OBJECTS[this.id] = this;
    };
    
    HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY = 1000; // in milliseconds
    
    HpxImageSurvey.prototype.init = function(view, callback) {
    	this.view = view;
    	
        if (!this.cm) {
            this.cm = new ColorMap(this.view);
        }
    	
    	//this.tileBuffer = new TileBuffer();
    	// tileBuffer is now shared across different image surveys
    	this.tileBuffer = this.view.tileBuffer;
    	
    	this.useCors = false;
    	var self = this;
        if ($.support.cors) {
            // testing if server supports CORS ( http://www.html5rocks.com/en/tutorials/cors/ )
            $.ajax({
                type: 'GET',
                url: this.rootUrl + '/properties',
                contentType: 'text/plain',
                xhrFields: {
                },
                headers: {
                },
                success: function() {
                    // CORS is supported
                    self.useCors = true;
                    
                    self.retrieveAllskyTextures();
                    if (callback) {
                        callback();
                    }
                },
                error: function() {
                    // CORS is not supported
                    self.retrieveAllskyTextures();
                    if (callback) {
                        callback();
                    }
                }
              });
        }
        else {
            this.retrieveAllskyTextures();
            callback();
        }
    	
    };
    
    HpxImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";
    
    HpxImageSurvey.SURVEYS_OBJECTS = {};
    HpxImageSurvey.SURVEYS = [
     {
        "id": "P/2MASS/color",
        "url": "http://alasky.u-strasbg.fr/2MASS/Color",
        "name": "2MASS colored",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/DSS2/color",
        "url": "http://alasky.u-strasbg.fr/DSS/DSSColor",
        "name": "DSS colored",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/DSS2/red",
        "url": "http://alasky.u-strasbg.fr/DSS/DSS2Merged",
        "name": "DSS2 Red (F+R)",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg fits"
     },
     {
        "id": "P/Fermi/color",
        "url": "http://alasky.u-strasbg.fr/Fermi/Color",
        "name": "Fermi color",
        "maxOrder": 3,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/Finkbeiner",
        "url": "http://alasky.u-strasbg.fr/FinkbeinerHalpha",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "jpeg fits",
        "name": "Halpha"
     },
     {
        "id": "P/GALEXGR6/AIS/color",
        "url": "http://alasky.u-strasbg.fr/GALEX/GR6-02-Color",
        "name": "GALEX Allsky Imaging Survey colored",
        "maxOrder": 8,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/IRIS/color",
        "url": "http://alasky.u-strasbg.fr/IRISColor",
        "name": "IRIS colored",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/Mellinger/color",
        "url": "http://alasky.u-strasbg.fr/MellingerRGB",
        "name": "Mellinger colored",
        "maxOrder": 4,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/SDSS9/color",
        "url": "http://alasky.u-strasbg.fr/SDSS/DR9/color",
        "name": "SDSS9 colored",
        "maxOrder": 10,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/SPITZER/color",
        "url": "http://alasky.u-strasbg.fr/SpitzerI1I2I4color",
        "name": "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
        "maxOrder": 9,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/VTSS/Ha",
        "url": "http://alasky.u-strasbg.fr/VTSS/Ha",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "png jpeg fits",
        "name": "VTSS-Ha"
     },
     {
        "id": "P/XMM/EPIC",
        "url": "http://saada.u-strasbg.fr/xmmallsky",
        "name": "XMM-Newton stacked EPIC images (no phot. normalization)",
        "maxOrder": 7,
        "frame": "equatorial",
        "format": "png jpeg fits"
     },
     {
         "id": "P/XMM/PN/color",
          "url": "http://saada.unistra.fr/xmmpnsky",
          "name": "XMM PN colored",
          "maxOrder": 7,
          "frame": "equatorial",
          "format": "png jpeg"
     },
     {
         "id": "P/allWISE/color",
         "url": "http://alasky.u-strasbg.fr/AllWISE/RGB-W4-W2-W1/",
         "name": "AllWISE color",
         "maxOrder": 8,
         "frame": "equatorial",
         "format": "jpeg"
     },
     {
         "id": "P/GLIMPSE360",
         "url": "http://www.spitzer.caltech.edu/glimpse360/aladin/data",
         "name": "GLIMPSE360",
         "maxOrder": 9,
         "frame": "equatorial",
         "format": "jpeg"
     }
  ];


    
    HpxImageSurvey.getAvailableSurveys = function() {
    	return HpxImageSurvey.SURVEYS;
    };
    
    HpxImageSurvey.getSurveyInfoFromId = function(id) {
        var surveys = HpxImageSurvey.getAvailableSurveys();
        for (var i=0; i<surveys.length; i++) {
            if (surveys[i].id==id) {
                return surveys[i];
            }
        }
        return null;
    };

    HpxImageSurvey.getSurveyFromId = function(id) {
        if (HpxImageSurvey.SURVEYS_OBJECTS[id]) {
            return HpxImageSurvey.SURVEYS_OBJECTS[id];
        }
        var surveyInfo = HpxImageSurvey.getSurveyInfoFromId(id);
        if (surveyInfo) {
            return new HpxImageSurvey(surveyInfo.id, surveyInfo.name, surveyInfo.url, surveyInfo.frame, surveyInfo.maxOrder);
        }

        return null;
    }
   
/* 
    HpxImageSurvey.getSurveyFromId = function(id) {
    	var info = HpxImageSurvey.getSurveyInfoFromId(id);
    	if (info) {
    		return new HpxImageSurvey(info.id, info.name, info.url, info.frame, info.maxOrder);
    	}
    	
    	return null;
    };
*/
    
    HpxImageSurvey.prototype.getTileURL = function(norder, npix) {
    	var dirIdx = Math.floor(npix/10000)*10000;
    	return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + "." + this.imgFormat;
    };
    
    HpxImageSurvey.prototype.retrieveAllskyTextures = function() {
    	// start loading of allsky
    	var img = new Image();
    	if (this.useCors) {
            img.crossOrigin = 'anonymous';
        }
    	var self = this;
    	img.onload = function() {
    		// sur ipad, le fichier qu'on récupère est 2 fois plus petit. Il faut donc déterminer la taille de la texture dynamiquement
    	    self.allskyTextureSize = img.width/27;
            self.allskyTexture = img;
   
            /* 
    		// récupération des 768 textures (NSIDE=4)
    		for (var j=0; j<29; j++) {
    			for (var i=0; i<27; i++) {
    				var c = document.createElement('canvas');
    				c.width = c.height = self.allskyTextureSize;
    				c.allSkyTexture = true;
    				var context = c.getContext('2d');
    				context.drawImage(img, i*self.allskyTextureSize, j*self.allskyTextureSize, self.allskyTextureSize, self.allskyTextureSize, 0, 0, c.width, c.height);
    				self.allskyTextures.push(c);
    			}
    		}
            */
    		self.view.requestRedraw();
    	};
    	img.src = this.rootUrl + '/Norder3/Allsky.' + this.imgFormat;
    
    };
    
    HpxImageSurvey.prototype.redrawAllsky = function(ctx, cornersXYViewMap, fov, norder) {
    	// for norder deeper than 6, we think it brings nothing to draw the all-sky
    	if (this.view.curNorder>6) {
    		return;
    	}
    	
    	if ( ! this.allskyTexture ) {
    		return;
    	}
    	

    	var cornersXYView;
        var coeff = 0;
        var center;
        var ipix;
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;


    		
            if ( ! this.allskyTexture || !Tile.isImageOk(this.allskyTexture) ) {
                continue;
            }

            var dy = this.allskyTextureSize * Math.floor(ipix/27);
            var dx = this.allskyTextureSize * (ipix - 27*Math.floor(ipix/27));

    		
    
    		// TODO : plutot agrandir le clip ?
    	    // grow cornersXYView
    	    if (fov>40) {
    			coeff = 0.02;
                coeff = 0.0;
    	        center = {x: (cornersXYView[0].vx+cornersXYView[2].vx)/2, y: (cornersXYView[0].vy+cornersXYView[2].vy)/2};
    	        for (var i=0; i<4; i++) {
    	            var diff = {x: cornersXYView[i].vx-center.x, y: cornersXYView[i].vy-center.y};
    	            cornersXYView[i].vx += coeff*diff.x;
    	            cornersXYView[i].vy += coeff*diff.y;
    	        }
    	    }
    			
    	    this.drawOneTile(ctx, this.allskyTexture, cornersXYView, this.allskyTextureSize, null, dx, dy, true);
    	}
    };
    
    HpxImageSurvey.prototype.getColorMap = function() {
        return this.cm;
    };
    
    var drawEven = true;
    // TODO: avoir un mode où on ne cherche pas à dessiner d'abord les tuiles parentes (pour génération vignettes côté serveur)
    HpxImageSurvey.prototype.redrawHighres = function(ctx, cornersXYViewMap, norder) {
        
        // DOES THAT FIX THE PROBLEM ???
        if (cornersXYViewMap.length==0) {
            return;
        }
        
        drawEven = ! drawEven;
        var now = new Date().getTime();
        var updateNeededTiles = (now-this.lastUpdateDateNeededTiles) > HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY;
        var tile, url, parentTile, parentUrl;
        var parentNorder = norder - 1;
        var cornersXYView, parentCornersXYView;
        var tilesToDraw = [];
        var parentTilesToDraw = [];
        var parentTilesToDrawIpix = {};
        var missingTiles = false;
        
        var tilesToDownload = [];
        var parentTilesToDownload = [];
        
        var parentIpix;
        var ipix;
        
        // tri des tuiles selon la distance
        if (updateNeededTiles) {
            var center = [(cornersXYViewMap[0][0].vx+cornersXYViewMap[0][1].vx)/2, (cornersXYViewMap[0][0].vy+cornersXYViewMap[0][1].vy)/2];
            var newCornersXYViewMap = cornersXYViewMap.sort(function(a, b) {
                var cA = [(a[0].vx+a[2].vx)/2, (a[0].vy+a[2].vy)/2];
                var cB = [(b[0].vx+b[2].vx)/2, (b[0].vy+b[2].vy)/2]; 

                var distA = (cA[0]-center[0])*(cA[0]-center[0]) + (cA[1]-center[1])*(cA[1]-center[1]);
                var distB = (cB[0]-center[0])*(cB[0]-center[0]) + (cB[1]-center[1])*(cB[1]-center[1]);
                
                return distA-distB;
                    
            });
            cornersXYViewMap = newCornersXYViewMap;
        }

        
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;
    		/*
    		if (ipix%2==0 && ! drawEven) {
    		    continue;
    		}
    		else if (ipix%2==1 && drawEven) {
    		    continue;
    		}
    		*/
            
            // on demande à charger le parent (cas d'un zoomOut)
            // TODO : mettre priorité plus basse
            parentIpix = ~~(ipix/4);
        	parentUrl = this.getTileURL(parentNorder, parentIpix);
            if (updateNeededTiles && parentNorder>=3) {
            	parentTile = this.tileBuffer.addTile(parentUrl);
                if (parentTile) {
                    parentTilesToDownload.push({img: parentTile.img, url: parentUrl});
                }
            }
            
            url = this.getTileURL(norder, ipix);
            tile = this.tileBuffer.getTile(url);
            
            if ( ! tile ) {
                missingTiles = true;
                
                if (updateNeededTiles) {
                    var tile = this.tileBuffer.addTile(url);
                    if (tile) {
                        tilesToDownload.push({img: tile.img, url: url});
                    }
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = this.view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
    
                continue;
            }
            else if ( ! Tile.isImageOk(tile.img)) {
                missingTiles = true;
                if (updateNeededTiles && ! tile.img.dlError) {
                    tilesToDownload.push({img: tile.img, url: url});
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = this.view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
                
                continue;
            }
            tilesToDraw.push({img: tile.img, corners: cornersXYView});
        }
    	
    
    
        // draw parent tiles
        for (var k=0, len = parentTilesToDraw.length; k<len; k++) {
        	this.drawOneTile(ctx, parentTilesToDraw[k].img, parentTilesToDraw[k].corners, parentTilesToDraw[k].img.width);
        }
        
        // draw tiles
        /*
        // using loop unrolling
        var iterations = Math.ceil(tilesToDraw.length / 8);
        var startAt = tilesToDraw.length % 8;
        var i = 0;
        var theTileToDraw;
        do {
            switch(startAt){
                case 0: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 7: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 6: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 5: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 4: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 3: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 2: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 1: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
            }
            startAt = 0;
        } while (--iterations > 0);
        */
        
        // draw tiles
        ///*
        for (var k=0, len = tilesToDraw.length; k<len; k++) {
        	var alpha = null;
        	var img = tilesToDraw[k].img;
        	if (img.fadingStart) {
        		if (img.fadingEnd && now<img.fadingEnd) {
        			alpha = 0.2 + (now - img.fadingStart)/(img.fadingEnd - img.fadingStart)*0.8;
        		}
        	}
        	this.drawOneTile(ctx, img, tilesToDraw[k].corners, img.width, alpha);
        }
        //*/
    

        // demande de chargement des tuiles manquantes et mise à jour lastUpdateDateNeededTiles
        if (updateNeededTiles) {
            // demande de chargement des tuiles
            for (var k=0, len = tilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(tilesToDownload[k].img, tilesToDownload[k].url, this.useCors);
            }
            //demande de chargement des tuiles parentes
            for (var k=0, len = parentTilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(parentTilesToDownload[k].img, parentTilesToDownload[k].url, this.useCors);
            }
            this.lastUpdateDateNeededTiles = now;
        }
        if (missingTiles) {
            // callback pour redemander un display dans 1000ms
            this.view.requestRedrawAtDate(now+HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY+10);
        }
    };
    
    function dist2(x1,y1,x2,y2) {
    	return Math.pow(x2-x1, 2) + Math.pow(y2-y1, 2);
    }
    
    HpxImageSurvey.prototype.drawOneTile = function(ctx, img, cornersXYView, textureSize, alpha, dx, dy, applyCorrection) {
        
        // apply CM
        var newImg = this.useCors ? this.cm.apply(img) : img;
        
        
    	// is the tile a diamond ?
    //	var round = AladinUtils.myRound;
    //	var b = cornersXYView;
    //	var flagDiamond =  round(b[0].vx - b[2].vx) == round(b[1].vx - b[3].vx)
    //    				&& round(b[0].vy - b[2].vy) == round(b[1].vy - b[3].vy); 
    	
    	drawTexturedTriangle(ctx, newImg,
                cornersXYView[0].vx, cornersXYView[0].vy,
                cornersXYView[1].vx, cornersXYView[1].vy,
    	        cornersXYView[3].vx, cornersXYView[3].vy,
    	        textureSize-1, textureSize-1,
    	        textureSize-1, 0,
    	        0, textureSize-1,
    	        alpha,
                dx, dy, applyCorrection);
        drawTexturedTriangle(ctx, newImg,
        		cornersXYView[1].vx, cornersXYView[1].vy,
        		cornersXYView[3].vx, cornersXYView[3].vy,
        		cornersXYView[2].vx, cornersXYView[2].vy,
        		textureSize-1, 0,
        		0, textureSize-1,
        		0, 0,
        		alpha,
                dx, dy, applyCorrection);
    };
    
    
    	        
    
    // uses affine texture mapping to draw a textured triangle
    // at screen coordinates [x0, y0], [x1, y1], [x2, y2] from
    // img *pixel* coordinates [u0, v0], [u1, v1], [u2, v2]
    // code from http://www.dhteumeuleu.com/lab/image3D.html
    function drawTexturedTriangle(ctx, img, x0, y0, x1, y1, x2, y2,
                                        u0, v0, u1, v1, u2, v2, alpha,
                                        dx, dy, applyCorrection) {

        dx = dx || 0;
        dy = dy || 0;

        if (!applyCorrection) {
            applyCorrection = false;
        }

        u0 += dx;
        u1 += dx;
        u2 += dx;
        v0 += dy;
        v1 += dy;
        v2 += dy;
        var xc = (x0 + x1 + x2) / 3;
        var yc = (y0 + y1 + y2) / 3;


        // ---- centroid ----
        var xc = (x0 + x1 + x2) / 3;
        var yc = (y0 + y1 + y2) / 3;
        ctx.save();
        if (alpha) {
        	ctx.globalAlpha = alpha;
        }
    
        var coeff = 0.01; // default value
        if (applyCorrection) {
            coeff = 0.01;
        }
        // ---- scale triangle by (1 + coeff) to remove anti-aliasing and draw ----
        ctx.beginPath();
        ctx.moveTo(((1+coeff) * x0 - xc * coeff), ((1+coeff) * y0 - yc * coeff));
        ctx.lineTo(((1+coeff) * x1 - xc * coeff), ((1+coeff) * y1 - yc * coeff));
        ctx.lineTo(((1+coeff) * x2 - xc * coeff), ((1+coeff) * y2 - yc * coeff));
        ctx.closePath();
        ctx.clip();


        // this is needed to prevent to see some lines between triangles
        if (applyCorrection) {
            coeff = 0.03;
            x0 = ((1+coeff) * x0 - xc * coeff), y0 = ((1+coeff) * y0 - yc * coeff);
            x1 = ((1+coeff) * x1 - xc * coeff), y1 = ((1+coeff) * y1 - yc * coeff);
            x2 = ((1+coeff) * x2 - xc * coeff), y2 = ((1+coeff) * y2 - yc * coeff);
        }

        // ---- transform texture ----
        var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
        ctx.transform(
            -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
             (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
             (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
            -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
             (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
             (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
        );
        ctx.drawImage(img, 0, 0);
        //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); 
        
    //    ctx.globalAlpha = 1.0;
    
        ctx.restore();
    }
    
    /*
    function drawTexturedTriangle4Points(ctx, img, x0, y0, x1, y1, x2, y2,
            u0, v0, u1, v1, u2, v2) {
    
    	var x3 = x1+x2-x0;
    	var y3 = y1+y2-y0;
    // ---- centroid ----
    var xc = (x0 + x1 + x2 + x3) / 4;
    var yc = (y0 + y1 + y2 + y3) / 4;
    ctx.save();
    ctx.beginPath();
    // ---- scale triagle by 1.05 to remove anti-aliasing and draw ----
    ctx.moveTo((1.05 * x0 - xc * 0.05), (1.05 * y0 - yc * 0.05));
    ctx.lineTo((1.05 * x1 - xc * 0.05), (1.05 * y1 - yc * 0.05));
    ctx.lineTo((1.05 * x3 - xc * 0.05), (1.05 * y3 - yc * 0.05));
    ctx.lineTo((1.05 * x2 - xc * 0.05), (1.05 * y2 - yc * 0.05));
    ctx.closePath();
    ctx.clip();
    // ---- transform texture ----
    var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
    ctx.transform(
    -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
    (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
    (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
    -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
    (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
    (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
    );
    //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); // faster ??
    ctx.drawImage(img, 0, 0); // slower ??
    
    ctx.restore();
    }
    */
    
    function grow(b, val)  {
    
    	  var b1 = new Array(b.length);
    	  for( var i=0; i<4; i++ ) {
    		  b1[i] = {vx: b[i].vx, vy: b[i].vy};
    	  }
    
    	  for( var i=0; i<2; i++ ) {
    	     var a= i==1 ? 1 : 0;
    	     var c= i==1 ? 2 : 3;
    
    	     var angle = Math.atan2(b1[c].vy-b1[a].vy, b1[c].vx-b1[a].vx);
    	     var chouilla = val*Math.cos(angle);
    	     b1[a].vx -= chouilla;
    	     b1[c].vx += chouilla;
    	     chouilla = val*Math.sin(angle);
    	     b1[a].vy -= chouilla;
    	     b1[c].vy += chouilla;
    	  }
      return b1;
    }
    
    // @api
    HpxImageSurvey.prototype.setAlpha = function(alpha) {
        alpha = +alpha; // coerce to number
        this.alpha = Math.max(0, Math.min(alpha, 1));
        this.view.requestRedraw();
    };
    
    // @api
    HpxImageSurvey.prototype.getAlpha = function() {
        return this.alpha;
    }

    return HpxImageSurvey;
})();
