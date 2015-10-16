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
 * File View.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

View = (function() {

    /** Constructor */
    function View (aladin, location, fovDiv, cooFrame, zoom) {
            this.aladin = aladin;
            this.options = aladin.options;
    		this.aladinDiv = this.aladin.aladinDiv;
            this.popup = new Popup(this.aladinDiv);

    		this.createCanvases();
    		this.location = location;
    		this.fovDiv = fovDiv;
    		this.mustClearCatalog = true;
    		this.mustRedrawReticle = true;
    		
    		this.mode = View.PAN;
    		
    		this.minFOV = this.maxFOV = null; // by default, no restriction
    		
    		this.healpixGrid = new HealpixGrid(this.imageCanvas);
    		if (cooFrame) {
                this.cooFrame = cooFrame;
            }
            else {
                this.cooFrame = CooFrameEnum.GAL;
            }
    		
    		var lon, lat;
    		lon = lat = 0;
    		
    		this.projectionMethod = ProjectionEnum.SIN;
    		this.projection = new Projection(lon, lat);
    		this.projection.setProjection(this.projectionMethod);
            this.zoomLevel = 0;
            this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
    
    		this.viewCenter = {lon: lon, lat: lat}; // position of center of view
    		
    		if (zoom) {
                this.setZoom(zoom);
            }
    		
    		// current image survey displayed
    		this.imageSurvey = null;
    		// current catalog displayed
    		this.catalogs = [];
            // overlays (footprints for instance)
    		this.overlays = [];
            // MOCs
    		this.mocs = [];
    		
    
    		
    		this.tileBuffer = new TileBuffer(); // tile buffer is shared across different image surveys
    		this.fixLayoutDimensions();
            
    
    		this.curNorder = 1;
    		this.realNorder = 1;
            this.curOverlayNorder = 1;
    		
    		// some variables for mouse handling
    		this.dragging = false;
    		this.dragx = null;
    		this.dragy = null;
    		this.needRedraw = true;
    
            this.downloader = new Downloader(this); // the downloader object is shared across all HpxImageSurveys
            this.flagForceRedraw = false;
    
            this.fadingLatestUpdate = null;
    		
            this.dateRequestRedraw = null;
            
            this.showGrid = false; // coordinates grid
            
    		init(this);
    		

    		// listen to window resize and reshape canvases
    		this.resizeTimer = null;
    		var self = this;
    		$(window).resize(function() {
    		    clearTimeout(self.resizeTimer);
    		    self.resizeTimer = setTimeout(function() {self.fixLayoutDimensions(self)}, 100);
    		});
    	};
	
    // différents modes
    View.PAN = 0;
    View.SELECT = 1;
    	
    
    // TODO: should be put as an option at layer level	
	View.DRAW_SOURCES_WHILE_DRAGGING = true;
	View.DRAW_MOCS_WHILE_DRAGGING = true;
	
	
	// (re)create needed canvases
	View.prototype.createCanvases = function() {
	    var a = $(this.aladinDiv);
	    a.find('.aladin-imageCanvas').remove();
	    a.find('.aladin-catalogCanvas').remove();
	    a.find('.aladin-reticleCanvas').remove();
        
        // canvas to draw the images
        this.imageCanvas = $("<canvas class='aladin-imageCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the catalogs
        this.catalogCanvas = $("<canvas class='aladin-catalogCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the reticle
        this.reticleCanvas = $("<canvas class='aladin-reticleCanvas'></canvas>").appendTo(this.aladinDiv)[0];
	};
	
	
	// called at startup and when window is resized
	View.prototype.fixLayoutDimensions = function() {
        Utils.cssScale = undefined;
		
        this.width = $(this.aladinDiv).width();
		this.height = $(this.aladinDiv).height();
		
		this.width = Math.max(this.width, 1);
		this.height = Math.max(this.height, 1); // this prevents many problems when div size is 0s
        
		
		this.cx = this.width/2;
		this.cy = this.height/2;
		
		this.largestDim = Math.max(this.width, this.height);
		this.smallestDim = Math.min(this.width, this.height);
		this.ratio = this.largestDim/this.smallestDim;

		
		this.mouseMoveIncrement = 160/this.largestDim;

		
		// reinitialize 2D context
		this.imageCtx = this.imageCanvas.getContext("2d");
		this.catalogCtx = this.catalogCanvas.getContext("2d");
		this.reticleCtx = this.reticleCanvas.getContext("2d");
		
		this.imageCtx.canvas.width = this.width;
		this.catalogCtx.canvas.width = this.width;
        this.reticleCtx.canvas.width = this.width;

		
		this.imageCtx.canvas.height = this.height;
		this.catalogCtx.canvas.height = this.height;
        this.reticleCtx.canvas.height = this.height;
        
        this.computeNorder();
        this.requestRedraw();
	};
    

	View.prototype.setMode = function(mode) {
	    this.mode = mode;
	    if (this.mode==View.SELECT) {
	        this.setCursor('crosshair');
	    }
	    else {
	        this.setCursor('default');
	    }
	};
	
	View.prototype.setCursor = function(cursor) {
        if (this.reticleCanvas.style.cursor==cursor) {
            return;
        }
	    this.reticleCanvas.style.cursor = cursor;
	};

	
	
	/**
	 * return dataURL string corresponding to the current view
	 */
	View.prototype.getCanvasDataURL = function(imgType) {
        imgType = imgType || "image/png"; 
	    var c = document.createElement('canvas');
        c.width = this.width;
        c.height = this.height;
        var ctx = c.getContext('2d');
        ctx.drawImage(this.imageCanvas, 0, 0);
        ctx.drawImage(this.catalogCanvas, 0, 0);
        ctx.drawImage(this.reticleCanvas, 0, 0);
        
	    return c.toDataURL(imgType);
	    //return c.toDataURL("image/jpeg", 0.01); // setting quality only works for JPEG (?)
	};


	/**
	 * Compute the FoV in degrees of the view and update mouseMoveIncrement
	 * 
	 * @param view
	 * @returns FoV (array of 2 elements : width and height) in degrees
	 */
	computeFov = function(view) {
		var fov = doComputeFov(view, view.zoomFactor);
		
		
		view.mouseMoveIncrement = fov/view.imageCanvas.width;
			
		return fov;
	};
	
	doComputeFov = function(view, zoomFactor) {
	 // if zoom factor < 1, we view 180°
        if (view.zoomFactor<1) {
            fov = 180;
        }
        else {
            // TODO : fov sur les 2 dimensions !!
            // to compute FoV, we first retrieve 2 points at coordinates (0, view.cy) and (width-1, view.cy)
            var xy1 = AladinUtils.viewToXy(0, view.cy, view.width, view.height, view.largestDim, zoomFactor);
            var lonlat1 = view.projection.unproject(xy1.x, xy1.y);
            
            var xy2 = AladinUtils.viewToXy(view.imageCanvas.width-1, view.cy, view.width, view.height, view.largestDim, zoomFactor);
            var lonlat2 = view.projection.unproject(xy2.x, xy2.y);
            
            
            fov = new Coo(lonlat1.ra, lonlat1.dec).distance(new Coo(lonlat2.ra, lonlat2.dec));
        }
        
        return fov;
	};
	
	updateFovDiv = function(view) {
	    if (isNaN(view.fov)) {
	        view.fovDiv.html("FoV:");
	        return;
	    }
        // màj valeur FoV
        var fovStr;
        if (view.fov>1) {
            fovStr = Math.round(view.fov*100)/100 + "°";
        }
        else if (view.fov*60>1) {
            fovStr = Math.round(view.fov*60*100)/100 + "'";
        }
        else {
            fovStr = Math.round(view.fov*3600*100)/100 + '"';
        }
        view.fovDiv.html("FoV: " + fovStr);
	};
	
	
	createListeners = function(view) {
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
            hasTouchEvents = true;
        }
        
        // various listeners
        onDblClick = function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
            try {
                var lonlat = view.projection.unproject(xy.x, xy.y);
            }
            catch(err) {
                return;
            }
            radec = [];
            // convert to J2000 if needed
            if (view.cooFrame==CooFrameEnum.GAL) {
                radec = CooConversion.GalacticToJ2000([lonlat.ra, lonlat.dec]);
            }
            else {
                radec = [lonlat.ra, lonlat.dec];
            }

            view.pointTo(radec[0], radec[1]);
        };
        if (! hasTouchEvents) {
            $(view.reticleCanvas).dblclick(onDblClick);
        }
        
        
        $(view.reticleCanvas).bind("mousedown touchstart", function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if (e.originalEvent && e.originalEvent.targetTouches) {
                view.dragx = e.originalEvent.targetTouches[0].clientX;
                view.dragy = e.originalEvent.targetTouches[0].clientY;
            }
            else {
                /*
                view.dragx = e.clientX;
                view.dragy = e.clientY;
                */
                view.dragx = xymouse.x;
                view.dragy = xymouse.y;
            }
            view.dragging = true;
            if (view.mode==View.PAN) {
                view.setCursor('move');
            }
            else if (view.mode==View.SELECT) {
                view.selectStartCoo = {x: view.dragx, y: view.dragy};
            }
            return false; // to disable text selection
        });
        var lastClickedObject; // save last object clicked by mouse
        $(view.reticleCanvas).bind("mouseup mouseout touchend", function(e) {
            if (view.mode==View.SELECT && view.dragging) {
                view.aladin.fire('selectend', 
                                 view.getObjectsInBBox(view.selectStartCoo.x, view.selectStartCoo.y,
                                                       view.dragx-view.selectStartCoo.x, view.dragy-view.selectStartCoo.y));    
            }
            if (view.dragging) {
                view.setCursor('default');
                view.dragging = false;
                
            }
            view.mustClearCatalog = true;
            view.mustRedrawReticle = true; // pour effacer selection bounding box
            view.dragx = view.dragy = null;


            if (e.type==="mouseout") {
                view.requestRedraw();
                return;
            }

            var xymouse = view.imageCanvas.relMouseCoords(e);
            // popup to show ?
            var objs = view.closestObjects(xymouse.x, xymouse.y, 5);
            if (objs) {
                var o = objs[0];
                // display marker
                if (o.marker) {
                    // could be factorized in Source.actionClicked
                    view.popup.setTitle(o.popupTitle);
                    view.popup.setText(o.popupDesc);
                    view.popup.setSource(o);
                    view.popup.show();
                }
                // show measurements
                else {
                    // TODO: show measurements
                    if (view.aladin.objClickedFunction) {
                        var ret = view.aladin.objClickedFunction(o);
                    }
                    else {
                        if (lastClickedObject) {
                            lastClickedObject.actionOtherObjectClicked();
                        }
                        o.actionClicked();
                    }
                    lastClickedObject = o;
                }
            }
            else {
                if (lastClickedObject) {
                    view.aladin.measurementTable.hide();
                    lastClickedObject.actionOtherObjectClicked();

                    lastClickedObject = null;
                    if (view.aladin.objClickedFunction) {
                        var ret = view.aladin.objClickedFunction(null);
                    }
                }
                
            }


            // TODO : remplacer par mécanisme de listeners
            // on avertit les catalogues progressifs
            view.refreshProgressiveCats();

            view.requestRedraw();
        });
        var lastHoveredObject; // save last object hovered by mouse
        $(view.reticleCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if (!view.dragging || hasTouchEvents) {
                    updateLocation(view, xymouse.x, xymouse.y, true);
                    /*
                    var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                    var lonlat;
                    try {
                        lonlat = view.projection.unproject(xy.x, xy.y);
                    }
                    catch(err) {
                    }
                    if (lonlat) {
                        view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, true);
                    }
                    */
                if (!view.dragging && ! view.mode==View.SELECT) {
                    // objects under the mouse ?
                    var closest = view.closestObjects(xymouse.x, xymouse.y, 5);
                    if (closest) {
                        view.setCursor('pointer');
                        if (view.aladin.objHoveredFunction && closest[0]!=lastHoveredObject) {
                            var ret = view.aladin.objHoveredFunction(closest[0]);
                        }
                        lastHoveredObject = closest[0];
                    }
                    else {
                        view.setCursor('default');
                        if (view.aladin.objHoveredFunction && lastHoveredObject) {
                            lastHoveredObject = null;
                            // call callback function to notify we left the hovered object
                            var ret = view.aladin.objHoveredFunction(null);
                        }
                    }
                }
                if (!hasTouchEvents) return;
            }

            var xoffset, yoffset;
            var pos1, pos2;
            
            if (e.originalEvent && e.originalEvent.targetTouches) {
                // ???
                xoffset = e.originalEvent.targetTouches[0].clientX-view.dragx;
                yoffset = e.originalEvent.targetTouches[0].clientY-view.dragy;
                var xy1 = AladinUtils.viewToXy(e.originalEvent.targetTouches[0].clientX, e.originalEvent.targetTouches[0].clientY, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);
            }
            else {
                /*
                xoffset = e.clientX-view.dragx;
                yoffset = e.clientY-view.dragy;
                */
                xoffset = xymouse.x-view.dragx;
                yoffset = xymouse.y-view.dragy;
                
                var xy1 = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                
                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);
                
            }
            
            // TODO : faut il faire ce test ??
//            var distSquared = xoffset*xoffset+yoffset*yoffset;
//            if (distSquared<3) {
//                return;
//            }
            if (e.originalEvent && e.originalEvent.targetTouches) {
                view.dragx = e.originalEvent.targetTouches[0].clientX;
                view.dragy = e.originalEvent.targetTouches[0].clientY;
            }
            else {
                view.dragx = xymouse.x;
                view.dragy = xymouse.y;
                /*
                view.dragx = e.clientX;
                view.dragy = e.clientY;
                */
            }
            
            if (view.mode==View.SELECT) {
                  view.requestRedraw();
                  return;
            }

            //view.viewCenter.lon += xoffset*view.mouseMoveIncrement/Math.cos(view.viewCenter.lat*Math.PI/180.0);
            /*
            view.viewCenter.lon += xoffset*view.mouseMoveIncrement;
            view.viewCenter.lat += yoffset*view.mouseMoveIncrement;
            */
            view.viewCenter.lon += pos2.ra -  pos1.ra;
            view.viewCenter.lat += pos2.dec - pos1.dec;
            

            
            // can not go beyond poles
            if (view.viewCenter.lat>90) {
                view.viewCenter.lat = 90;
            }
            else if (view.viewCenter.lat < -90) {
                view.viewCenter.lat = -90;
            }
            
            // limit lon to [0, 360]
            if (view.viewCenter.lon < 0) {
                view.viewCenter.lon = 360 + view.viewCenter.lon;
            }
            else if (view.viewCenter.lon > 360) {
                view.viewCenter.lon = view.viewCenter.lon % 360;
            }
            view.requestRedraw();
        }); //// endof mousemove ////
        
        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }

        $(view.reticleCanvas).on('mousewheel', function(event) {
            event.preventDefault();
            event.stopPropagation();
            var level = view.zoomLevel;
            var delta = event.deltaY;
            if (delta>0) {
                level += 1;
            }
            else {
                level -= 1;
            }
            view.setZoomLevel(level);
            
            return false;
        });

	};
	
	init = function(view) {
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#aladin-statsDiv').length>0) {
        	$('#aladin-statsDiv')[0].appendChild( stats.domElement );
        }
        
        view.stats = stats;

        createListeners(view);

        view.displayHpxGrid = false;
        view.displaySurvey = true;
        view.displayCatalog = false;
        view.displayReticle = true;
        
		// initial draw
		view.fov = computeFov(view);
		updateFovDiv(view);
		
		view.redraw();
	};

	function updateLocation(view, x, y, italic) {
	    if (!view.projection) {
	        return;
	    }
	    var xy = AladinUtils.viewToXy(x, y, view.width, view.height, view.largestDim, view.zoomFactor);
        var lonlat;
        try {
            lonlat = view.projection.unproject(xy.x, xy.y);
        }
        catch(err) {
        }
        if (lonlat) {
            view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, italic);
        }
	}
	
	View.prototype.requestRedrawAtDate = function(date) {
	    this.dateRequestDraw = date;
	};
	
	

	/**
	 * redraw the whole view
	 */
	View.prototype.redraw = function() {
		var saveNeedRedraw = this.needRedraw;
		requestAnimFrame(this.redraw.bind(this));

		var now = new Date().getTime();
		
		if (this.dateRequestDraw && now>this.dateRequestDraw) {
		    this.dateRequestDraw = null;
		} 
		else if (! this.needRedraw) {
            if ( ! this.flagForceRedraw) {
			    return;
            }
            else {
                this.flagForceRedraw = false;
            }
		}
		this.stats.update();

		var imageCtx = this.imageCtx;
		//////// 1. Draw images ////////
		
		//// clear canvas ////
		// TODO : do not need to clear if fov small enough ?
		imageCtx.clearRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
		////////////////////////
		
		// black background
        if (this.projectionMethod==ProjectionEnum.SIN) {
            if (this.fov>80) {
                imageCtx.fillStyle = "rgb(0,0,0)";
                imageCtx.beginPath();
                imageCtx.arc(this.cx, this.cy, this.cx*this.zoomFactor, 0, 2*Math.PI, true);
                imageCtx.fill();
            }
            // pour éviter les losanges blancs qui apparaissent quand les tuiles sont en attente de chargement
            else if (this.fov<60) {
                imageCtx.fillStyle = "rgb(0,0,0)";
                imageCtx.fillRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
            }
        }

        
        // TODO : voir si on doit vraiment faire ces vérifs à chaque coup
		if (!this.projection) {
			this.projection = new Projection(this.viewCenter.lon, this.viewCenter.lat);
		}
		else {
			this.projection.setCenter(this.viewCenter.lon, this.viewCenter.lat);
		}
		this.projection.setProjection(this.projectionMethod);
	

		// ************* Tracé au niveau allsky (faible résolution) *****************
		var cornersXYViewMapAllsky = this.getVisibleCells(3);
		var cornersXYViewMapHighres = null;
		if (this.curNorder>=3) {
			if (this.curNorder==3) {
				cornersXYViewMapHighres = cornersXYViewMapAllsky;
			}
			else {
				cornersXYViewMapHighres = this.getVisibleCells(this.curNorder);
			}
		}

		// redraw image survey
		if (this.imageSurvey && this.imageSurvey.isReady && this.displaySurvey) {
		    // TODO : a t on besoin de dessiner le allsky si norder>=3 ?
		    // TODO refactoring : devrait être une méthode de HpxImageSurvey
			this.imageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curNorder);
            if (this.curNorder>=3) {
                this.imageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, this.curNorder);
            }
		}
		

        // redraw overlay image survey
		// TODO : does not work if different frames 
		if (this.overlayImageSurvey && this.overlayImageSurvey.isReady) {
		    imageCtx.globalAlpha = this.overlayImageSurvey.getAlpha();
	        if (this.fov>50) {
		        this.overlayImageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curOverlayNorder);
	        }
	        if (this.curOverlayNorder>=3) {
                var norderOverlay = Math.min(this.curOverlayNorder, this.overlayImageSurvey.maxOrder);
                if ( norderOverlay != this.curNorder ) {
				    cornersXYViewMapHighres = this.getVisibleCells(norderOverlay);
                }
	            this.overlayImageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, norderOverlay);
	        }
           imageCtx.globalAlpha = 1.0;

		}
		
		
		// redraw HEALPix grid
        if( this.displayHpxGrid) {
        	if (cornersXYViewMapHighres && this.curNorder>3) {
        		this.healpixGrid.redraw(imageCtx, cornersXYViewMapHighres, this.fov, this.curNorder);
        	}
            else {
        	    this.healpixGrid.redraw(imageCtx, cornersXYViewMapAllsky, this.fov, 3);
            }
        }
        
        // redraw coordinates grid
        if (this.showGrid) {
            if (this.cooGrid==null) {
                this.cooGrid = new CooGrid();
            }
            
            this.cooGrid.redraw(imageCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.fov);
        }
 		


        
		////// 2. Draw catalogues////////
		var catalogCtx = this.catalogCtx;

		var catalogCanvasCleared = false;
        if (this.mustClearCatalog) {
            catalogCtx.clearRect(0, 0, this.width, this.height);
            catalogCanvasCleared = true;
            this.mustClearCatalog = false;
        }
		if (this.catalogs && this.catalogs.length>0 && this.displayCatalog && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
		      // TODO : ne pas effacer systématiquement
	        //// clear canvas ////
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
		    }
		    for (var i=0; i<this.catalogs.length; i++) {
		        this.catalogs[i].draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
		    }
        }

		////// 3. Draw overlays////////
        var overlayCtx = this.catalogCtx;
		if (this.overlays && this.overlays.length>0 && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
		    }
		    for (var i=0; i<this.overlays.length; i++) {
		        this.overlays[i].draw(overlayCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
		    }
        }
        

        // draw MOCs
        var mocCtx = this.catalogCtx;
		if (this.mocs && this.mocs.length>0 && (! this.dragging  || View.DRAW_MOCS_WHILE_DRAGGING)) {
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
		    }
            for (var i=0; i<this.mocs.length; i++) {
                this.mocs[i].draw(mocCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.fov);
            }
        }


		if (this.mode==View.SELECT) {
		    mustRedrawReticle = true;
		}
		////// 4. Draw reticle ///////
		// TODO: reticle should be placed in a static DIV, no need to waste a canvas
		var reticleCtx = this.reticleCtx;
		if (this.mustRedrawReticle || this.mode==View.SELECT) {
            reticleCtx.clearRect(0, 0, this.width, this.height);
		}
		if (this.displayReticle) {
		    
		    if (! this.reticleCache) {
    		    // build reticle image
    	        var c = document.createElement('canvas');
    	        var s = this.options.reticleSize;
    	        c.width = s;
    	        c.height = s;
    	        var ctx = c.getContext('2d');
    	        ctx.lineWidth = 2;
    	        ctx.strokeStyle = this.options.reticleColor;
    	        ctx.beginPath();
    	        ctx.moveTo(s/2, s/2+(s/2-1));
    	        ctx.lineTo(s/2, s/2+2);
    	        ctx.moveTo(s/2, s/2-(s/2-1));
    	        ctx.lineTo(s/2, s/2-2);
    	        
    	        ctx.moveTo(s/2+(s/2-1), s/2);
    	        ctx.lineTo(s/2+2,  s/2);
    	        ctx.moveTo(s/2-(s/2-1), s/2);
    	        ctx.lineTo(s/2-2,  s/2);
    	        
    	        ctx.stroke();
    	        
    	        this.reticleCache = c;
		    }
    	        
	        reticleCtx.drawImage(this.reticleCache, this.width/2 - this.reticleCache.width/2, this.height/2 - this.reticleCache.height/2);
		    
    		
    		this.mustRedrawReticle = false;
		}
		
		// draw selection box
		if (this.mode==View.SELECT && this.dragging) {
		    reticleCtx.fillStyle = "rgba(100, 240, 110, 0.25)";
		    var w = this.dragx - this.selectStartCoo.x;
		    var h =  this.dragy - this.selectStartCoo.y;
		    
		    reticleCtx.fillRect(this.selectStartCoo.x, this.selectStartCoo.y, w, h);
		}
        
        
 		// TODO : est ce la bonne façon de faire ?
 		if (saveNeedRedraw==this.needRedraw) {
 			this.needRedraw = false;
 		}


        // objects lookup
        if (!this.dragging) {
            this.updateObjectsLookup();
        }
	};

    View.prototype.forceRedraw = function() {
        this.flagForceRedraw = true;
    };
    
    View.prototype.refreshProgressiveCats = function() {
        if (! this.catalogs) {
            return;
        }
        for (var i=0; i<this.catalogs.length; i++) {
            if (this.catalogs[i].type=='progressivecat') {
                this.catalogs[i].loadNeededTiles();
            }
        }
    };

    View.prototype.getVisiblePixList = function(norder, frameSurvey) {
        var nside = Math.pow(2, norder);

        var pixList;
		var npix = HealpixIndex.nside2Npix(nside);
        if (this.fov>80) {
            pixList = [];
            for (var ipix=0; ipix<npix; ipix++) {
                pixList.push(ipix);
            }
        }
        else {
            var hpxIdx = new HealpixIndex(nside);
            hpxIdx.init();
            var spatialVector = new SpatialVector();
            // si frame != frame survey image, il faut faire la conversion dans le système du survey
            var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
            var radec = this.projection.unproject(xy.x, xy.y);
            var lonlat = [];
            if (frameSurvey && frameSurvey != this.cooFrame) {
                if (frameSurvey==CooFrameEnum.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]);
                }
                else if (frameSurvey==CooFrameEnum.GAL) {
                    lonlat = CooConversion.J2000ToGalactic([radec.ra, radec.dec]);
                }
            }
            else {
                lonlat = [radec.ra, radec.dec];
            }
            spatialVector.set(lonlat[0], lonlat[1]);
            var radius = this.fov*0.5*this.ratio;
            // we need to extend the radius
            if (this.fov>60) {
                radius *= 1.6;
            }
            else if (this.fov>12) {
                radius *=1.45;
            }
            else {
                radius *= 1.1;
            }



            pixList = hpxIdx.queryDisc(spatialVector, radius*Math.PI/180.0, true, true);
            // add central pixel at index 0
            var polar = Utils.radecToPolar(lonlat[0], lonlat[1]);
            ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
            pixList.unshift(ipixCenter);

        }

        return pixList;
    };
	
    // TODO: optimize this method !!
	View.prototype.getVisibleCells = function(norder, frameSurvey) {
	    if (! frameSurvey && this.imageSurvey) {
	        frameSurvey = this.imageSurvey.cooFrame;
	    }
		var cells = []; // will be returned
		var cornersXY = [];
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		var npix = HealpixIndex.nside2Npix(nside);
		var ipixCenter = null;
		
		// build list of pixels
        // TODO: pixList can be obtained from getVisiblePixList
		var pixList;
		if (this.fov>80) {
			pixList = [];
			for (var ipix=0; ipix<npix; ipix++) {
				pixList.push(ipix);
			}
		}
		else {
			var hpxIdx = new HealpixIndex(nside);
			hpxIdx.init();
			var spatialVector = new SpatialVector();
			// si frame != frame survey image, il faut faire la conversion dans le système du survey
			var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
			var radec = this.projection.unproject(xy.x, xy.y);
			var lonlat = [];
			if (frameSurvey && frameSurvey != this.cooFrame) {
				if (frameSurvey==CooFrameEnum.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]); 
                }
                else if (frameSurvey==CooFrameEnum.GAL) {
                    lonlat = CooConversion.J2000ToGalactic([radec.ra, radec.dec]);
                }
			}
			else {
				lonlat = [radec.ra, radec.dec];
			}
			spatialVector.set(lonlat[0], lonlat[1]);
			var radius = this.fov*0.5*this.ratio;
			// we need to extend the radius
			if (this.fov>60) {
				radius *= 1.6;
			}
			else if (this.fov>12) {
				radius *=1.45;
			}
            else {
                radius *= 1.1;
            }
			
			
				
			pixList = hpxIdx.queryDisc(spatialVector, radius*Math.PI/180.0, true, true);
			// add central pixel at index 0
			var polar = Utils.radecToPolar(lonlat[0], lonlat[1]);
			ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
			pixList.unshift(ipixCenter);
		}
		
		
		var ipix;
		var lon, lat;
		for (var ipixIdx=0, len=pixList.length; ipixIdx<len; ipixIdx++) {
			ipix = pixList[ipixIdx];
			if (ipix==ipixCenter && ipixIdx>0) { 
				continue;
			}
			var cornersXYView = [];
			corners = HealpixCache.corners_nest(ipix, nside);

			for (var k=0; k<4; k++) {
				spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	            // need for frame transformation ?
	            if (frameSurvey && frameSurvey != this.cooFrame) {
	                if (frameSurvey==CooFrameEnum.J2000) {
	                    var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	                else if (frameSurvey==CooFrameEnum.GAL) {
	                    var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	            }
	            else {
	                lon = spVec.ra();
	                lat = spVec.dec();
	            }
	            
				cornersXY[k] = this.projection.project(lon, lat);
			}


			if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
	            continue;
	        }



			for (var k=0; k<4; k++) {
				cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
			}

            var indulge = 10;
            // detect pixels outside view. Could be improved !
            // we minimize here the number of cells returned
            if( cornersXYView[0].vx<0 && cornersXYView[1].vx<0 && cornersXYView[2].vx<0 &&cornersXYView[3].vx<0) {
                continue;
            }
            if( cornersXYView[0].vy<0 && cornersXYView[1].vy<0 && cornersXYView[2].vy<0 &&cornersXYView[3].vy<0) {
                continue;
            }
            if( cornersXYView[0].vx>=this.width && cornersXYView[1].vx>=this.width && cornersXYView[2].vx>=this.width &&cornersXYView[3].vx>=this.width) {
                continue;
            }
            if( cornersXYView[0].vy>=this.height && cornersXYView[1].vy>=this.height && cornersXYView[2].vy>=this.height &&cornersXYView[3].vy>=this.height) {
                continue;
            }


			// check if pixel is visible
//			if (this.fov<160) { // don't bother checking if fov is large enough
//				if ( ! AladinUtils.isHpxPixVisible(cornersXYView, this.width, this.height) ) {
//					continue;
//				}
//			}
			// check if we have a pixel at the edge of the view in AITOFF --> TO BE MODIFIED
			if (this.projection.PROJECTION==ProjectionEnum.AITOFF) {
				var xdiff = cornersXYView[0].vx-cornersXYView[2].vx;
				var ydiff = cornersXYView[0].vy-cornersXYView[2].vy;
				var distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
				xdiff = cornersXYView[1].vx-cornersXYView[3].vx;
				ydiff = cornersXYView[1].vy-cornersXYView[3].vy;
				distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
			}
			
			cornersXYView.ipix = ipix;
			cells.push(cornersXYView);
		}
		
		return cells;
	};
	
	
	
	// get position in view for a given HEALPix cell
	View.prototype.getPositionsInView = function(ipix, norder) {
		var cornersXY = [];
		var lon, lat;
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		
		
		var cornersXYView = [];  // will be returned
		var corners = HealpixCache.corners_nest(ipix, nside);

		for (var k=0; k<4; k++) {
			spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	        // need for frame transformation ?
			if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
	            if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
	                var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	            else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
	                var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	        }
	        else {
	            lon = spVec.ra();
	            lat = spVec.dec();
	        }
	            
			cornersXY[k] = this.projection.project(lon, lat);
		}
		
		if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
            return null;
        }


		for (var k=0; k<4; k++) {
			cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
		}

		return cornersXYView;
	};
	
	
	View.prototype.computeZoomFactor = function(level) {
    	if (level>0) {
    	    return AladinUtils.getZoomFactorForAngle(180/Math.pow(1.15, level), this.projectionMethod);
		}
		else {
		    return 1 + 0.1*level;
		}
	};
	
	View.prototype.setZoom = function(fovDegrees) {
	    if (fovDegrees<0 || fovDegrees>180) {
	        return;
	    }
	    var zoomLevel = Math.log(180/fovDegrees)/Math.log(1.15);
	    this.setZoomLevel(zoomLevel);
	};
	
	View.prototype.setShowGrid = function(showGrid) {
	    this.showGrid = showGrid;
	    this.requestRedraw();
	};

	
    View.prototype.setZoomLevel = function(level) {
        if (this.minFOV || this.maxFOV) {
            var newFov = doComputeFov(this, this.computeZoomFactor(Math.max(-2, level)));
            if (this.maxFOV && newFov>this.maxFOV  ||  this.minFOV && newFov<this.minFOV)  {
                return;
            }
        }
        
        if (this.projectionMethod==ProjectionEnum.SIN) {
            if (this.aladin.options.allowFullZoomout === true) {
                // special case for Andreas Wicenec until I fix the problem
                if (this.width/this.height>2) {
                    this.zoomLevel = Math.max(-7, level); // TODO : canvas freezes in firefox when max level is small
                }
                else if (this.width/this.height<0.5) {
                    this.zoomLevel = Math.max(-2, level); // TODO : canvas freezes in firefox when max level is small
                }
                else {
                    this.zoomLevel = Math.max(-6, level); // TODO : canvas freezes in firefox when max level is small
                }
            }
            else {
                this.zoomLevel = Math.max(-2, level); // TODO : canvas freezes in firefox when max level is small
            }
        }
        else {
            this.zoomLevel = Math.max(-7, level); // TODO : canvas freezes in firefox when max level is small
        }
        
        
        this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
        
        this.fov = computeFov(this);
        updateFovDiv(this);
        
        this.computeNorder();
        
        this.forceRedraw();
		this.requestRedraw();
		
        // on avertit les catalogues progressifs
        if (! this.debounceProgCatOnZoom) {
            var self = this;
            this.debounceProgCatOnZoom = Utils.debounce(function() {self.refreshProgressiveCats();}, 300);
        }
        this.debounceProgCatOnZoom();
		
    };
    
    /**
     * compute and set the norder corresponding to the current view resolution
     */
    View.prototype.computeNorder = function() {
        var resolution = this.fov / this.largestDim; // in degree/pixel
        var tileSize = 512;
        var nside = HealpixIndex.calculateNSide(3600*tileSize*resolution); // 512 = taille d'une image "tuile"
        var norder = Math.log(nside)/Math.log(2);
        norder = Math.max(norder, 1);
        this.realNorder = norder;

            
        // forcer le passage à norder 3 (sinon, on reste flou trop longtemps)
        if (this.fov<=50 && norder<=2) {
            norder = 3;
        }
           

        // si l'on ne souhaite pas afficher le allsky
        if (this.imageSurvey && norder<=2 && this.imageSurvey.minOrder>2) {
            norder = this.imageSurvey.minOrder;
        }

        var overlayNorder  = norder;
        if (this.imageSurvey && norder>this.imageSurvey.maxOrder) {
            norder = this.imageSurvey.maxOrder;
        }
        if (this.overlayImageSurvey && overlayNorder>this.overlayImageSurvey.maxOrder) {
            overlayNorder = this.overlayImageSurvey.maxOrder;
        }
        // should never happen, as calculateNSide will return something <=HealpixIndex.ORDER_MAX
        if (norder>HealpixIndex.ORDER_MAX) {
            norder = HealpixIndex.ORDER_MAX;
        }
        if (overlayNorder>HealpixIndex.ORDER_MAX) {
            overlayNorder = HealpixIndex.ORDER_MAX;
        }
            
        this.curNorder = norder;
        this.curOverlayNorder = overlayNorder;

        
    };
	
    View.prototype.untaintCanvases = function() {
        this.createCanvases();
        createListeners(this);
        this.fixLayoutDimensions();
    };
    
    View.prototype.setOverlayImageSurvey = function(overlayImageSurvey, callback) {
        if (! overlayImageSurvey) {
            this.overlayImageSurvey = null;
            this.requestRedraw();
            return;
        }
        
        // reset canvas to "untaint" canvas if needed
        // we test if the previous base image layer was using CORS or not
        if ($.support.cors && this.overlayImageSurvey && ! this.overlayImageSurvey.useCors) {
            this.untaintCanvases();
        }
        
        var newOverlayImageSurvey;
        if (typeof overlayImageSurvey == "string") {
            newOverlayImageSurvey = HpxImageSurvey.getSurveyFromId(overlayImageSurvey);
            if ( ! newOverlayImageSurvey) {
                newOverlayImageSurvey = HpxImageSurvey.getSurveyFromId(HpxImageSurvey.DEFAULT_SURVEY_ID);
            }
        }
        else {
            newOverlayImageSurvey = overlayImageSurvey;
        }
        newOverlayImageSurvey.isReady = false;
        this.overlayImageSurvey = newOverlayImageSurvey;
        
        var self = this;
        newOverlayImageSurvey.init(this, function() {
            //self.imageSurvey = newImageSurvey;
            self.computeNorder();
            newOverlayImageSurvey.isReady = true;
            self.requestRedraw();
            self.updateObjectsLookup();
            
            if (callback) {
                callback();
            }
        });
    };

    View.prototype.setUnknownSurveyIfNeeded = function() {
        if (unknownSurveyId) {
            this.setImageSurvey(unknownSurveyId);
            unknownSurveyId = undefined;
        }
    }
    
    var unknownSurveyId = undefined;
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
	View.prototype.setImageSurvey = function(imageSurvey, callback) {
	    if (! imageSurvey) {
	        return;
	    }
	    
	    // reset canvas to "untaint" canvas if needed
	    // we test if the previous base image layer was using CORS or not
	    if ($.support.cors && this.imageSurvey && ! this.imageSurvey.useCors) {
	        this.untaintCanvases();
	    }
	    
		var newImageSurvey;
		if (typeof imageSurvey == "string") {
		    newImageSurvey = HpxImageSurvey.getSurveyFromId(imageSurvey);
		    if ( ! newImageSurvey) {
		        newImageSurvey = HpxImageSurvey.getSurveyFromId(HpxImageSurvey.DEFAULT_SURVEY_ID);
                unknownSurveyId = imageSurvey;
console.log(unknownSurveyId);
		    }
		}
		else {
		    newImageSurvey = imageSurvey;
		}
		
		// ràz buffer
		this.tileBuffer = new TileBuffer();
        
		newImageSurvey.isReady = false;
		this.imageSurvey = newImageSurvey;
		
        var self = this;
        newImageSurvey.init(this, function() {
            //self.imageSurvey = newImageSurvey;
            self.computeNorder();
            newImageSurvey.isReady = true;
            self.requestRedraw();
            self.updateObjectsLookup();
            
            if (callback) {
                callback();
            }
        });
	};
	
	View.prototype.requestRedraw = function() {
		this.needRedraw = true;
		//redraw(this);
	};
	
	View.prototype.changeProjection = function(projectionMethod) {
		this.projectionMethod = projectionMethod;
		this.requestRedraw();
	};

	View.prototype.changeFrame = function(cooFrame) {
		this.cooFrame = cooFrame;
        // recompute viewCenter
        if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = lb[0];
            this.viewCenter.lat = lb[1]; 
        }
        else if (this.cooFrame==CooFrameEnum.J2000) {
            var radec = CooConversion.GalacticToJ2000([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = radec[0];
            this.viewCenter.lat = radec[1]; 
        }
		this.requestRedraw();
	};

    View.prototype.showHealpixGrid = function(show) {
        this.displayHpxGrid = show;
        this.requestRedraw();
    };
    
    View.prototype.showSurvey = function(show) {
        this.displaySurvey = show;

        this.requestRedraw();
    };
    
    View.prototype.showCatalog = function(show) {
        this.displayCatalog = show;

        if (!this.displayCatalog) {
            this.mustClearCatalog = true;
        }
        this.requestRedraw();
    };
    
    View.prototype.showReticle = function(show) {
        this.displayReticle = show;

        this.mustRedrawReticle = true;
        this.requestRedraw();
    };

    View.prototype.pointTo = function(ra, dec) {
        ra = parseFloat(ra);
        dec = parseFloat(dec);
        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        if (this.cooFrame==CooFrameEnum.J2000) {
		    this.viewCenter.lon = ra;
		    this.viewCenter.lat = dec;
        }
        else if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([ra, dec]);
		    this.viewCenter.lon = lb[0];
		    this.viewCenter.lat = lb[1];
        }

        this.forceRedraw();
        this.requestRedraw();
        var self = this;
        setTimeout(function() {self.refreshProgressiveCats();}, 1000);

    };
    View.prototype.makeUniqLayerName = function(name) {
        if (! this.layerNameExists(name)) {
            return name;
        }
        for (var k=1;;++k) {
            var newName = name + '_' + k;
            if ( ! this.layerNameExists(newName)) {
                return newName;
            }
        }
    };
    View.prototype.layerNameExists = function(name) {
        var c = this.catalogs;
        for (var k=0; k<c.length; k++) {
            if (name==c[k].name) {
                return true;
            }
        }
        return false;
    };

    View.prototype.removeLayers = function() {
        this.catalogs = [];
        this.overlays = [];
        this.requestRedraw();
    };

    View.prototype.addCatalog = function(catalog) {
        catalog.name = this.makeUniqLayerName(catalog.name);
        this.catalogs.push(catalog);
        if (catalog.type=='catalog') {
            catalog.setView(this);
        }
        else if (catalog.type=='progressivecat') {
            catalog.init(this);
        }
    };
    View.prototype.addOverlay = function(overlay) {
        this.overlays.push(overlay);
        overlay.setView(this);
    };
    
    View.prototype.addMOC = function(moc) {
        this.mocs.push(moc);
        moc.setView(this);
    };
    
    View.prototype.getObjectsInBBox = function(x, y, w, h) {
        if (w<0) {
            x = x+w;
            w = -w;
        }
        if (h<0) {
            y = y+h;
            h = -h;
        }
        var objList = [];
        var cat, sources, s;
        if (this.catalogs) {
            for (var k=0; k<this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l=0; l<sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }
                    if (s.x>=x && s.x<=x+w && s.y>=y && s.y<=y+h) {
                        objList.push(s);
                    }
                }
            }
        }
        return objList;
        
    };

    // update objLookup, lookup table 
    View.prototype.updateObjectsLookup = function() {
        this.objLookup = [];

        var cat, sources, s, x, y;
        if (this.catalogs) {
            for (var k=0; k<this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l=0; l<sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }
                    x = s.x;
                    y = s.y;
                    if (!this.objLookup[x]) {
                        this.objLookup[x] = [];
                    }
                    if (!this.objLookup[x][y]) {
                        this.objLookup[x][y] = [];
                    }
                    this.objLookup[x][y].push(s);
                }       
            }           
        }     
    }

    // return closest object within a radius of maxRadius pixels. maxRadius is an integer
    View.prototype.closestObjects = function(x, y, maxRadius) {
        if (!this.objLookup) {
            return null;
        }
        var closest, dist;
        for (var r=0; r<=maxRadius; r++) {
            closest = dist = null;
            for (var dx=-maxRadius; dx<=maxRadius; dx++) {
                if (! this.objLookup[x+dx]) {
                    continue;
                }
                for (var dy=-maxRadius; dy<=maxRadius; dy++) {
                    if (this.objLookup[x+dx][y+dy]) {
                        if (!closest) {
                            closest = this.objLookup[x+dx][y+dy];
                        }
                        else {
                            var d = dx*dx + dy*dy;
                            if (d<dist) {
                                dist = d;
                                closest = this.objLookup[x+dx][y+dy];
                            }
                        }
                    }
                }
            }
            if (closest) {
                return closest;
            }
        }
        return null;
    };
    
    return View;
})();
