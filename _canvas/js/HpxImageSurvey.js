/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HpxImageSurvey
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HpxImageSurvey = function(name, rootUrl, cooFrame, maxOrder, tileBuffer) {
	this.name = name;
	this.rootUrl = rootUrl;
    this.cooFrame = cooFrame;
    this.maxOrder = maxOrder;
	
	this.allskyTextures = [];
	

	this.allskyTextureSize = 0;

    this.UPDATE_NEEDED_TILES_DELAY = 1000; // in milliseconds
    this.lastUpdateDateNeededTiles = 0;
    
};

HpxImageSurvey.prototype.init = function(view) {
	this.view = view;
	
	//this.tileBuffer = new TileBuffer();
	// tileBuffer is now shared across different image surveys
	this.tileBuffer = view.tileBuffer;

	this.retrieveAllskyTextures();
};

HpxImageSurvey.getAvailableSurveys = function() {
	return [
			{name: "DSS Red", url: "http://alasky.u-strasbg.fr/DSS/DSS2Merged", frame: CooFrameEnum.J2000, maxOrder: 9},
			{name: "DSS Color", url: "http://alasky.u-strasbg.fr/DssColor", frame: CooFrameEnum.J2000, maxOrder: 9},
			{name: "Mellinger RGB", url: "http://alasky.u-strasbg.fr/MellingerRGB", frame: CooFrameEnum.GAL, maxOrder: 4},
			{name: "SDSS DR7 Color", url: "http://alasky.u-strasbg.fr/SDSS/Color", frame: CooFrameEnum.J2000, maxOrder: 10},
			{name: "GALEX AIS Color", url: "http://alasky.u-strasbg.fr/GALEX/GR6-02-Color", frame: CooFrameEnum.J2000, maxOrder: 8},
			{name: "IRAC Color", url: "http://alasky.u-strasbg.fr/SpitzerI1I2I4color", frame: CooFrameEnum.GAL, maxOrder: 9},
			{name: "IRAS-IRIS Color", url: "http://alasky.u-strasbg.fr/IRISColor", frame: CooFrameEnum.GAL, maxOrder: 3},
			{name: "WISE W1 low-res", url: "http://alasky.u-strasbg.fr/WISE/W1", frame: CooFrameEnum.GAL, maxOrder: 5},
			{name: "2MASS Color", url: "http://alasky.u-strasbg.fr/2MASS/Color", frame: CooFrameEnum.J2000, maxOrder: 9},
			{name: "Halpha composite map", url: "http://alasky.u-strasbg.fr/FinkbeinerHalpha", frame: CooFrameEnum.GAL, maxOrder: 3},
			{name: "Planck HFI color", url: "http://alasky.u-strasbg.fr/PLANCK/HFIColor353-545-857", frame: CooFrameEnum.GAL, maxOrder: 3},
			{name: "XMM-Newton", url: "http://saada.u-strasbg.fr/xmmallsky", frame: CooFrameEnum.GAL, maxOrder: 7},
			{name: "GALEX NUV", url: "http://alasky.u-strasbg.fr/GALEX/GR6-01-ND", frame: CooFrameEnum.J2000, maxOrder: 8}
			//{name: "SDSS DR9", url: "http://alaskybis.u-strasbg.fr/SDSS-DR9", frame: CooFrameEnum.J2000, maxOrder: 10}

			
	];
};

HpxImageSurvey.getSurveyInfoFromName = function(name) {
    var surveys = HpxImageSurvey.getAvailableSurveys();
    for (var i=0; i<surveys.length; i++) {
        if (surveys[i].name==name) {
            return surveys[i];
        }
    }
    return null;
};

HpxImageSurvey.getSurveyFromName = function(name) {
	var info = HpxImageSurvey.getSurveyInfoFromName(name);
	if (info) {
		return new HpxImageSurvey(info.name, info.url, info.frame, info.maxOrder);
	}
	
	return null;
};

HpxImageSurvey.prototype.getTileURL = function(norder, npix) {
	var dirIdx = Math.floor(npix/10000)*10000;
	return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + ".jpg";
};

HpxImageSurvey.prototype.retrieveAllskyTextures = function() {
	// start loading of allsky
	var img = new Image();
	var survey = this;
	img.onload = function() {
		// sur ipad, le fichier qu'on récupère est 2 fois plus petit. Il faut donc déterminer la taille de la texture dynamiquement
		survey.allskyTextureSize = img.width/27;

		// récupération des 768 textures (NSIDE=4)
		for (var j=0; j<29; j++) {
			for (var i=0; i<27; i++) {
				var c = document.createElement('canvas');
				c.width = c.height = survey.allskyTextureSize;
				c.allSkyTexture = true;
				var context = c.getContext('2d');
				context.drawImage(img, i*survey.allskyTextureSize, j*survey.allskyTextureSize, survey.allskyTextureSize, survey.allskyTextureSize, 0, 0, c.width, c.height);
				survey.allskyTextures.push(c);
			}
		}
		survey.view.requestRedraw();
	};
	img.src = this.rootUrl + '/Norder3/Allsky.jpg';

};

HpxImageSurvey.prototype.redrawAllsky = function(ctx, cornersXYViewMap, fov, norder) {
	// for norder deeper than 6, we think it brings nothing to draw the all-sky
	if (view.curNorder>6) {
		return;
	}
	
	if ( ! this.allskyTextures ) {
		return;
	}
	
    var cornersXYView;
    var coeff = 0;
    var center;
    var ipix;
	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
		cornersXYView = cornersXYViewMap[k];
		ipix = cornersXYView.ipix;
		
        if ( ! this.allskyTextures[ipix] || !Tile.isImageOk(this.allskyTextures[ipix]) ) {
            continue;
        }
		

		// TODO : plutot agrandir le clip ?
	    // grow cornersXYView
	    if (fov>40) {
			coeff = 0.02;
	        center = {x: (cornersXYView[0].vx+cornersXYView[2].vx)/2, y: (cornersXYView[0].vy+cornersXYView[2].vy)/2};
	        for (var i=0; i<4; i++) {
	            var diff = {x: cornersXYView[i].vx-center.x, y: cornersXYView[i].vy-center.y};
	            cornersXYView[i].vx += coeff*diff.x;
	            cornersXYView[i].vy += coeff*diff.y;
	        }
	    }
			
	    this.drawOneTile(ctx, this.allskyTextures[ipix], cornersXYView, this.allskyTextureSize);
	}
};

// TODO: avoir un mode où on ne cherche pas à dessiner d'abord les tuiles parentes (pour génération vignettes côté serveur)
HpxImageSurvey.prototype.redrawHighres = function(ctx, cornersXYViewMap, norder) {
    var now = new Date().getTime();
    var updateNeededTiles = (now-this.lastUpdateDateNeededTiles) > this.UPDATE_NEEDED_TILES_DELAY;
    var tile, url, parentTile, parentUrl;
    var parentNorder = norder - 1;
    var cornersXYView, parentCornersXYView;
    var tilesToDraw = [];
    var parentTilesToDraw = [];
    var parentTilesToDrawIpix = {};
    
    var parentIpix;
    var ipix;
	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
		cornersXYView = cornersXYViewMap[k];
		ipix = cornersXYView.ipix;
        
        // on demande à charger le parent (cas d'un zoomOut)
        // TODO : mettre priorité plus basse
        parentIpix = ~~(ipix/4);
    	parentUrl = this.getTileURL(parentNorder, parentIpix);
        if (updateNeededTiles && parentNorder>=3) {
        	parentTile = this.tileBuffer.addTile(parentUrl);
            if (parentTile) {
                this.view.downloader.requestDownload(parentTile.img, parentUrl);
            }
        }
        
        url = this.getTileURL(norder, ipix);
        tile = this.tileBuffer.getTile(url);
        
        if ( ! tile ) {
            
            if (updateNeededTiles) {
                var tile = this.tileBuffer.addTile(url);
                if (tile) {
                    this.view.downloader.requestDownload(tile.img, url);
                }
            }
            
            // is the parent tile available ?
            if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
            	parentTile = this.tileBuffer.getTile(parentUrl);
            	if (parentTile && Tile.isImageOk(parentTile.img)) {
            		parentCornersXYView = view.getPositionsInView(parentIpix, parentNorder);
            		if (parentCornersXYView) {
            			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
            		}
            	}
            	parentTilesToDrawIpix[parentIpix] = 1;
            }

            continue;
        }
        else if ( ! Tile.isImageOk(tile.img)) {
            if (updateNeededTiles) {
                this.view.downloader.requestDownload(tile.img, url);
            }
            
            // is the parent tile available ?
            if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
            	parentTile = this.tileBuffer.getTile(parentUrl);
            	if (parentTile && Tile.isImageOk(parentTile.img)) {
            		parentCornersXYView = view.getPositionsInView(parentIpix, parentNorder);
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
    for (var k=0, len = tilesToDraw.length; k<len; k++) {
    	var alpha = null;
    	var img = tilesToDraw[k].img;
    	if (img.fadingStart) {
    		if (img.fadingEnd && now<img.fadingEnd) {
    			alpha = 0.2 + (now - img.fadingStart)/(img.fadingEnd - img.fadingStart)*0.8;
    		}
    	}
    	this.drawOneTile(ctx, tilesToDraw[k].img, tilesToDraw[k].corners, tilesToDraw[k].img.width, alpha);
    }


    // mise à jour lastUpdateDateNeededTiles
    if (updateNeededTiles) {
        lastUpdateDateNeededTiles = now;
    }
};

function dist2(x1,y1,x2,y2) {
	return Math.pow(x2-x1, 2) + Math.pow(y2-y1, 2);
}

HpxImageSurvey.prototype.drawOneTile = function(ctx, img, cornersXYView, textureSize, alpha) {
	// is the tile a diamond ?
//	var round = AladinUtils.myRound;
//	var b = cornersXYView;
//	var flagDiamond =  round(b[0].vx - b[2].vx) == round(b[1].vx - b[3].vx)
//    				&& round(b[0].vy - b[2].vy) == round(b[1].vy - b[3].vy); 
	
	                  
	
	
	drawTexturedTriangle(ctx, img,
            cornersXYView[0].vx, cornersXYView[0].vy,
            cornersXYView[1].vx, cornersXYView[1].vy,
	        cornersXYView[3].vx, cornersXYView[3].vy,
	        textureSize-1, textureSize-1,
	        textureSize-1, 0,
	        0, textureSize-1,
	        alpha);
    drawTexturedTriangle(ctx, img,
    		cornersXYView[1].vx, cornersXYView[1].vy,
    		cornersXYView[3].vx, cornersXYView[3].vy,
    		cornersXYView[2].vx, cornersXYView[2].vy,
    		textureSize-1, 0,
    		0, textureSize-1,
    		0, 0,
    		alpha);
};


	        

// uses affine texture mapping to draw a textured triangle
// at screen coordinates [x0, y0], [x1, y1], [x2, y2] from
// img *pixel* coordinates [u0, v0], [u1, v1], [u2, v2]
// code from http://www.dhteumeuleu.com/lab/image3D.html
function drawTexturedTriangle(ctx, img, x0, y0, x1, y1, x2, y2,
                                    u0, v0, u1, v1, u2, v2, alpha) {
	
    // ---- centroid ----
    var xc = (x0 + x1 + x2) / 3;
    var yc = (y0 + y1 + y2) / 3;
    ctx.save();
    if (alpha) {
    	ctx.globalAlpha = alpha;
    }

    ctx.beginPath();
    var coeff = 0.05;
    // ---- scale triangle by 1.05 to remove anti-aliasing and draw ----
    ctx.moveTo(((1+coeff) * x0 - xc * coeff), ((1+coeff) * y0 - yc * coeff));
    ctx.lineTo(((1+coeff) * x1 - xc * coeff), ((1+coeff) * y1 - yc * coeff));
    ctx.lineTo(((1+coeff) * x2 - xc * coeff), ((1+coeff) * y2 - yc * coeff));
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
