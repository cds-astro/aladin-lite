/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Aladin.js (main class)
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Aladin = (function() {
	var Aladin = function(aladinDiv, options) {
		HealpixCache.init();
		options = options || {};
		
//		canvas.imageSmoothingEnabled = false;
//		canvas.webkitImageSmoothingEnabled = false;
//		canvas.mozImageSmoothingEnabled = false;

		this.aladinDiv = aladinDiv;

		// parent div
		$(aladinDiv).css("position", "relative");
		// div where we write the position
		var locationDiv = $('<div id="locationDiv" style="z-index: 1000;position:absolute; padding: 2px 4px 2px 4px;  font-size: 13px; background-color: rgba(255, 255, 255, 0.5)"></div>').appendTo(aladinDiv);

		// canvas to draw the images
		var imageCanvas = $("<canvas style='position: absolute; left: 0; top: 0; z-index: 1;'></canvas>").appendTo(aladinDiv)[0];
		// canvas to draw the catalogs
		var catalogCanvas = $("<canvas style='position: absolute; left: 0; top: 0; z-index: 2;'></canvas>").appendTo(aladinDiv)[0];
		
		// Aladin logo
		$("<img src='http://aladin.u-strasbg.fr/aladin_large.gif' width='50' height='29' style='position: absolute; bottom: 5px; right: 10px; z-index: 3;' />").appendTo(aladinDiv);
		
		// control panel
		if ((options.showControl) || options.showControl==undefined) {
			$('<button id="showControlBox" style="z-index: 20; position: absolute; right: 0px; top: 0px;">Controls</button>').appendTo(aladinDiv);
			$('<div id="controlBox" style="display: none;background: white;position: absolute; right: 0px; top: 0px; border: 2px solid; padding: 4px 10px 10px 10px; z-index: 30; ">' +
            '<button id="closeControlBox" style="float: right;">Close</button>' +
            '<div style="clear: both;">' +
	        '<form id="targetForm" style="clear; both;">Target: <input type="text" id="target" /></form>' +
	        'Frame: <select id="frameChoice"><option>J2000</option><option selected="selected">GAL</option></select><br/>' +
	        'Projection: <select id="projectionChoice"><option>SINUS</option><option>AITOFF</option></select><br/>' +
	        '<input type="checkbox" id="displayHpxGrid"/><label for="displayHpxGrid">Show HEALPix grid</label><br/>' +
	        '<input type="checkbox" id="displaySurvey" checked="checked" /><label for="displaySurvey">Show survey</label><br/>' +
	        '<input type="checkbox" id="displayCatalog" /><label for="displayCatalog">Show catalog</label><br/>' +
	        '<select id="surveySelection"></select><br/>' +
	        'Zoom:<br/>' +
	        '<button id="zoomPlus" style="width: 30%"><b> + </b></button> <button id="zoomMinus"  style="width: 30%"><b> - </b></button>' +
	        '</div></div>').appendTo(aladinDiv);
			
			$('#showControlBox').click(function() {$('#controlBox').show();});
			$('#closeControlBox').click(function() {$('#controlBox').hide();});

		}
		
		var surveys = HpxImageSurvey.getAvailableSurveys();
        for (var i=0; i<surveys.length; i++) {
        	$('#surveySelection').append($("<option />").val(surveys[i].name).text(surveys[i].name));
        };
        
        

		
		
		
		
		
		
		
		
		
		var location = new Location(locationDiv);
        
		var frame = null;
		if (options && options.frame) {
			frame = options.frame;
		}
		this.view = new View(this.aladinDiv, imageCanvas, catalogCanvas, location, frame);

		if (options && options.zoomLevel) {
            this.view.setZoomLevel(options.zoomLevel);
        }
		
		if (options && options.target) {
			this.gotoObject(options.target);
		}
		else {
			this.gotoPosition(266.416833, -29.007806); // point to galactic center
		}
		var surveyInfo = null;
		if (options && options.survey) {
			surveyInfo = HpxImageSurvey.getSurveyInfoFromName(options.survey);
		}
		if (! surveyInfo) {
			surveyInfo = surveys[0];
		}
		this.setImageSurvey(new HpxImageSurvey(surveyInfo.name, surveyInfo.url, surveyInfo.frame, surveyInfo.maxOrder));
		
		
	    
    	var aladin = this;
    	$('#frameChoice').change(function() {
    		aladin.setFrame($(this).val());
    	});
    	$('#projectionChoice').change(function() {
    		aladin.setProjection($(this).val());
    	});
        $('#displayHpxGrid').change(function() {
            aladin.showHealpixGrid($(this).is(':checked'));
        });
        $('#displaySurvey').change(function() {
            aladin.showSurvey($(this).is(':checked'));
        });
        $('#displayCatalog').change(function() {
            aladin.showCatalog($(this).is(':checked'));
        });

        $('#targetForm').submit(function() {
            aladin.gotoObject($('#target').val());
            return false;
        });
        
        $('#zoomPlus').click(function() {
        	aladin.increaseZoom();
        });
        
        $('#zoomMinus').click(function() {
            aladin.decreaseZoom();
        });
        
        $('#surveySelection').change(function() {
            var survey = surveys[$(this)[0].selectedIndex];
        	aladin.setImageSurvey(new HpxImageSurvey(survey.name, survey.url, survey.frame, survey.maxOrder));
        });
        
        

		
	};
	
	
	// TODO : à écrire !!
	Aladin.prototype.setFov = function(fovDegrees) {
		
	};
	
    Aladin.prototype.setFrame = function(frameName) {
        if (! frameName) {
            return;
        }
        frameName = frameName.toLowerCase();
        if (frameName.indexOf('j2000')==0) {
            this.view.changeFrame(CooFrameEnum.J2000);
        }
        else if (frameName.indexOf('gal')==0) {
            this.view.changeFrame(CooFrameEnum.GAL);
        }
    }

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
	}
    
    // point view to a given object (resolved by Sesame) or position
    Aladin.prototype.gotoObject = function(targetName) {
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
	        				   if (data.Target && data.Target.Resolver && data.Target.Resolver.jpos) {
	        					   var ra = data.Target.Resolver.jradeg;
	        					   var dec = data.Target.Resolver.jdedeg;
	        					   self.view.pointTo(ra, dec);
	        				   }
	        				   else {
	                                if (console) console.log(data);
	        				   }
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
	                       });
    	}
    };
    
    Aladin.prototype.gotoPosition = function(ra, dec) {
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

    
    Aladin.prototype.setImageSurvey = function(imageSurvey) {
    	this.view.setImageSurvey(imageSurvey);
    };
    
    Aladin.prototype.increaseZoom = function() {
    	this.view.setZoomLevel(this.view.zoomLevel+5);
    };
    
    Aladin.prototype.decreaseZoom = function() {
    	this.view.setZoomLevel(this.view.zoomLevel-5);
    };
	
	return Aladin;
})();
