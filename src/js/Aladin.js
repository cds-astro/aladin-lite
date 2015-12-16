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
 * File Aladin.js (main class)
 * Facade to expose Aladin Lite methods
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Aladin = (function() {
    
    // Constructor
    var Aladin = function(aladinDiv, requestedOptions) {
        // check that aladinDiv exists, stop immediately otherwise
        if ($(aladinDiv).length==0) {
            console.log('Could not find div ' + aladinDiv + '. Aborting creation of Aladin Lite instance');
            return;
        }


	    HealpixCache.init();
        
	    var self = this;
	    
	    // if not options was set, try to retrieve them from the query string
	    if (requestedOptions===undefined) {
	        requestedOptions = this.getOptionsFromQueryString();
	    }
	    requestedOptions = requestedOptions || {};
	    
	    
	    // 'fov' option was previsouly called 'zoom'
	    if ('zoom' in requestedOptions) {
	        var fovValue = requestedOptions.zoom;
	        delete requestedOptions.zoom;
	        requestedOptions.fov = fovValue;
	    }
	    // merge with default options
	    var options = {};
	    for (var key in Aladin.DEFAULT_OPTIONS) {
	        if (requestedOptions[key] !== undefined) {
	            options[key] = requestedOptions[key];
	        }
	        else {
	            options[key] = Aladin.DEFAULT_OPTIONS[key];
	        }
	    }
	    for (var key in requestedOptions) {
	        if (Aladin.DEFAULT_OPTIONS[key]===undefined) {
	            options[key] = requestedOptions[key];
	        }
	    }
	    
        this.options = options;

	    

		this.aladinDiv = aladinDiv;

		// parent div
		$(aladinDiv).addClass("aladin-container");
		
	      
		var cooFrame = CooFrameEnum.fromString(options.cooFrame, CooFrameEnum.J2000);
		// div where we write the position
		var frameInJ2000 = cooFrame==CooFrameEnum.J2000;
        
		var locationDiv = $('<div class="aladin-location">'
		                    + (options.showFrame ? '<select class="aladin-frameChoice"><option value="' + CooFrameEnum.J2000 + '" '
		                    + (frameInJ2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.GAL + '" '
		                    + (! frameInJ2000 ? 'selected="selected"' : '') + '>GAL</option></select>' : '')
		                    + '<span class="aladin-location-text"></span></div>')
		                    .appendTo(aladinDiv);
		// div where FoV value is written
		var fovDiv = $('<div class="aladin-fov"></div>').appendTo(aladinDiv);
		
		
		// zoom control
        if (options.showZoomControl) {
	          $('<div class="aladin-zoomControl"><a href="#" class="zoomPlus" title="Zoom in">+</a><a href="#" class="zoomMinus" title="Zoom out">&ndash;</a></div>').appendTo(aladinDiv);
	    }
        
        // maximize control
        if (options.showFullscreenControl) {
            $('<div class="aladin-fullscreenControl aladin-maximize" title="Full screen"></div>')
                .appendTo(aladinDiv);
        }
        this.fullScreenBtn = $(aladinDiv).find('.aladin-fullscreenControl')
        this.fullScreenBtn.click(function() {
            self.toggleFullscreen();
        });

        



		// Aladin logo
		$("<div class='aladin-logo-container'><a href='http://aladin.u-strasbg.fr/' title='Powered by Aladin Lite' target='_blank'><div class='aladin-logo'></div></a></div>").appendTo(aladinDiv);
		
		
		// we store the boxes
		this.boxes = [];

        // measurement table
        this.measurementTable = new MeasurementTable(aladinDiv);

		
		
		var location = new Location(locationDiv.find('.aladin-location-text'));
        
		// set different options
		this.view = new View(this, location, fovDiv, cooFrame, options.fov);
		this.view.setShowGrid(options.showCooGrid);

	    // retrieve available surveys
	    $.ajax({
	        url: "http://aladin.u-strasbg.fr/java/nph-aladin.pl",
	        data: {"frame": "aladinLiteDic"},
	        method: 'GET',
	        dataType: 'jsonp',
	        success: function(data) {
                var map = {};
                for (var k=0; k<data.length; k++) {
                    map[data[k].id] = true;
                }
                // retrieve existing surveys
                for (var k=0; k<HpxImageSurvey.SURVEYS.length; k++) {
                    if (! map[HpxImageSurvey.SURVEYS[k].id]) {
                        data.push(HpxImageSurvey.SURVEYS[k]);
                    }
                }
	            HpxImageSurvey.SURVEYS = data;
                self.view.setUnknownSurveyIfNeeded();
	        },
	        error: function() {
	        }
	    });
		
	      // layers control panel
        // TODO : valeur des checkbox en fonction des options
		// TODO : classe LayerBox
        if (options.showLayersControl) {
            var d = $('<div class="aladin-layersControl-container" title="Manage layers"><div class="aladin-layersControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var layerBox = $('<div class="aladin-box aladin-layerBox aladin-cb-list"></div>');
            layerBox.appendTo(aladinDiv);
            
            this.boxes.push(layerBox);
            
            // we return false so that the default event is not submitted, and to prevent event bubbling
            d.click(function() {self.hideBoxes();self.showLayerBox();return false;});

        }

        
        // goto control panel
        if (options.showGotoControl) {
            var d = $('<div class="aladin-gotoControl-container" title="Go to position"><div class="aladin-gotoControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var gotoBox = 
                $('<div class="aladin-box aladin-gotoBox">' +
                  '<a class="aladin-closeBtn">&times;</a>' +
                  '<div style="clear: both;"></div>' +
                  '<form class="aladin-target-form">Go to: <input type="text" placeholder="Object name/position" /></form></div>');
            gotoBox.appendTo(aladinDiv);
            this.boxes.push(gotoBox);
            
            var input = gotoBox.find('.aladin-target-form input');
            input.on("paste keydown", function() {
                $(this).removeClass('aladin-unknownObject'); // remove red border
            });
            
            // TODO : classe GotoBox
            d.click(function() {
                self.hideBoxes();
                input.val('');
                input.removeClass('aladin-unknownObject');
                gotoBox.show();
                input.focus();
                
                
                return false;
            });
            gotoBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
        }
        
        // share control panel
        if (options.showShareControl) {
            var d = $('<div class="aladin-shareControl-container" title="Share current view"><div class="aladin-shareControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var shareBox = 
                $('<div class="aladin-box aladin-shareBox">' +
                  '<a class="aladin-closeBtn">&times;</a>' +
                  '<div style="clear: both;"></div>' +
                  '<b>Share</b>' +
                  '<input type="text" class="aladin-shareInput" />' +
                  '</div>');
            shareBox.appendTo(aladinDiv);
            this.boxes.push(shareBox);
            
            
            // TODO : classe GotoBox
            d.click(function() {
                self.hideBoxes();
                shareBox.show();
                
                
                return false;
            });
            shareBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
        }
		
		
        this.gotoObject(options.target);

        if (options.log) {
            var params = requestedOptions;
            params['version'] = Aladin.VERSION;
            Logger.log("startup", params);
        }
        
		this.showReticle(options.showReticle);
		
		if (options.catalogUrls) {
		    for (var k=0, len=options.catalogUrls.length; k<len; k++) {
		        this.createCatalogFromVOTable(options.catalogUrls[k]);
		    }
		}
		
		this.setImageSurvey(options.survey);
		this.view.showCatalog(options.showCatalog);
		
	    
    	var aladin = this;
    	$(aladinDiv).find('.aladin-frameChoice').change(function() {
    		aladin.setFrame($(this).val());
    	});
    	$('#projectionChoice').change(function() {
    		aladin.setProjection($(this).val());
    	});
        

        $(aladinDiv).find('.aladin-target-form').submit(function() {
            aladin.gotoObject($(this).find('input').val(), function() {
                $(aladinDiv).find('.aladin-target-form input').addClass('aladin-unknownObject');
            });
            return false;
        });
        
        var zoomPlus = $(aladinDiv).find('.zoomPlus');
        zoomPlus.click(function() {
        	aladin.increaseZoom();
        	return false;
        });
        zoomPlus.bind('mousedown', function(e) {
            e.preventDefault(); // to prevent text selection
        });
        
        var zoomMinus = $(aladinDiv).find('.zoomMinus');
        zoomMinus.click(function() {
            aladin.decreaseZoom();
            return false;
        });
        zoomMinus.bind('mousedown', function(e) {
            e.preventDefault(); // to prevent text selection
        });
        
        // go to full screen ?
        if (options.fullScreen) {
            window.setTimeout(function() {self.toggleFullscreen();}, 1000);
        }
	};
	
    /**** CONSTANTS ****/
    Aladin.VERSION = "{ALADIN-LITE-VERSION-NUMBER}"; // will be filled by the build.sh script
    
    Aladin.JSONP_PROXY = "http://alasky.u-strasbg.fr/cgi/JSONProxy";
    
    Aladin.DEFAULT_OPTIONS = {
        target:                 "0 +0",
        cooFrame:               "J2000",
        survey:                 "P/DSS2/color",
        fov:                    60,
        showReticle:            true,
        showZoomControl:        true,
        showFullscreenControl:  true,
        showLayersControl:      true,
        showGotoControl:        true,
        showShareControl:       false,
        showCatalog:            true, // TODO: still used ??
        showFrame:              true,
        showCooGrid:            false,
        fullScreen:             false,
        reticleColor:           "rgb(178, 50, 178)",
        reticleSize:            22,
        log:                    true,
        allowFullZoomout:       false
    };

    
    Aladin.prototype.toggleFullscreen = function() {
        this.fullScreenBtn.toggleClass('aladin-maximize aladin-restore');
        var isInFullscreen = this.fullScreenBtn.hasClass('aladin-restore');
        this.fullScreenBtn.attr('title', isInFullscreen ? 'Restore original size' : 'Full screen');
        $(this.aladinDiv).toggleClass('aladin-fullscreen');
        
        this.view.fixLayoutDimensions();
    };
    
    Aladin.prototype.updateSurveysDropdownList = function(surveys) {
        surveys = surveys.sort(function(a, b) {
            if (! a.order) {
                return a.id > b.id;
            }
            return a.order && a.order > b.order ? 1 : -1;
        });
        var select = $(this.aladinDiv).find('.aladin-surveySelection');
        select.empty();
        for (var i=0; i<surveys.length; i++) {
            var isCurSurvey = this.view.imageSurvey.id==surveys[i].id;
            select.append($("<option />").attr("selected", isCurSurvey).val(surveys[i].id).text(surveys[i].name));
        };
    };
    
    Aladin.prototype.getOptionsFromQueryString = function() {
        var options = {};
        var requestedTarget = $.urlParam('target');
        if (requestedTarget) {
            options.target = requestedTarget;
        }
        var requestedFrame = $.urlParam('frame');
        if (requestedFrame && CooFrameEnum[requestedFrame] ) {
            options.frame = requestedFrame;
        }
        var requestedSurveyId = $.urlParam('survey');
        if (requestedSurveyId && HpxImageSurvey.getSurveyInfoFromId(requestedSurveyId)) {
            options.survey = requestedSurveyId;
        }
        var requestedZoom = $.urlParam('zoom');
        if (requestedZoom && requestedZoom>0 && requestedZoom<180) {
            options.zoom = requestedZoom;
        }
        
        var requestedShowreticle = $.urlParam('showReticle');
        if (requestedShowreticle) {
            options.showReticle = requestedShowreticle.toLowerCase()=='true';
        }
        
        var requestedCooFrame =  $.urlParam('cooFrame');
        if (requestedCooFrame) {
            options.cooFrame = requestedCooFrame;
        }
        
        var requestedFullscreen =  $.urlParam('fullScreen');
        if (requestedFullscreen !== undefined) {
            options.fullScreen = requestedFullscreen;
        }
        
        return options;
    };
	
    // TODO: rename to setFoV
    //@oldAPI
	Aladin.prototype.setZoom = function(fovDegrees) {
		this.view.setZoom(fovDegrees);
	};

	// @API
	Aladin.prototype.setFoV = Aladin.prototype.setFov = function(fovDegrees) {
		this.view.setZoom(fovDegrees);
	};
	
    Aladin.prototype.setFrame = function(frameName) {
        if (! frameName) {
            return;
        }
        var newFrame = CooFrameEnum.fromString(frameName, CooFrameEnum.J2000);
        if (newFrame==this.view.cooFrame)  {
            return;
        }

        this.view.changeFrame(newFrame);
        // màj select box
        $(this.aladinDiv).find('.aladin-frameChoice').val(newFrame);
    };

	Aladin.prototype.setProjection = function(projectionName) {
		if (! projectionName) {
			return;
		}
		projectionName = projectionName.toLowerCase();
		switch(projectionName) {
			case "aitoff":
				this.view.changeProjection(ProjectionEnum.AITOFF);
				break;
			case "sinus":
			default:
				this.view.changeProjection(ProjectionEnum.SIN);
		}
	};
    
    // point view to a given object (resolved by Sesame) or position
    // TODO: should we use function 
    Aladin.prototype.gotoObject = function(targetName, errorCallback) {
    	var isObjectName = /[a-zA-Z]/.test(targetName);
    	
    	// try to parse as a position
    	if ( ! isObjectName) {
    		var coo = new Coo();

			coo.parse(targetName);
			var lonlat = [coo.lon, coo.lat];
			if (this.view.cooFrame == CooFrameEnum.GAL) {
				lonlat = CooConversion.GalacticToJ2000(lonlat);
			}
    		this.view.pointTo(lonlat[0], lonlat[1]);
    	}
    	// ask resolution by Sesame
    	else {
	        var self = this;
	        Sesame.resolve(targetName,
	                       function(data) {
	        					   var ra = data.Target.Resolver.jradeg;
	        					   var dec = data.Target.Resolver.jdedeg;
	        					   self.view.pointTo(ra, dec);
	        				   /*
	                           if (data.sesame.error) {
	                                if (console) console.log(data.sesame.error);
	                           }
	                           else {
	                               var radec = data.sesame.decimalPosition.split(" ");
	                               self.view.pointTo(parseFloat(radec[0]), parseFloat(radec[1]));
	                           }
	                           */
	                       },
	                       function(data) {
	                            if (console) {
	                                console.log("Could not resolve object name " + targetName);
	                                console.log(data);
	                            }
	                            if (errorCallback) {
	                                errorCallback();
	                            }
	                       });
    	}
    };
    
    
    
    /**
     * go to a given position, expressed in the current coordinate frame
     * 
     * @API
     */
    Aladin.prototype.gotoPosition = function(lon, lat) {
        var radec;
        // first, convert to J2000 if needed
        if (this.view.cooFrame==CooFrameEnum.GAL) {
            radec = CooConversion.GalacticToJ2000([lon, lat]);
        }
        else {
            radec = [lon, lat];
        }
    	this.view.pointTo(radec[0], radec[1]);
    };
    
    
    var doAnimation = function(aladin) {
        var params = aladin.animationParams;
        if (params==null) {
            return;
        }
        var now = new Date().getTime();
        // this is the animation end: set the view to the end position, and call complete callback 
        if (now>params['end']) {
            aladin.gotoRaDec(params['raEnd'], params['decEnd']);
            
            if (params['complete']) {
                params['complete']();
            }
            
            return;
        }
        
        // compute current position
        var curRa =  params['raStart'] + (params['raEnd'] - params['raStart']) * (now-params['start']) / (params['end'] - params['start']);
        var curDec = params['decStart'] + (params['decEnd'] - params['decStart']) * (now-params['start']) / (params['end'] - params['start']);
        
        aladin.gotoRaDec(curRa, curDec);
        
        setTimeout(function() {doAnimation(aladin);}, 50);
        
    };
    /*
     * animate smoothly from the current position to the given ra, dec
     * 
     * the total duration (in seconds) of the animation can be given (otherwise set to 5 seconds by default)
     * 
     * complete: a function to call once the animation has completed
     * 
     * @API
     * 
     */
    Aladin.prototype.animateToRaDec = function(ra, dec, duration, complete) {
        duration = duration || 5;
        
        this.animationParams = null;
        doAnimation(this);
        
        var animationParams = {};
        animationParams['start'] = new Date().getTime();
        animationParams['end'] = new Date().getTime() + 1000*duration;
        var raDec = this.getRaDec();
        animationParams['raStart'] = raDec[0];
        animationParams['decStart'] = raDec[1];
        animationParams['raEnd'] = ra;
        animationParams['decEnd'] = dec;
        animationParams['complete'] = complete;
        
        this.animationParams = animationParams;
        
        doAnimation(this);
    };
    
    /**
     * get current [ra, dec] position of the center of the view
     * 
     * @API
     */
    Aladin.prototype.getRaDec = function() {
        if (this.view.cooFrame==CooFrameEnum.J2000) {
            return [this.view.viewCenter.lon, this.view.viewCenter.lat];
        }
        else {
            var radec = CooConversion.GalacticToJ2000([this.view.viewCenter.lon, this.view.viewCenter.lat]);
            return radec;
            
        }
    };
    
    
    /**
     * point to a given position, expressed as a ra,dec coordinate
     * 
     * @API
     */
    Aladin.prototype.gotoRaDec = function(ra, dec) {
        this.view.pointTo(ra, dec);
    };

    Aladin.prototype.showHealpixGrid = function(show) {
        this.view.showHealpixGrid(show);
    };
    
    Aladin.prototype.showSurvey = function(show) {
        this.view.showSurvey(show);
    };
    Aladin.prototype.showCatalog = function(show) {
        this.view.showCatalog(show);
    };
    Aladin.prototype.showReticle = function(show) {
        this.view.showReticle(show);
        $('#displayReticle').attr('checked', show);
    };
    Aladin.prototype.removeLayers = function() {
        this.view.removeLayers();
    };

    // should be merged into a unique "add" method
    Aladin.prototype.addCatalog = function(catalog) {
        this.view.addCatalog(catalog);
    };
    Aladin.prototype.addOverlay = function(overlay) {
        this.view.addOverlay(overlay);
    };
    Aladin.prototype.addMOC = function(moc) {
        this.view.addMOC(moc);
    };
    

  
    // @oldAPI
    Aladin.prototype.createImageSurvey = function(id, name, rootUrl, cooFrame, maxOrder, options) {
        return new HpxImageSurvey(id, name, rootUrl, cooFrame, maxOrder, options);        
    };


 
    // @api
    Aladin.prototype.getBaseImageLayer = function() {
        return this.view.imageSurvey;
    };
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
    // @api
    // @old
    Aladin.prototype.setImageSurvey = function(imageSurvey, callback) {
        this.view.setImageSurvey(imageSurvey, callback);
        this.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        if (this.options.log) {
            var id = imageSurvey;
            if (typeof imageSurvey !== "string") {
                id = imageSurvey.rootUrl;
            }

            Logger.log("changeImageSurvey", id);
        }
    };
    // @api
    Aladin.prototype.setBaseImageLayer = Aladin.prototype.setImageSurvey;
    
    // @api
    Aladin.prototype.getOverlayImageLayer = function() {
        return this.view.overlayImageSurvey;
    };
    // @api
    Aladin.prototype.setOverlayImageLayer = function(imageSurvey, callback) {
        this.view.setOverlayImageSurvey(imageSurvey, callback);
    };
    

    Aladin.prototype.increaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel+step);
    };
    
    Aladin.prototype.decreaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel-step);
    };
    
    // @oldAPI
    Aladin.prototype.createCatalog = function(options) {
        return A.catalog(options);
    };


    Aladin.prototype.createProgressiveCatalog = function(url, frame, maxOrder, options) {
        return new ProgressiveCat(url, frame, maxOrder, options);
    };
    
    // @oldAPI
    Aladin.prototype.createSource = function(ra, dec, data) {
        return new cds.Source(ra, dec, data);
    };
    // @oldAPI
    Aladin.prototype.createMarker = function(ra, dec, options, data) {
        options = options || {};
        options['marker'] = true;
        return new cds.Source(ra, dec, data, options);
    };

    Aladin.prototype.createOverlay = function(options) {
        return new Overlay(options);
    };

    // API
    Aladin.prototype.createFootprintsFromSTCS = function(stcs) {
        var polygons = Overlay.parseSTCS(stcs);
        var fps = [];
        for (var k=0, len=polygons.length; k<len; k++) {
            fps.push(new Footprint(polygons[k]));
        }
        return fps;
    };

    // API
    A.MOCFromURL = function(url, options, successCallback) {
        var moc = new MOC(options);
        moc.dataFromURL(url, successCallback);

        return moc;
    };
    
    // @oldAPI
    Aladin.prototype.createCatalogFromVOTable = function(url, options) {
        return A.catalogFromURL(url, options);
    };

    // API
    A.catalogFromURL = function(url, options, successCallback, useProxy) {
        var catalog = A.catalog(options);
        cds.Catalog.parseVOTable(url, function(sources) {
            catalog.addSources(sources);
            if (successCallback) {
                successCallback(sources);
            }
        }, catalog.maxNbSources, useProxy);
        return catalog;
    };

    // API
    // @param target: can be either a string representing a position or an object name, or can be an object with keys 'ra' and 'dec' (values being in decimal degrees)
    A.catalogFromSimbad = function(target, radius, options, successCallback) {
        options = options || {};
        if (! ('name' in options)) {
            options['name'] = 'Simbad';
        }
        var url = URLBuilder.buildSimbadCSURL(target, radius);
        return A.catalogFromURL(url, options, successCallback, false);
    };
     
    // API
    A.catalogFromNED = function(target, radius, options, successCallback) {
        options = options || {};
        if (! ('name' in options)) {
            options['name'] = 'NED';
        }
        var url;
        if (target && (typeof target  === "object")) {
            if ('ra' in target && 'dec' in target) {
                url = URLBuilder.buildNEDPositionCSURL(target.ra, target.dec, radius);
            }
        }
        else {
    	    var isObjectName = /[a-zA-Z]/.test(target);
            if (isObjectName)  {
                url = URLBuilder.buildNEDObjectCSURL(target, radius);
            }
            else {
                var coo = new Coo();
                coo.parse(target);
                url = URLBuilder.buildNEDPositionCSURL(coo.lon, coo.lat, radius);
            }
        }

        return A.catalogFromURL(url, options, successCallback);
    };

    A.catalogFromVizieR = function(vizCatId, target, radius, options, successCallback) {
        options = options || {};
        if (! ('name' in options)) {
            options['name'] = 'VizieR:' + vizCatId;
        }
        var url = URLBuilder.buildVizieRCSURL(vizCatId, target, radius);
        return A.catalogFromURL(url, options, successCallback, false);
    };

     
     // API
     Aladin.prototype.on = function(what, myFunction) {
         if (what==='select') {
             this.selectFunction = myFunction;
         }
         else if (what=='objectClicked') {
            this.objClickedFunction = myFunction;
         }
         else if (what=='objectHovered') {
            this.objHoveredFunction = myFunction;
         }
     };
     
     Aladin.prototype.select = function() {
         this.fire('selectstart');
     };
     
     Aladin.prototype.fire = function(what, params) {
         if (what==='selectstart') {
             this.view.setMode(View.SELECT);
         }
         else if (what==='selectend') {
             this.view.setMode(View.PAN);
             if (this.selectFunction) {
                 this.selectFunction(params);
             }
         }
     };
     
     Aladin.prototype.hideBoxes = function() {
         if (this.boxes) {
             for (var k=0; k<this.boxes.length; k++) {
                 this.boxes[k].hide();
             }
         }
     };
     
     // ?
     Aladin.prototype.updateCM = function() {
         
     };
     
     // TODO : LayerBox should be a separate object
     Aladin.prototype.showLayerBox = function() {
         var self = this;
         
         // first, update
         var layerBox = $(this.aladinDiv).find('.aladin-layerBox');
         layerBox.empty();
         layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
                 '<div style="clear: both;"></div>' +
                 '<div class="aladin-label">Base image layer</div>' +
                 '<select class="aladin-surveySelection"></select>' +
                 '<div class="aladin-cmap">Color map:' +
                 '<div><select class="aladin-cmSelection"></select><button class="aladin-btn aladin-btn-small aladin-reverseCm" type="button">Reverse</button></div></div>' +
                 '<div class="aladin-box-separator"></div>' +
                 '<div class="aladin-label">Overlay layers</div>');
         
         var cmDiv = layerBox.find('.aladin-cmap');
         
         // fill color maps options
         var cmSelect = layerBox.find('.aladin-cmSelection');
         for (var k=0; k<ColorMap.MAPS_NAMES.length; k++) {
             cmSelect.append($("<option />").text(ColorMap.MAPS_NAMES[k]));
         }
         cmSelect.val(self.getBaseImageLayer().getColorMap().map);

         
         // loop over catalogs
         var cats = this.view.catalogs;
         var str = '<ul>';
         for (var k=cats.length-1; k>=0; k--) {
             var name = cats[k].name;
             var checked = '';
             if (cats[k].isShowing) {
                 checked = 'checked="checked"';
             }
             var nbSources = cats[k].getSources().length;
             var title = nbSources + ' source' + ( nbSources>1 ? 's' : '');
             var color = cats[k].color;
             var rgbColor = $('<div></div>').css('color', color).css('color'); // trick to retrieve the color as 'rgb(,,)'
             var labelColor = Color.getLabelColorForBackground(rgbColor);
             str += '<li><div class="aladin-layerIcon" style="background: ' + color + ';"></div><input type="checkbox" ' + checked + ' id="aladin_lite_' + name + '"></input><label for="aladin_lite_' + name + '" class="aladin-layer-label" style="background: ' + color + '; color:' + labelColor + ';" title="' + title + '">' + name + '</label></li>';
         }
         str += '</ul>';
         layerBox.append(str);
         
         layerBox.append('<div class="aladin-blank-separator"></div>');
         
         // gestion du réticule
         var checked = '';
         if (this.view.displayReticle) {
             checked = 'checked="checked"';
         }
         var reticleCb = $('<input type="checkbox" ' + checked + ' id="displayReticle" />');
         layerBox.append(reticleCb).append('<label for="displayReticle">Reticle</label><br/>');
         reticleCb.change(function() {
             self.showReticle($(this).is(':checked'));
         });
         
         // Gestion grille Healpix
         checked = '';
         if (this.view.displayHpxGrid) {
             checked = 'checked="checked"';
         }
         var hpxGridCb = $('<input type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
         layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
         hpxGridCb.change(function() {
             self.showHealpixGrid($(this).is(':checked'));
         });
         
         
         layerBox.append('<div class="aladin-box-separator"></div>' +
              '<div class="aladin-label">Tools</div>');
         var exportBtn = $('<button class="aladin-btn" type="button">Export view as PNG</button>');
         layerBox.append(exportBtn);
         exportBtn.click(function() {
             self.exportAsPNG();
         });
                 
                 /*
                 '<div class="aladin-box-separator"></div>' +
                 '<div class="aladin-label">Projection</div>' +
                 '<select id="projectionChoice"><option>SINUS</option><option>AITOFF</option></select><br/>'
                 */

         layerBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
         
         // update list of surveys
         this.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
         var surveySelection = $(this.aladinDiv).find('.aladin-surveySelection');
         surveySelection.change(function() {
             var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
             self.setImageSurvey(survey.id, function() {
                 var baseImgLayer = self.getBaseImageLayer();
                 
                 if (baseImgLayer.useCors) {
                     // update color map list with current value color map
                     cmSelect.val(baseImgLayer.getColorMap().map);
                     cmDiv.show();
                     
                     exportBtn.show();
                 }
                 else {
                     cmDiv.hide();
                     
                     exportBtn.hide();
                 }
             });

             
             
         });
         
         //// COLOR MAP management ////////////////////////////////////////////
         // update color map
         cmDiv.find('.aladin-cmSelection').change(function() {
             var cmName = $(this).find(':selected').val();
             self.getBaseImageLayer().getColorMap().update(cmName);
         });
         
         // reverse color map
         cmDiv.find('.aladin-reverseCm').click(function() {
             self.getBaseImageLayer().getColorMap().reverse(); 
         });
         if (this.getBaseImageLayer().useCors) {
             cmDiv.show();
             exportBtn.show();
         }
         else {
             cmDiv.hide();
             exportBtn.hide();
         }
         layerBox.find('.aladin-reverseCm').parent().attr('disabled', true);
         //////////////////////////////////////////////////////////////////////
         
         
         // handler to hide/show overlays
         $(this.aladinDiv).find('.aladin-layerBox ul input').change(function() {
             var catName = ($(this).attr('id').substr(12));
             var cat = self.layerByName(catName);
             if ($(this).is(':checked')) {
                 cat.show();
             }
             else {
                 cat.hide();
             }
         });
         
         // finally show
         layerBox.show();
         
     };
     
     Aladin.prototype.layerByName = function(name) {
         var c = this.view.catalogs;
         for (var k=0; k<this.view.catalogs.length; k++) {
             if (name==c[k].name) {
                 return c[k];
             }
         }
         return null;
     };
     
     // TODO : integrate somehow into API ?
     Aladin.prototype.exportAsPNG = function(imgFormat) {
         window.open(this.getViewDataURL(), "Aladin Lite snapshot");
     };

    /**
     * Return the current view as a data URL (base64-formatted string)
     * Parameters:
     * - imgFormat (optional): 'image/png' or 'image/jpeg'
     *
     * @API
    */
    Aladin.prototype.getViewDataURL = function(imgFormat) {
        return this.view.getCanvasDataURL(imgFormat);
    }
     
     /** limit FOV range
      * @API
      * @param minFOV in degrees when zoom in at max
      * @param maxFOV in degreen when zoom out at max
     */
     Aladin.prototype.setFOVRange = function(minFOV, maxFOV) {
         if (minFOV>maxFOV) {
             var tmp = minFOV;
             minFOV = maxFOV;
             maxFOV = tmp;
         }
         
         this.view.minFOV = minFOV;
         this.view.maxFOV = maxFOV;
         
     };
     
     /**
      * Transform pixel coordinates to world coordinates
      * 
      * Origin (0,0) of pixel coordinates is at top left corner of Aladin Lite view
      * 
      * @API
      * 
      * @param x
      * @param y
      * 
      * @return a [ra, dec] array with world coordinates in degrees
      * 
      */
     Aladin.prototype.pix2world = function(x, y) {
         var xy = AladinUtils.viewToXy(x, y, this.view.width, this.view.height, this.view.largestDim, this.view.zoomFactor);
         
         var radec = this.view.projection.unproject(xy.x, xy.y);
         
         var res;
         if (this.view.cooFrame==CooFrameEnum.GAL) {
             res = CooConversion.GalacticToJ2000([radec.ra, radec.dec]);
         }
         else {
             res =  [radec.ra, radec.dec];
         }
             
         return res;
     };
     
     /**
      * Transform world coordinates to pixel coordinates in the view
      * 
      * @API
      * 
      * @param ra  
      * @param dec
      * 
      * @return a [x, y] array with pixel coordinates in the view. Returns null if the projection failed somehow
      *   
      */
     Aladin.prototype.world2pix = function(ra, dec) {
         var xy;
         if (this.view.cooFrame==CooFrameEnum.GAL) {
             var lonlat = CooConversion.J2000ToGalactic([ra, dec]);
             xy = this.view.projection.project(lonlat[0], lonlat[1]);
         }
         else {
             xy = this.view.projection.project(ra, dec);
         }
         if (xy) {
             var xyview = AladinUtils.xyToView(xy.X, xy.Y, this.view.width, this.view.height, this.view.largestDim, this.view.zoomFactor);
             return [xyview.vx, xyview.vy];
         }
         else {
             return null;
         }
     };
     
     /**
      * 
      * @API
      * 
      * @param ra  
      * @param nbSteps the number of points to return along each side (the total number of points returned is 4*nbSteps)
      * 
      * @return set of points along the current FoV with the following format: [[ra1, dec1], [ra2, dec2], ..., [ra_n, dec_n]]
      *   
      */
     Aladin.prototype.getFovCorners = function(nbSteps) {
         // default value: 1
         if (!nbSteps || nbSteps<1) {
             nbSteps = 1;
         }
         
         var points = [];
         var x1, y1, x2, y2;
         for (var k=0; k<4; k++) {
             x1 = (k==0 || k==3) ? 0 : this.view.width-1;
             y1 = (k<2) ? 0 : this.view.height-1;
             x2 = (k<2) ? this.view.width-1 : 0;
             y2 = (k==1 || k==2) ? this.view.height-1 :0;
             
             for (var step=0; step<nbSteps; step++) {
                 points.push(this.pix2world(x1 + step/nbSteps * (x2-x1), y1 + step/nbSteps * (y2-y1)));
             }
         }
         
         return points;
         
     };
     
     /**
      * @API
      * 
      * @return the current FoV size in degrees as a 2-elements array
      */
     Aladin.prototype.getFov = function() {
         var fovX = this.view.fov;
         var s = this.getSize();
         var fovY = s[1] / s[0] * fovX;
         // TODO : take into account AITOFF projection where fov can be larger than 180
         fovX = Math.min(fovX, 180);
         fovY = Math.min(fovY, 180);
         
         return [fovX, fovY];
     };
     
     /**
      * @API
      * 
      * @return the size in pixels of the Aladin Lite view
      */
     Aladin.prototype.getSize = function() {
         return [this.view.width, this.view.height];
     };
     
     /**
      * @API
      * 
      * @return the jQuery object representing the DIV element where the Aladin Lite instance lies
      */
     Aladin.prototype.getParentDiv = function() {
         return $(this.aladinDiv);
     };
    
	return Aladin;
})();

//// New API ////
// For developers using Aladin lite: all objects should be created through the API, 
// rather than creating directly the corresponding JS objects
// This facade allows for more flexibility as objects can be updated/renamed harmlessly

//@API
A.aladin = function(divSelector, options) {
  return new Aladin($(divSelector)[0], options);
};

//@API
// TODO : lecture de properties
A.imageLayer = function(id, name, rootUrl, options) {
    return new HpxImageSurvey(id, name, rootUrl, null, null, options);
};

// @API
A.source = function(ra, dec, data, options) {
    return new cds.Source(ra, dec, data, options);
};

// @API
A.marker = function(ra, dec, options, data) {
    options = options || {};
    options['marker'] = true;
    return A.source(ra, dec, data, options);
};

// @API
A.polygon = function(raDecArray) {
    var l = raDecArray.length;
    if (l>0) {
        // close the polygon if needed
        if (raDecArray[0][0]!=raDecArray[l-1][0] || raDecArray[0][1]!=raDecArray[l-1][1]) {
            raDecArray.push([raDecArray[0][0], raDecArray[0][1]]);
        }
    }
    return new Footprint(raDecArray);
};

//@API
A.polyline = function(raDecArray, options) {
    return new Polyline(raDecArray, options);
};


// @API
A.circle = function(ra, dec, radiusDeg, options) {
    return new Circle([ra, dec], radiusDeg, options);
};

// @API
A.graphicOverlay = function(options) {
    return new Overlay(options);
};

// @API
A.catalog = function(options) {
    return new cds.Catalog(options);
};

// @API
/*
 * return a URL allowing to share the current view
 */
Aladin.prototype.getShareURL = function() {
    var radec = this.getRaDec();
    var coo = new Coo();
    coo.prec = 7;
    coo.lon = radec[0];
    coo.lat = radec[1];
    return 'http://aladin.u-strasbg.fr/AladinLite/?target=' + encodeURIComponent(coo.format('s')) +
           '&fov=' + this.getFov()[0].toFixed(2) + '&survey=' + encodeURIComponent(this.getBaseImageLayer().id);
};

// @API
/*
 * return, as a string, the HTML embed code
 */
Aladin.prototype.getEmbedCode = function() {
    var radec = this.getRaDec();
    var coo = new Coo();
    coo.prec = 7;
    coo.lon = radec[0];
    coo.lat = radec[1];

    var survey = this.getBaseImageLayer().id;
    var fov = this.getFov()[0];
    var s = '';
    s += '<link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.css" />\n';
    s += '<script type="text/javascript" src="http://code.jquery.com/jquery-1.9.1.min.js" charset="utf-8"></script>\n';
    s += '<div id="aladin-lite-div" style="width:400px;height:400px;"></div>\n';
    s += '<script type="text/javascript" src="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.js" charset="utf-8"></script>\n';
    s += '<script type="text/javascript">\n';
    s += 'var aladin = A.aladin("#aladin-lite-div", {survey: "' + survey + 'P/DSS2/color", fov: ' + fov.toFixed(2) + ', target: "' + coo.format('s') + '"});\n';
    s += '</script>';
    return s;
};

// @API
/*
 * Creates remotely a HiPS from a FITS image URL and displays it
 */
Aladin.prototype.displayFITS = function(url, options) {
    options = options || {};
    var data = {url: url};
    if (options.color) {
        data.color = true;
    }
    if (options.outputFormat) {
        data.format = options.outputFormat;
    }
    if (options.order) {
        data.order = options.order;
    }
    if (options.nocache) {
        data.nocache = options.nocache;
    }
    $.ajax({
        url: 'http://alasky.u-strasbg.fr/cgi/fits2HiPS',
        data: data,
        method: 'GET',
        dataType: 'json',
        success: function(response) {
            if (response.status!='success') {
                alert('An error occured: ' + response.message);
                return;
            }
            var label = options.label || "FITS image"; 
            aladin.setOverlayImageLayer(aladin.createImageSurvey(label, label, response.data.url, "equatorial", response.data.meta.max_norder, {imgFormat: 'png'}));
            aladin.setFoV(response.data.meta.fov);
            aladin.gotoRaDec(response.data.meta.ra, response.data.meta.dec);
            var transparency = (options && options.transparency) || 1.0;
            aladin.getOverlayImageLayer().setAlpha(transparency);

        }
    });

};

// @API
/*
 * Creates remotely a HiPS from a JPEG or PNG image with astrometry info
 * and display it
 */
Aladin.prototype.displayJPG = Aladin.prototype.displayPNG = function(url, options) {
    options = options || {};
    options.color = true;
    options.label = "JPG/PNG image";
    options.outputFormat = 'png';
    this.displayFITS(url, options);
};



// conservé pour compatibilité avec existant
// @oldAPI
if ($) {
    $.aladin = A.aladin;
}

// TODO: callback function onAladinLiteReady
