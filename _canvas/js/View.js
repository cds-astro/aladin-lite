/******************************************************************************
 * Aladin HTML5 project
 * 
 * File View.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

View = function(aladinDiv, imageCanvas, catalogCanvas, location, cooFrame) {
		this.aladinDiv = aladinDiv; 
		this.imageCanvas = imageCanvas;
		this.catalogCanvas = catalogCanvas;
		this.location = location;
		this.healpixGrid = new HealpixGrid(this.imageCanvas);
		if (cooFrame) {
            this.cooFrame = cooFrame;
        }
        else {
            this.cooFrame = CooFrameEnum.GAL;
        }
		this.projectionMethod = ProjectionEnum.SIN;
		this.projection = new Projection(0, 0);
		this.projection.setProjection(this.projectionMethod);
        this.zoomLevel = 0;
        this.zoomFactor = this.computeZoomFactor();
        
		this.viewCenter = {lon: 0, lat: 0}; // position of center of view
		
		// current image survey displayed
		this.imageSurvey = null;
		// current catalog displayed
		this.catalog = null;
		this.catalog = new Catalog();
		this.catalog.addSource(new Source(201.36508, -43.01911));
		this.catalog.addSource(new Source(0.0, 0.0));
//		this.catalog.addSource(new Source(10, 5));
//		this.catalog.addSource(new Source(0, 15));
//		this.catalog.addSource(new Source(23.0, -20));

		
		this.tileBuffer = new TileBuffer(); // tile buffer is shared across different image surveys
		this.fixLayoutDimensions();
        

//		this.width = this.imageCanvas.width;
//		this.height = this.imageCanvas.height;
//		
//		this.cx = this.width/2;
//		this.cy = this.height/2;
//		
//		this.largestDim = Math.max(this.width, this.height);
//		this.smallestDim = Math.min(this.width, this.height);
//		this.ratio = this.largestDim/this.smallestDim;
//
//		
//		this.mouseMoveIncrement = 160/this.largestDim;
		
		this.curNorder = 1;
		
		// some variables for mouse handling
		this.dragging = false;
		this.dragx = null;
		this.dragy = null;
		this.needRedraw = true;

        this.downloader = new Downloader(this); // the downloader object is shared across all HpxImageSurveys
        this.flagForceRedraw = false;

        this.fadingLatestUpdate = null;
		
        
		init(this);
		
		
//		this.resizeTimer = null;
//		var self = this;
//		$(window).resize(function() {
//		    clearTimeout(self.resizeTimer);
//		    self.resizeTimer = setTimeout(self.fixLayoutDimensions(self), 100);
//		});
	};
	
	View.DRAW_SOURCES_WHILE_DRAGGING = false;
	
	// called at startup and when window is resized
	View.prototype.fixLayoutDimensions = function() {
		this.width = $(this.aladinDiv).width();
		this.height = $(this.aladinDiv).height();
		
		
		this.cx = this.width/2;
		this.cy = this.height/2;
		
		this.largestDim = Math.max(this.width, this.height);
		this.smallestDim = Math.min(this.width, this.height);
		this.ratio = this.largestDim/this.smallestDim;

		
		this.mouseMoveIncrement = 160/this.largestDim;
		

		
		// reinitialize 2D context
		this.imageCtx = this.imageCanvas.getContext("2d");
		this.catalogCtx = this.catalogCanvas.getContext("2d");
		
		this.imageCtx.canvas.width = this.width;
		this.catalogCtx.canvas.width = this.width;
		
		this.imageCtx.canvas.height = this.height;
		this.catalogCtx.canvas.height = this.height;
	};
    





	/**
	 * Compute the FoV in degrees of the view and update mouseMoveIncrement
	 * 
	 * @param view
	 * @returns FoV (array of 2 elements : width and height) in degrees
	 */
	computeFov = function(view) {
		var fov;
		// if zoom factor < 1, we see 180°
		if (view.zoomFactor<1) {
			fov = 180;
		}
		else {
			// TODO : fov sur les 2 dimensions !!
			// to compute FoV, we first retrieve 2 points at coordinates (0, view.cy) and (width-1, view.cy)
			var xy1 = AladinUtils.viewToXy(0, view.cy, view.width, view.height, view.largestDim, view.zoomFactor);
			var lonlat1 = view.projection.unproject(xy1.x, xy1.y);
			
			var xy2 = AladinUtils.viewToXy(view.imageCanvas.width-1, view.cy, view.width, view.height, view.largestDim, view.zoomFactor);
			var lonlat2 = view.projection.unproject(xy2.x, xy2.y);
			
			
			fov = new Coo(lonlat1.ra, lonlat1.dec).distance(new Coo(lonlat2.ra, lonlat2.dec));
		}
		
		view.mouseMoveIncrement = fov/view.imageCanvas.width;
			
		return fov;
	};
	
	/**
	 * compute the norder corresponding to the current view resolution
	 */
	computeNOrder = function(view) {
		var resolution = view.fov / view.largestDim; // in degree/pixel
		var tileSize = 512;
		var nside = HealpixIndex.calculateNSide(3600*tileSize*resolution); // 512 = taille d'une image "tuile"
		var norder = Math.log(nside)/Math.log(2);
		//norder += 1;
		norder = Math.max(norder, 1);

		// forcer le passage à norder 3?
//		if (view.fov<50 && norder==2) {
//			norder = 3;
//		}
		
        if (view.imageSurvey && norder>view.imageSurvey.maxOrder) {
            norder = view.imageSurvey.maxOrder;
        }
        // should never happen, as calculateNSide will return something <=HealpixIndex.ORDER_MAX
        if (norder>HealpixIndex.ORDER_MAX) {
        	norder = HealpixIndex.ORDER_MAX;
        }
		return norder;
	};
	
	init = function(view) {

		
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#statsDiv').length>0) {
        	$('#statsDiv')[0].appendChild( stats.domElement );
        }
        
        view.stats = stats;
		
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
            $(view.catalogCanvas).dblclick(onDblClick);
        }
        
        
		$(view.catalogCanvas).bind("mousedown touchstart", function(e) {
			if (e.originalEvent && e.originalEvent.targetTouches) {
				view.dragx = e.originalEvent.targetTouches[0].clientX;
				view.dragy = e.originalEvent.targetTouches[0].clientY;
			}
			else {
				view.dragx = e.clientX;
				view.dragy = e.clientY;
			}
			view.dragging = true;
            view.catalogCanvas.style.cursor = 'move';
            return false; // to disable text selection
		});
		$(view.catalogCanvas).bind("mouseup mouseout touchend", function(e) {
			view.dragx = view.dragy = null;
			view.dragging = false;
            view.catalogCanvas.style.cursor = 'default';
			view.requestRedraw();
		});
		$(view.catalogCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();

			if (!view.dragging || hasTouchEvents) {
				if (view.projection) {
					var xymouse = view.imageCanvas.relMouseCoords(e);
					var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
					var lonlat;
					try {
						lonlat = view.projection.unproject(xy.x, xy.y);
					}
					catch(err) {
					}
					if (lonlat) {
						view.location.update(lonlat.ra, lonlat.dec, view.cooFrame);
					}
				}
				if (!hasTouchEvents) return;
			}

			var xoffset, yoffset;
			if (e.originalEvent && e.originalEvent.targetTouches) {
				xoffset = e.originalEvent.targetTouches[0].clientX-view.dragx;
				yoffset = e.originalEvent.targetTouches[0].clientY-view.dragy;
			}
			else {
				xoffset = e.clientX-view.dragx;
				yoffset = e.clientY-view.dragy;
			}
			var distSquared = xoffset*xoffset+yoffset*yoffset;
			// TODO : faut il faire ce test ??
			if (distSquared<3) {
				return;
			}
			if (e.originalEvent && e.originalEvent.targetTouches) {
				view.dragx = e.originalEvent.targetTouches[0].clientX;
				view.dragy = e.originalEvent.targetTouches[0].clientY;
			}
			else {
				view.dragx = e.clientX;
				view.dragy = e.clientY;
			}

			view.viewCenter.lon += xoffset*view.mouseMoveIncrement;
			view.viewCenter.lat += yoffset*view.mouseMoveIncrement;

			
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

		$(view.catalogCanvas).bind('mousewheel', function(event, delta) {
			event.preventDefault();
			event.stopPropagation();
			var level = view.zoomLevel;
			if (delta>0) {
				level += 1;
			}
			else {
				level -= 1;
			}
			view.setZoomLevel(level);
			
			return false;
		});
	
        view.displayHpxGrid = false;
        view.displaySurvey = true;
        view.displayCataog = false;
		// initial draw
		view.fov = computeFov(view);
		// TODO : voir comment séparer cette dépendance de la vue
		window.view = view;
		redraw(view);
	};

	/**
	 * redraw the whole view
	 */
	redraw = function() {
		//console.log(view);
		
		var saveNeedRedraw = view.needRedraw;
		requestAnimFrame(redraw);

		// TODO : à remettre
		if (! view.needRedraw) {
            if ( ! view.flagForceRedraw) {
			    return;
            }
            else {
                view.flagForceRedraw = false;
            }
		}
		view.stats.update();

		var imageCtx = view.imageCtx;
		//////// 1. Draw images ////////
		
		//// clear canvas ////
		imageCtx.clearRect(0, 0, view.imageCanvas.width, view.imageCanvas.height);
		////////////////////////
		
		// black background
        if (view.fov>80 && view.projectionMethod==ProjectionEnum.SIN) {
        	imageCtx.fillStyle = "rgb(0,0,0)";
        	imageCtx.beginPath();
        	imageCtx.arc(view.cx, view.cy, view.cx*view.zoomFactor, 0, 2*Math.PI, true);
        	imageCtx.fill();
        }

		if (!view.projection) {
			view.projection = new Projection(view.viewCenter.lon, view.viewCenter.lat);
		}
		else {
			view.projection.setCenter(view.viewCenter.lon, view.viewCenter.lat);
		}
		view.projection.setProjection(view.projectionMethod);
	

		// ************* Tracé au niveau allsky (faible résolution) *****************
		var cornersXYViewMapAllsky = view.getVisibleCells(3);
		var cornersXYViewMapHighres = null;
		if (view.curNorder>=3) {
			if (view.curNorder==3) {
				cornersXYViewMapHighres = cornersXYViewMapAllsky;
			}
			else {
				cornersXYViewMapHighres = view.getVisibleCells(view.curNorder);
			}
		}

		// redraw image survey
		if (view.imageSurvey && view.displaySurvey) {
			view.imageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, view.fov, view.curNorder);
            if (view.curNorder>=3) {
                view.imageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, view.curNorder);
            }
		}
		
		
		
		
		
		
		// redraw grid
        if( view.displayHpxGrid) {
        	if (cornersXYViewMapHighres && view.curNorder>3) {
        		view.healpixGrid.redraw(imageCtx, cornersXYViewMapHighres, view.fov, view.curNorder);
        	}
            else {
        	    view.healpixGrid.redraw(imageCtx, cornersXYViewMapAllsky, view.fov, 3);
            }
        }
 		
 		// draw FoV value
        imageCtx.beginPath();
        imageCtx.font = "16pt";
        imageCtx.fillStyle = "rgb(230,120,250)";
        imageCtx.textWidth = 2.5;
        imageCtx.fillText("FoV: " + Math.round(view.fov*100)/100 + "°", 20, view.height-20);
        imageCtx.stroke();

        
		////// 2. Draw catalogues////////
		var catalogCtx = view.catalogCtx;
		//// clear canvas ////
		catalogCtx.clearRect(0, 0, view.width, view.height);
		if (view.catalog && view.displayCatalog && (! view.dragging || View.DRAW_SOURCES_WHILE_DRAGGING)) {
        	view.catalog.draw(catalogCtx, view.projection, view.width, view.height, view.largestDim, view.zoomFactor, view.cooFrame);
        }
        
        
 		// TODO : est ce la bonne façon de faire ?
 		if (saveNeedRedraw==view.needRedraw) {
 			view.needRedraw = false;
 		}
	};

    View.prototype.forceRedraw = function() {
        this.flagForceRedraw = true;
    };
	
	View.prototype.getVisibleCells = function(norder) {
		var cells = []; // will be returned
		var cornersXY = [];
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		var npix = HealpixIndex.nside2Npix(nside);
		var ipixCenter = null;
		
		// build list of pixels
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
			if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
				if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]); 
                }
                else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
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
	            continue;
	        }


			for (var k=0; k<4; k++) {
				cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
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
	
	
	View.prototype.computeZoomFactor = function() {
    	if (this.zoomLevel>0) {
		    //return 1+0.035*Math.pow(this.zoomLevel, 2);
            //return 1/Math.pow(0.94, this.zoomLevel);
    		return 1/Math.pow(0.9, this.zoomLevel);
		}
		else {
		    return 1 + 0.1*this.zoomLevel;
		}
		/*
		if (this.zoomLevel==0) {
			return 1;
		}
		if (this.zoomLevel>0) {
			return Math.sqrt(0.3+this.zoomLevel);
		}
		if (this.zoomLevel<0) {
			return 1/Math.log(10-this.zoomLevel);
		}
		*/
	};

    View.prototype.setZoomLevel = function(level) {
        this.zoomLevel = Math.max(-3, level);
        this.zoomFactor = this.computeZoomFactor();
        this.fov = computeFov(this);
        this.curNorder = computeNOrder(this);

		this.requestRedraw();
        this.forceRedraw();
    };
	
	View.prototype.setImageSurvey = function(imageSurvey) {
		this.imageSurvey = null;
		this.imageSurvey = imageSurvey;
        this.curNorder = computeNOrder(this);
        this.imageSurvey.init(this);
        this.requestRedraw();
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

        this.requestRedraw();
        this.forceRedraw();
    };
	
