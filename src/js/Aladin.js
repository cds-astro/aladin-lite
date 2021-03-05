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

import { View } from "./View.js";
import { MOC } from "./MOC.js";
import { Utils } from "./Utils.js";
import { Overlay } from "./Overlay.js";
import { Footprint } from "./Footprint.js";
import { Circle } from "./Circle.js";
import { Ellipse } from "./Ellipse.js";
import { Polyline } from "./Polyline.js";
import { AladinUtils } from "./AladinUtils.js";
import { Logger } from "./Logger.js";
import { Catalog } from "./Catalog.js";
import { ProgressiveCat } from "./ProgressiveCat.js";
import { Sesame } from "./Sesame.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { MeasurementTable } from "./MeasurementTable.js";
import { Location } from "./Location.js";
import { Source } from "./Source.js";
import { HpxImageSurvey } from "./HpxImageSurvey.js";
import { Coo } from "./libs/astro/coo.js";
import { CooConversion } from "./CooConversion.js";
import { Color } from "./Color.js";
import { ColorMap } from "./ColorMap.js";
import { URLBuilder } from "./URLBuilder.js";
import { HiPSDefinition } from "./HiPSDefinition.js";
import { DiscoveryTree } from "./DiscoveryTree.js";
import { ImageSurveyLayer } from "./ImageSurveyLayer.js";

export let Aladin = (function () {

    // Constructor
    var Aladin = function (aladinDiv, requestedOptions) {
        // check that aladinDiv exists, stop immediately otherwise
        if ($(aladinDiv).length == 0) {
            return;
        }
        this.webglAPI = null;

        var self = this;

        // if not options was set, try to retrieve them from the query string
        if (requestedOptions === undefined) {
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
            if (Aladin.DEFAULT_OPTIONS[key] === undefined) {
                options[key] = requestedOptions[key];
            }
        }

        this.options = options;

        $("<style type='text/css'> .aladin-reticleColor { color: " + this.options.reticleColor + "; font-weight:bold;} </style>").appendTo(aladinDiv);



        this.aladinDiv = aladinDiv;

        this.reduceDeformations = true;

        // parent div
        $(aladinDiv).addClass("aladin-container");


        var cooFrame = CooFrameEnum.fromString(options.cooFrame, CooFrameEnum.J2000);
        // locationDiv is the div where we write the position
        var locationDiv = $('<div class="aladin-location">'
            + (options.showFrame ? '<select class="aladin-frameChoice"><option value="' + CooFrameEnum.J2000.label + '" '
                + (cooFrame == CooFrameEnum.J2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.J2000d.label + '" '
                + (cooFrame == CooFrameEnum.J2000d ? 'selected="selected"' : '') + '>J2000d</option><option value="' + CooFrameEnum.GAL.label + '" '
                + (cooFrame == CooFrameEnum.GAL ? 'selected="selected"' : '') + '>GAL</option></select>' : '')
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
        this.fullScreenBtn.click(function () {
            self.toggleFullscreen(self.options.realFullscreen);
        });
        // react to fullscreenchange event to restore initial width/height (if user pressed ESC to go back from full screen)
        $(document).on('fullscreenchange webkitfullscreenchange mozfullscreenchange MSFullscreenChange', function (e) {
            var fullscreenElt = document.fullscreenElement || document.webkitFullscreenElement || document.mozFullScreenElement || document.msFullscreenElement;
            if (fullscreenElt === null || fullscreenElt === undefined) {
                self.fullScreenBtn.removeClass('aladin-restore');
                self.fullScreenBtn.addClass('aladin-maximize');
                self.fullScreenBtn.attr('title', 'Full screen');
                $(self.aladinDiv).removeClass('aladin-fullscreen');

                var fullScreenToggledFn = self.callbacksByEventName['fullScreenToggled'];
                var isInFullscreen = self.fullScreenBtn.hasClass('aladin-restore');
                (typeof fullScreenToggledFn === 'function') && fullScreenToggledFn(isInFullscreen);
            }
        });





        // Aladin logo
        $("<div class='aladin-logo-container'><a href='https://aladin.unistra.fr/' title='Powered by Aladin Lite' target='_blank'><div class='aladin-logo'></div></a></div>").appendTo(aladinDiv);


        // we store the boxes
        this.boxes = [];

        // measurement table
        this.measurementTable = new MeasurementTable(aladinDiv);



        var location = new Location(locationDiv.find('.aladin-location-text'));

        // set different options
        this.view = new View(this, location, fovDiv, cooFrame, options.fov);
        this.view.setShowGrid(options.showCooGrid);

        // retrieve available surveys
        // TODO: replace call with MocServer
        /*$.ajax({
            url: "//aladin.unistra.fr/java/nph-aladin.pl",
            data: { "frame": "aladinLiteDic" },
            method: 'GET',
            dataType: 'jsonp', // could this be repaced by json ??
            success: function (data) {
                var map = {};
                for (var k = 0; k < data.length; k++) {
                    map[data[k].id] = true;
                }
                // retrieve existing surveys
                for (var k = 0; k < HpxImageSurvey.SURVEYS.length; k++) {
                    if (!map[HpxImageSurvey.SURVEYS[k].id]) {
                        data.push(HpxImageSurvey.SURVEYS[k]);
                    }
                }
                HpxImageSurvey.SURVEYS = data;
                self.view.setUnknownSurveyIfNeeded();
            },
            error: function () {
            }
        });*/

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
            d.click(function () { self.hideBoxes(); self.showLayerBox(); return false; });

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
            input.on("paste keydown", function () {
                $(this).removeClass('aladin-unknownObject'); // remove red border
            });

            // TODO : classe GotoBox
            d.click(function () {
                self.hideBoxes();
                input.val('');
                input.removeClass('aladin-unknownObject');
                gotoBox.show();
                input.focus();


                return false;
            });
            gotoBox.find('.aladin-closeBtn').click(function () { self.hideBoxes(); return false; });
        }

        // simbad pointer tool
        if (options.showSimbadPointerControl) {
            var d = $('<div class="aladin-simbadPointerControl-container" title="SIMBAD pointer"><div class="aladin-simbadPointerControl"></div></div>');
            d.appendTo(aladinDiv);

            d.click(function () {
                self.view.setMode(View.TOOL_SIMBAD_POINTER);
            });
        }

        // share control panel
        if (options.showShareControl) {
            var d = $('<div class="aladin-shareControl-container" title="Get link for current view"><div class="aladin-shareControl"></div></div>');
            d.appendTo(aladinDiv);

            var shareBox =
                $('<div class="aladin-box aladin-shareBox">' +
                    '<a class="aladin-closeBtn">&times;</a>' +
                    '<div style="clear: both;"></div>' +
                    'Link to previewer: <span class="info"></span>' +
                    '<input type="text" class="aladin-shareInput" />' +
                    '</div>');
            shareBox.appendTo(aladinDiv);
            this.boxes.push(shareBox);


            // TODO : classe GotoBox, GenericBox
            d.click(function () {
                self.hideBoxes();
                shareBox.show();
                var url = self.getShareURL();
                shareBox.find('.aladin-shareInput').val(url).select();
                document.execCommand('copy');

                return false;
            });
            shareBox.find('.aladin-closeBtn').click(function () { self.hideBoxes(); return false; });
        }


        this.gotoObject(options.target, undefined, {forceAnimation: false});

        if (options.log) {
            var params = requestedOptions;
            params['version'] = Aladin.VERSION;
            Logger.log("startup", params);
        }

        this.showReticle(options.showReticle);

        if (options.catalogUrls) {
            for (var k = 0, len = options.catalogUrls.length; k < len; k++) {
                this.createCatalogFromVOTable(options.catalogUrls[k]);
            }
        }
        
        /*let webglAPI = await import('../render/pkg/');
        console.log('webgl imported');
        let shaders = await loadShaders(webglAPI);
        console.log(shaders);
    
        // Start our Rust application. You can find `WebClient` in `src/lib.rs`
        let resources = {
            'kernel': kernel,
        };
        Aladin.wasmLibs.webglAPI = new webglAPI.WebClient(shaders, resources);
        let webgl = Aladin.wasmLibs.webglAPI;
        webgl.resize(500, 400);*/

        /*let imageSurveyInfo = HpxImageSurvey.getSurveyInfoFromId(options.survey);
        console.log('image survey, ', imageSurveyInfo)
        webgl.setImageSurvey(imageSurveyInfo);*/
        // Add the image layers
        // For that we check the survey key of options
        // It can be given as a single string or an array of strings
        // for multiple blending surveys
        if (options.survey) {
            (async () => {
                if (typeof options.survey === Array) {
                    let i = 0;
                    options.survey.forEach(async (rootUrlOrId) => {
                        const survey = await Aladin.createImageSurvey(rootUrlOrId);
                        if (i == 0) {
                            this.setImageSurvey(survey, "base");
                        } else {
                            this.addImageSurvey(survey, "base");
                        }
                        i++;
                    });
                } else {
                    const survey = await Aladin.createImageSurvey(options.survey, "base");
                    this.setImageSurvey(survey, "base");
                }
            })();

        }
        this.view.showCatalog(options.showCatalog);


        var aladin = this;
        $(aladinDiv).find('.aladin-frameChoice').change(function () {
            aladin.setFrame($(this).val());
        });


        $(aladinDiv).find('.aladin-target-form').submit(function () {
            aladin.gotoObject($(this).find('input').val(), function () {
                $(aladinDiv).find('.aladin-target-form input').addClass('aladin-unknownObject');
            });
            return false;
        });

        var zoomPlus = $(aladinDiv).find('.zoomPlus');
        zoomPlus.click(function () {
            aladin.increaseZoom();
            return false;
        });
        zoomPlus.bind('mousedown', function (e) {
            e.preventDefault(); // to prevent text selection
        });

        var zoomMinus = $(aladinDiv).find('.zoomMinus');
        zoomMinus.click(function () {
            aladin.decreaseZoom();
            return false;
        });
        zoomMinus.bind('mousedown', function (e) {
            e.preventDefault(); // to prevent text selection
        });



        this.callbacksByEventName = {}; // we store the callback functions (on 'zoomChanged', 'positionChanged', ...) here

        // initialize the Vue components
        if (typeof Vue != "undefined") {
            //Vue.component("layers", Layers)
            this.discoverytree = new DiscoveryTree(this);
        }

        this.view.redraw();

        // go to full screen ?
        if (options.fullScreen) {
            // strange behaviour to wait for a sec
            window.setTimeout(function () { self.toggleFullscreen(self.options.realFullscreen); }, 10);
        }
    };

    /**** CONSTANTS ****/
    Aladin.VERSION = "{ALADIN-LITE-VERSION-NUMBER}"; // will be filled by the build.sh script

    Aladin.JSONP_PROXY = "https://alasky.unistra.fr/cgi/JSONProxy";
    //Aladin.JSONP_PROXY = "https://alaskybis.unistra.fr/cgi/JSONProxy";

    // access to WASM libraries
    Aladin.wasmLibs = {};
    Aladin.webglAPI = [];
    Aladin.DEFAULT_OPTIONS = {
        target: "0 +0",
        cooFrame: "J2000",
        survey: "P/DSS2/color",
        fov: 60,
        showReticle: true,
        showZoomControl: true,
        showFullscreenControl: true,
        showLayersControl: true,
        showGotoControl: true,
        showSimbadPointerControl: false,
        showShareControl: false,
        showCatalog: true, // TODO: still used ??
        showFrame: true,
        showCooGrid: false,
        fullScreen: false,
        reticleColor: "rgb(178, 50, 178)",
        reticleSize: 22,
        log: true,
        allowFullZoomout: false,
        realFullscreen: false,
        showAllskyRing: false,
        allskyRingColor: '#c8c8ff',
        allskyRingWidth: 8,
        pixelateCanvas: true
    };


    // realFullscreen: AL div expands not only to the size of its parent, but takes the whole available screen estate 
    Aladin.prototype.toggleFullscreen = function (realFullscreen) {
        realFullscreen = Boolean(realFullscreen);

        this.fullScreenBtn.toggleClass('aladin-maximize aladin-restore');
        var isInFullscreen = this.fullScreenBtn.hasClass('aladin-restore');
        this.fullScreenBtn.attr('title', isInFullscreen ? 'Restore original size' : 'Full screen');
        $(this.aladinDiv).toggleClass('aladin-fullscreen');

        if (realFullscreen) {
            // go to "real" full screen mode
            if (isInFullscreen) {
                var d = this.aladinDiv;

                if (d.requestFullscreen) {
                    d.requestFullscreen();
                }
                else if (d.webkitRequestFullscreen) {
                    d.webkitRequestFullscreen();
                }
                else if (d.mozRequestFullScreen) { // notice the difference in capitalization for Mozilla functions ...
                    d.mozRequestFullScreen();
                }
                else if (d.msRequestFullscreen) {
                    d.msRequestFullscreen();
                }
            }
            // exit from "real" full screen mode
            else {
                if (document.exitFullscreen) {
                    document.exitFullscreen();
                }
                else if (document.webkitExitFullscreen) {
                    document.webkitExitFullscreen();
                }
                else if (document.mozCancelFullScreen) {
                    document.mozCancelFullScreen();
                }
                else if (document.webkitExitFullscreen) {
                    document.webkitExitFullscreen();
                }
            }
        }

        this.view.fixLayoutDimensions();

        // force call to zoomChanged callback
        var fovChangedFn = this.callbacksByEventName['zoomChanged'];
        (typeof fovChangedFn === 'function') && fovChangedFn(this.view.fov);

        var fullScreenToggledFn = this.callbacksByEventName['fullScreenToggled'];
        (typeof fullScreenToggledFn === 'function') && fullScreenToggledFn(isInFullscreen);
    };

    Aladin.prototype.updateSurveysDropdownList = function (surveys) {
        surveys = surveys.sort(function (a, b) {
            if (!a.order) {
                return a.id > b.id;
            }
            return a.order && a.order > b.order ? 1 : -1;
        });
        var select = $(this.aladinDiv).find('.aladin-surveySelection');
        select.empty();
        for (var i = 0; i < surveys.length; i++) {
            var isCurSurvey = this.view.imageSurvey.id == surveys[i].id;
            select.append($("<option />").attr("selected", isCurSurvey).val(surveys[i].id).text(surveys[i].name));
        };
    };

    Aladin.prototype.setAngleRotation = function (theta) {
        this.view.setAngleRotation(theta)
    }

    Aladin.prototype.getOptionsFromQueryString = function () {
        var options = {};
        var requestedTarget = $.urlParam('target');
        if (requestedTarget) {
            options.target = requestedTarget;
        }
        var requestedFrame = $.urlParam('frame');
        if (requestedFrame && CooFrameEnum[requestedFrame]) {
            options.frame = requestedFrame;
        }
        var requestedSurveyId = $.urlParam('survey');
        if (requestedSurveyId && HpxImageSurvey.getSurveyInfoFromId(requestedSurveyId)) {
            options.survey = requestedSurveyId;
        }
        var requestedZoom = $.urlParam('zoom');
        if (requestedZoom && requestedZoom > 0 && requestedZoom < 180) {
            options.zoom = requestedZoom;
        }

        var requestedShowreticle = $.urlParam('showReticle');
        if (requestedShowreticle) {
            options.showReticle = requestedShowreticle.toLowerCase() == 'true';
        }

        var requestedCooFrame = $.urlParam('cooFrame');
        if (requestedCooFrame) {
            options.cooFrame = requestedCooFrame;
        }

        var requestedFullscreen = $.urlParam('fullScreen');
        if (requestedFullscreen !== undefined) {
            options.fullScreen = requestedFullscreen;
        }

        return options;
    };

    // @API
    Aladin.prototype.setFoV = Aladin.prototype.setFov = function (fovDegrees) {
        this.view.setZoom(fovDegrees);
    };

    // @API
    // (experimental) try to adjust the FoV to the given object name. Does nothing if object is not known from Simbad
    Aladin.prototype.adjustFovForObject = function (objectName) {
        var self = this;
        this.getFovForObject(objectName, function (fovDegrees) {
            self.setFoV(fovDegrees);
        });
    };


    Aladin.prototype.getFovForObject = function (objectName, callback) {
        var query = "SELECT galdim_majaxis, V FROM basic JOIN ident ON oid=ident.oidref JOIN allfluxes ON oid=allfluxes.oidref WHERE id='" + objectName + "'";
        var url = '//simbad.u-strasbg.fr/simbad/sim-tap/sync?query=' + encodeURIComponent(query) + '&request=doQuery&lang=adql&format=json&phase=run';

        var ajax = Utils.getAjaxObject(url, 'GET', 'json', false)
        ajax.done(function (result) {
            var defaultFov = 4 / 60; // 4 arcmin
            var fov = defaultFov;

            if ('data' in result && result.data.length > 0) {
                var galdimMajAxis = Utils.isNumber(result.data[0][0]) ? result.data[0][0] / 60.0 : null; // result gives galdim in arcmin
                var magV = Utils.isNumber(result.data[0][1]) ? result.data[0][1] : null;

                if (galdimMajAxis !== null) {
                    fov = 2 * galdimMajAxis;
                }
                else if (magV !== null) {
                    if (magV < 10) {
                        fov = 2 * Math.pow(2.0, (6 - magV / 2.0)) / 60;
                    }
                }
            }

            (typeof callback === 'function') && callback(fov);
        });
    };

    Aladin.prototype.setFrame = function (frameName) {
        if (!frameName) {
            return;
        }
        var newFrame = CooFrameEnum.fromString(frameName, CooFrameEnum.J2000);
        if (newFrame == this.view.cooFrame) {
            return;
        }

        this.view.changeFrame(newFrame);
        // màj select box
        $(this.aladinDiv).find('.aladin-frameChoice').val(newFrame.label);
    };

    Aladin.prototype.setProjection = function (projectionName) {
        if (!projectionName) {
            return;
        }
        projectionName = projectionName.toLowerCase();
        /*console.log('setProj', projectionName);

        let projectionOptionElt = document.getElementById(projectionName);
        console.log("jKJHKSDJHF")

        console.log(projectionOptionElt);
        if (projectionOptionElt) {
            console.log("jKJHKSDJHF")
            projectionOptionElt.selected = 'selected';
        }*/
        
        this.view.changeProjection(projectionName);
        //this.view.fov_limit = this.webglAPI.getMaxFieldOfView() * 180 / Math.PI;
    };

    /** point view to a given object (resolved by Sesame) or position
     * @api
     *
     * @param: target; object name or position
     * @callbackOptions: (optional) the object with key 'success' and/or 'error' containing the success and error callback functions.
     *
     */
    Aladin.prototype.gotoObject = function (targetName, callbackOptions, options) {
        let successCallback = undefined;
        let errorCallback   = undefined;
        if (typeof callbackOptions === 'object') {
            if (callbackOptions.hasOwnProperty('success')) {
                successCallback = callbackOptions.success;
            }
            if (callbackOptions.hasOwnProperty('error')) {
                errorCallback = callbackOptions.error;
            }
        }
        // this is for compatibility reason with the previous method signature which was function(targetName, errorCallback)
        else if (typeof callbackOptions === 'function') {
            errorCallback = callbackOptions;
        }


        var isObjectName = /[a-zA-Z]/.test(targetName);

        // try to parse as a position
        if (!isObjectName) {
            var coo = new Coo();

            coo.parse(targetName);
            var lonlat = [coo.lon, coo.lat];
            // Convert it to icrs if the coo system is galactic
            if (this.view.aladin.webglAPI.cooSystem() === Aladin.wasmLibs.webgl.GALCooSys()) {
                lonlat = this.view.aladin.webglAPI.Gal2J2000(coo.lon, coo.lat);
            }
            this.view.pointTo(lonlat[0], lonlat[1], options);

            (typeof successCallback === 'function') && successCallback(this.getRaDec());
        }
        // ask resolution by Sesame
        else {
            var self = this;
            Sesame.resolve(targetName,
                function (data) { // success callback
                    // Location given in icrs at J2000
                    var ra = data.Target.Resolver.jradeg;
                    var dec = data.Target.Resolver.jdedeg;

                    self.view.pointTo(ra, dec, options);

                    (typeof successCallback === 'function') && successCallback(self.getRaDec());
                },
                function (data) { // errror callback
                    if (console) {
                        console.log("Could not resolve object name " + targetName);
                        console.log(data);
                    }
                    (typeof errorCallback === 'function') && errorCallback();
                });
        }
    };



    /**
     * go to a given position, expressed in the current coordinate frame
     * 
     * @API
     */
    Aladin.prototype.gotoPosition = function (lon, lat) {
        var radec;
        // first, convert to J2000 if needed
        if (this.view.cooFrame == CooFrameEnum.GAL) {
            radec = CooConversion.GalacticToJ2000([lon, lat]);
        }
        else {
            radec = [lon, lat];
        }
        this.view.pointTo(radec[0], radec[1]);
    };


    var doAnimation = function (aladin) {
        var params = aladin.animationParams;
        if (params == null || !params['running']) {
            return;
        }
        var now = new Date().getTime();
        // this is the animation end: set the view to the end position, and call complete callback 
        if (now > params['end']) {
            aladin.gotoRaDec(params['raEnd'], params['decEnd']);

            if (params['complete']) {
                params['complete']();
            }

            return;
        }

        // compute current position
        var fraction = (now - params['start']) / (params['end'] - params['start']);
        var curPos = intermediatePoint(params['raStart'], params['decStart'], params['raEnd'], params['decEnd'], fraction);
        var curRa = curPos[0];
        var curDec = curPos[1];
        //var curRa =  params['raStart'] + (params['raEnd'] - params['raStart']) * (now-params['start']) / (params['end'] - params['start']);
        //var curDec = params['decStart'] + (params['decEnd'] - params['decStart']) * (now-params['start']) / (params['end'] - params['start']);

        aladin.gotoRaDec(curRa, curDec);

        setTimeout(function () { doAnimation(aladin); }, 50);

    };

    /*
     * Stop all animations that have been initiated  by animateToRaDec or by zoomToFoV
     * @API
     *
     */
    Aladin.prototype.stopAnimation = function () {
        if (this.zoomAnimationParams) {
            this.zoomAnimationParams['running'] = false;
        }
        if (this.animationParams) {
            this.animationParams['running'] = false;
        }
    }

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
    Aladin.prototype.animateToRaDec = function (ra, dec, duration, complete) {
        duration = duration || 5;

        this.animationParams = null;

        var animationParams = {};
        animationParams['start'] = new Date().getTime();
        animationParams['end'] = new Date().getTime() + 1000 * duration;
        var raDec = this.getRaDec();
        animationParams['raStart'] = raDec[0];
        animationParams['decStart'] = raDec[1];
        animationParams['raEnd'] = ra;
        animationParams['decEnd'] = dec;
        animationParams['complete'] = complete;
        animationParams['running'] = true;

        this.animationParams = animationParams;

        doAnimation(this);
    };

    var doZoomAnimation = function (aladin) {
        var params = aladin.zoomAnimationParams;
        if (params == null || !params['running']) {
            return;
        }
        var now = new Date().getTime();
        // this is the zoom animation end: set the view to the end fov, and call complete callback 
        if (now > params['end']) {
            aladin.setFoV(params['fovEnd']);

            if (params['complete']) {
                params['complete']();
            }

            return;
        }

        // compute current position
        var fraction = (now - params['start']) / (params['end'] - params['start']);
        var curFov = params['fovStart'] + (params['fovEnd'] - params['fovStart']) * Math.sqrt(fraction);

        aladin.setFoV(curFov);

        setTimeout(function () { doZoomAnimation(aladin); }, 50);

    };
    /*
     * zoom smoothly from the current FoV to the given new fov to the given ra, dec
     * 
     * the total duration (in seconds) of the animation can be given (otherwise set to 5 seconds by default)
     * 
     * complete: a function to call once the animation has completed
     * 
     * @API
     * 
     */
    Aladin.prototype.zoomToFoV = function (fov, duration, complete) {
        duration = duration || 5;

        this.zoomAnimationParams = null;

        var zoomAnimationParams = {};
        zoomAnimationParams['start'] = new Date().getTime();
        zoomAnimationParams['end'] = new Date().getTime() + 1000 * duration;
        var fovArray = this.getFov();
        zoomAnimationParams['fovStart'] = Math.max(fovArray[0], fovArray[1]);
        zoomAnimationParams['fovEnd'] = fov;
        zoomAnimationParams['complete'] = complete;
        zoomAnimationParams['running'] = true;

        this.zoomAnimationParams = zoomAnimationParams;
        doZoomAnimation(this);
    };



    /**
     *  Compute intermediate point between points (lng1, lat1) and (lng2, lat2)
     *  at distance fraction times the total distance (fraction between 0 and 1)
     *
     *  Return intermediate points in degrees
     *
     */
    function intermediatePoint(lng1, lat1, lng2, lat2, fraction) {
        function degToRad(d) {
            return d * Math.PI / 180;
        }
        function radToDeg(r) {
            return r * 180 / Math.PI;
        }
        var lat1 = degToRad(lat1);
        var lng1 = degToRad(lng1);
        var lat2 = degToRad(lat2);
        var lng2 = degToRad(lng2);
        var d = 2 * Math.asin(
            Math.sqrt(Math.pow((Math.sin((lat1 - lat2) / 2)),
                2) +
                Math.cos(lat1) * Math.cos(lat2) *
                Math.pow(Math.sin((lng1 - lng2) / 2), 2)));
        var A = Math.sin((1 - fraction) * d) / Math.sin(d);
        var B = Math.sin(fraction * d) / Math.sin(d);
        var x = A * Math.cos(lat1) * Math.cos(lng1) + B *
            Math.cos(lat2) * Math.cos(lng2);
        var y = A * Math.cos(lat1) * Math.sin(lng1) + B *
            Math.cos(lat2) * Math.sin(lng2);
        var z = A * Math.sin(lat1) + B * Math.sin(lat2);
        var lon = Math.atan2(y, x);
        var lat = Math.atan2(z, Math.sqrt(Math.pow(x, 2) +
            Math.pow(y, 2)));

        return [radToDeg(lon), radToDeg(lat)];
    };




    /**
     * get current [ra, dec] position of the center of the view
     * 
     * @API
     */
    Aladin.prototype.getRaDec = function () {
        /*if (this.view.cooFrame.system == CooFrameEnum.SYSTEMS.J2000) {
            return [this.view.viewCenter.lon, this.view.viewCenter.lat];
        }
        else {
            var radec = CooConversion.GalacticToJ2000([this.view.viewCenter.lon, this.view.viewCenter.lat]);
            return radec;

        }*/
        let radec = this.webglAPI.getCenter();
        return radec;
    };


    /**
     * point to a given position, expressed as a ra,dec coordinate
     * 
     * @API
     */
    Aladin.prototype.gotoRaDec = function (ra, dec) {
        this.view.pointTo(ra, dec);
    };

    Aladin.prototype.showHealpixGrid = function (show) {
        this.view.showHealpixGrid(show);
    };

    Aladin.prototype.showSurvey = function (show) {
        this.view.showSurvey(show);
    };
    Aladin.prototype.showCatalog = function (show) {
        this.view.showCatalog(show);
    };
    Aladin.prototype.showReticle = function (show) {
        this.view.showReticle(show);
        $('#displayReticle').attr('checked', show);
    };
    Aladin.prototype.removeLayers = function () {
        this.view.removeLayers();
    };

    // these 3 methods should be merged into a unique "add" method
    Aladin.prototype.addCatalog = function (catalog) {
        this.view.addCatalog(catalog);
    };
    Aladin.prototype.addOverlay = function (overlay) {
        this.view.addOverlay(overlay);
    };
    Aladin.prototype.addMOC = function (moc) {
        this.view.addMOC(moc);
    };
    /*Aladin.prototype.addImageSurveyLayer = function (layer) {
        console.log("add layer", layer)
        this.view.addImageSurveyLayer(layer)
    };*/

    // @api
    /*Aladin.prototype.getBaseImageLayers = function () {
        return this.view.imageSurvey;
    };*/
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
    // @api
    // @old

    Aladin.createImageSurvey = async function(rootUrlOrId) {
        const survey = await HpxImageSurvey.create(rootUrlOrId);
        return survey;
    }

    Aladin.prototype.setImageSurvey = function (survey, layer) {
        let layerName;
        if (layer) {
            layerName = layer;
        } else {
            layerName = "base";
        }

        this.view.setImageSurvey(survey, layerName);
    };

    Aladin.prototype.setImageSurveysLayer = function (surveys, layer) {
        let layerName;
        if (layer) {
            layerName = layer;
        } else {
            layerName = "base";
        }

        this.view.setImageSurveysLayer(surveys, layerName);
    };

    Aladin.prototype.removeImageSurveysLayer = function (layer) {
        let layerName;
        if (layer) {
            layerName = layer;
        } else {
            layerName = "base";
        }

        this.view.removeImageSurveysLayer(layerName);
    };

    Aladin.prototype.moveImageSurveysLayerForward = function (layer) {
        this.view.moveImageSurveysLayerForward(surveys, layerName);
    };

    Aladin.prototype.addImageSurvey = function (survey, layer) {
        let layerName;
        if (layer) {
            layerName = layer;
        } else {
            layerName = "base";
        }
        this.view.addImageSurvey(survey, layerName);
    };

    Aladin.prototype.setOpacityLayer = function(opacity, layer) {
        let layerName;
        if (layer) {
            layerName = layer;
        } else {
            layerName = "base";
        }
        this.webglAPI.setOpacityLayer(opacity, layer)
    }

    // @api
    Aladin.prototype.setBaseImageSurveysLayer = function (surveys) {
        this.view.setImageSurveysLayer(surveys, 'base');
    };
    Aladin.prototype.setBaseImageSurvey = function (survey) {
        this.view.setImageSurvey(survey, 'base');
    };
    /*
    // @api
    Aladin.prototype.getOverlayImageLayer = function () {
        return this.view.overlayImageSurvey;
    };
    // @api
    Aladin.prototype.setOverlayImageLayer = function (imageSurvey, callback) {
        this.view.setOverlayImageSurvey(imageSurvey, callback);
    };
    */

    Aladin.prototype.increaseZoom = function (step) {
        //if (!step) {
        //    step = 5;
        //}
        //this.view.setZoomLevel(this.view.zoomLevel + step);
        this.view.increaseZoom();
    };

    Aladin.prototype.decreaseZoom = function (step) {
        //if (!step) {
        //    step = 5;
        //}
        //this.view.setZoomLevel(this.view.zoomLevel - step);
        this.view.decreaseZoom();
    };


    Aladin.prototype.createProgressiveCatalog = function (url, frame, maxOrder, options) {
        return new ProgressiveCat(url, frame, maxOrder, options);
    };

    Aladin.prototype.createOverlay = function (options) {
        return new Overlay(options);
    };




    Aladin.AVAILABLE_CALLBACKS = ['select', 'objectClicked', 'objectHovered', 'footprintClicked', 'footprintHovered', 'positionChanged', 'zoomChanged', 'click', 'mouseMove', 'fullScreenToggled', 'catalogReady'];
    // API
    //
    // setting callbacks
    Aladin.prototype.on = function (what, myFunction) {
        if (Aladin.AVAILABLE_CALLBACKS.indexOf(what) < 0) {
            return;
        }

        this.callbacksByEventName[what] = myFunction;
    };

    Aladin.prototype.select = function () {
        this.fire('selectstart');
    };

    Aladin.prototype.fire = function (what, params) {
        if (what === 'selectstart') {
            this.view.setMode(View.SELECT);
        }
        else if (what === 'selectend') {
            this.view.setMode(View.PAN);
            var callbackFn = this.callbacksByEventName['select'];
            (typeof callbackFn === 'function') && callbackFn(params);
        }
    };

    Aladin.prototype.hideBoxes = function () {
        if (this.boxes) {
            for (var k = 0; k < this.boxes.length; k++) {
                this.boxes[k].hide();
            }
        }
    };

    // ?
    Aladin.prototype.updateCM = function () {

    };

    // TODO : LayerBox (or Stack?) must be extracted as a separate object
    Aladin.prototype.showLayerBox = function () {
        var self = this;

        // first, update
        var layerBox = $(this.aladinDiv).find('.aladin-layerBox');
        layerBox.empty();
        layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
            '<div style="clear: both;"></div>' +
            '</div>');

        layerBox.append('<div class="aladin-label">Projection</div>' +
        '<select id="projectionChoice"><option id="sinus" value="sinus">SINUS</option><option id="aitoff" value="aitoff">AITOFF</option><option id="mollweide" value="mollweide">MOLLWEIDE</option><option id="mercator" value="mercator">MERCATOR</option><option id="arc" value="arc">ARC</option><option id="tan" value="tan">TAN</option></select><br/>');

        $('#projectionChoice').change(function () {
            //$(this).selected = $(this).val();
            aladin.setProjection($(this).val());
        });
        
        layerBox.append('<div class="aladin-box-separator"></div>' +
        '<div class="aladin-label">Overlay layers</div>');

        //var cmDiv = layerBox.find('.aladin-cmap');

        // fill color maps options
        /*var cmSelect = layerBox.find('.aladin-cmSelection');
        for (var k = 0; k < ColorMap.MAPS_NAMES.length; k++) {
            cmSelect.append($("<option />").text(ColorMap.MAPS_NAMES[k]));
        }
        console.log(self.getBaseImageLayer())
        console.log(self.getBaseImageLayer().getColorMap())
        cmSelect.val(self.getBaseImageLayer().getColorMap().mapName);*/


        // loop over all overlay layers
        var layers = this.view.allOverlayLayers;
        var str = '<ul>';
        for (var k = layers.length - 1; k >= 0; k--) {
            var layer = layers[k];
            var name = layer.name;
            var checked = '';
            if (layer.isShowing) {
                checked = 'checked="checked"';
            }

            var tooltipText = '';
            var iconSvg = '';
            if (layer.type == 'catalog' || layer.type == 'progressivecat') {
                var nbSources = layer.getSources().length;
                tooltipText = nbSources + ' source' + (nbSources > 1 ? 's' : '');

                iconSvg = AladinUtils.SVG_ICONS.CATALOG;
            }
            else if (layer.type == 'moc') {
                tooltipText = 'Coverage: ' + (100 * layer.skyFraction()).toFixed(3) + ' % of sky';

                iconSvg = AladinUtils.SVG_ICONS.MOC;
            }
            else if (layer.type == 'overlay') {
                iconSvg = AladinUtils.SVG_ICONS.OVERLAY;
            }

            var rgbColor = $('<div></div>').css('color', layer.color).css('color'); // trick to retrieve the color as 'rgb(,,)' - does not work for named colors :(
            var labelColor = Color.getLabelColorForBackground(rgbColor);

            // retrieve SVG icon, and apply the layer color
            var svgBase64 = window.btoa(iconSvg.replace(/FILLCOLOR/g, layer.color));
            str += '<li><div class="aladin-stack-icon" style=\'background-image: url("data:image/svg+xml;base64,' + svgBase64 + '");\'></div>';
            str += '<input type="checkbox" ' + checked + ' id="aladin_lite_' + name + '"></input><label for="aladin_lite_' + name + '" class="aladin-layer-label" style="background: ' + layer.color + '; color:' + labelColor + ';" title="' + tooltipText + '">' + name + '</label></li>';
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
        reticleCb.change(function () {
            self.showReticle($(this).is(':checked'));
        });

        // Gestion grille Healpix
        checked = '';
        if (this.view.displayHpxGrid) {
            checked = 'checked="checked"';
        }
        var hpxGridCb = $('<input type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
        layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
        hpxGridCb.change(function () {
            self.showHealpixGrid($(this).is(':checked'));
        });

        // Equatorial grid plot
        checked = '';
        if (this.view.showGrid) {
            checked = 'checked="checked"';
        }
        var equatorialGridCb = $('<input type="checkbox" ' + checked + ' id="displayEquatorialGrid"/>');
        layerBox.append(equatorialGridCb).append('<label for="displayEquatorialGrid">Equatorial grid</label><br/>');
        equatorialGridCb.change(function () {
            let isChecked = $(this).is(':checked');
            self.view.setShowGrid(isChecked);
        });


        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Tools</div>');
        var exportBtn = $('<button class="aladin-btn" type="button">Export view as PNG</button>');
        layerBox.append(exportBtn);
        exportBtn.click(function () {
            self.exportAsPNG();
        });

        layerBox.find('.aladin-closeBtn').click(function () { self.hideBoxes(); return false; });

        // update list of surveys
        /*this.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        var surveySelection = $(this.aladinDiv).find('.aladin-surveySelection');
        surveySelection.change(function () {
            var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
            self.setImageSurvey(survey.id, function () {
                var baseImgLayer = self.getBaseImageLayer();

                if (baseImgLayer.useCors) {
                    // update color map list with current value color map
                    cmSelect.val(baseImgLayer.getColorMap().mapName);
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
        cmDiv.find('.aladin-cmSelection').change(function () {
            var cmName = $(this).find(':selected').val();
            self.getBaseImageLayer().getColorMap().update(cmName);
        });

        // reverse color map
        cmDiv.find('.aladin-reverseCm').click(function () {
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
        */
        //////////////////////////////////////////////////////////////////////


        // handler to hide/show overlays
        $(this.aladinDiv).find('.aladin-layerBox ul input').change(function () {
            var layerName = ($(this).attr('id').substr(12));
            var layer = self.layerByName(layerName);
            if ($(this).is(':checked')) {
                layer.show();
            }
            else {
                layer.hide();
            }
        });

        // finally show
        layerBox.show();

    };

    Aladin.prototype.layerByName = function (name) {
        var c = this.view.allOverlayLayers;
        for (var k = 0; k < c.length; k++) {
            if (name == c[k].name) {
                return c[k];
            }
        }
        return null;
    };

    // TODO : integrate somehow into API ?
    Aladin.prototype.exportAsPNG = function (imgFormat) {
        var w = window.open();
        w.document.write('<img src="' + this.getViewDataURL() + '">');
        w.document.title = 'Aladin Lite snapshot';
    };

    /**
     * Return the current view as a data URL (base64-formatted string)
     * Parameters:
     * - options (optional): object with attributs
     *     * format (optional): 'image/png' or 'image/jpeg'
     *     * width: width in pixels of the image to output
     *     * height: height in pixels of the image to output
     *
     * @API
    */
    Aladin.prototype.getViewDataURL = function (options) {
        var options = options || {};
        // support for old API signature
        if (typeof options !== 'object') {
            var imgFormat = options;
            options = { format: imgFormat };
        }

        return this.view.getCanvasDataURL(options.format, options.width, options.height);
    }

    /**
     * Return the current view WCS as a key-value dictionary
     * Can be useful in coordination with getViewDataURL
     *
     * @API
    */
    Aladin.prototype.getViewWCS = function (options) {
        var raDec = this.getRaDec();
        var fov = this.getFov();
        // TODO: support for other projection methods than SIN
        return {
            NAXIS: 2,
            NAXIS1: this.view.width,
            NAXIS2: this.view.height,
            RADECSYS: 'ICRS',
            CRPIX1: this.view.width / 2,
            CRPIX2: this.view.height / 2,
            CRVAL1: raDec[0],
            CRVAL2: raDec[1],
            CTYPE1: 'RA---SIN',
            CTYPE2: 'DEC--SIN',
            CD1_1: fov[0] / this.view.width,
            CD1_2: 0.0,
            CD2_1: 0.0,
            CD2_2: fov[1] / this.view.height
        }
    }

    /** restrict FOV range
     * @API
     * @param minFOV in degrees when zoom in at max
     * @param maxFOV in degreen when zoom out at max
    */
    Aladin.prototype.setFovRange = Aladin.prototype.setFOVRange = function (minFOV, maxFOV) {
        if (minFOV > maxFOV) {
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
     * @return a [ra, dec] array with world coordinates in degrees. Returns undefined is something went wrong
     * 
     */
    Aladin.prototype.pix2world = function (x, y) {
        // this might happen at early stage of initialization
        if (!this.view) {
            return undefined;
        }

        //var xy = AladinUtils.viewToXy(x, y, this.view.width, this.view.height, this.view.largestDim, this.view.zoomFactor);

        var radec;
        try {
            //radec = this.view.projection.unproject(xy.x, xy.y);
            radec = this.view.aladin.webglAPI.screenToWorld(x, y);
        }
        catch (e) {
            return undefined;
        }

        var res;
        // Convert it to icrs j2000
        if (this.view.aladin.webglAPI.cooSystem() === Aladin.wasmLibs.webgl.GALCooSys()) {
            res = this.view.aladin.webglAPI.Gal2J2000(radec[0], radec[1]);
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
    Aladin.prototype.world2pix = function (ra, dec) {
        // this might happen at early stage of initialization
        if (!this.view) {
            return;
        }

        var xy;
        if (this.view.cooFrame == CooFrameEnum.GAL) {
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
    Aladin.prototype.getFovCorners = function (nbSteps) {
        // default value: 1
        if (!nbSteps || nbSteps < 1) {
            nbSteps = 1;
        }

        var points = [];
        var x1, y1, x2, y2;
        for (var k = 0; k < 4; k++) {
            x1 = (k == 0 || k == 3) ? 0 : this.view.width - 1;
            y1 = (k < 2) ? 0 : this.view.height - 1;
            x2 = (k < 2) ? this.view.width - 1 : 0;
            y2 = (k == 1 || k == 2) ? this.view.height - 1 : 0;

            for (var step = 0; step < nbSteps; step++) {
                let radec = this.webglAPI.screenToWorld(x1 + step / nbSteps * (x2 - x1), y1 + step / nbSteps * (y2 - y1));
                points.push(radec);
            }
        }

        return points;

    };

    /**
     * @API
     * 
     * @return the current FoV size in degrees as a 2-elements array
     */
    Aladin.prototype.getFov = function () {
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
    Aladin.prototype.getSize = function () {
        return [this.view.width, this.view.height];
    };

    /**
     * @API
     * 
     * @return the jQuery object representing the DIV element where the Aladin Lite instance lies
     */
    Aladin.prototype.getParentDiv = function () {
        return $(this.aladinDiv);
    };

    return Aladin;
})();

///////////////////////////////
/////// Aladin Lite API ///////
///////////////////////////////
let A = {};
//// New API ////
// For developers using Aladin lite: all objects should be created through the API, 
// rather than creating directly the corresponding JS objects
// This facade allows for more flexibility as objects can be updated/renamed harmlessly

//@API
A.aladin = function (divSelector, options) {
    return new Aladin($(divSelector)[0], options);
};

/*//@API
// TODO : lecture de properties
A.imageLayer = function (rootURLOrHiPSDefinition, options) {
    return new HpxImageSurvey(rootURLOrHiPSDefinition, options);
};*/

// @API
A.source = function (ra, dec, data, options) {
    return new Source(ra, dec, data, options);
};

// @API
A.marker = function (ra, dec, options, data) {
    options = options || {};
    options['marker'] = true;
    return A.source(ra, dec, data, options);
};

A.createImageSurvey = async function(rootUrlOrId) {
    const survey = await HpxImageSurvey.create(rootUrlOrId);
    return survey;
}

// @API
A.polygon = function (raDecArray) {
    var l = raDecArray.length;
    if (l > 0) {
        // close the polygon if needed
        if (raDecArray[0][0] != raDecArray[l - 1][0] || raDecArray[0][1] != raDecArray[l - 1][1]) {
            raDecArray.push([raDecArray[0][0], raDecArray[0][1]]);
        }
    }
    return new Footprint(raDecArray);
};

//@API
A.polyline = function (raDecArray, options) {
    return new Polyline(raDecArray, options);
};


// @API
A.circle = function (ra, dec, radiusDeg, options) {
    return new Circle([ra, dec], radiusDeg, options);
};

/**
 * 
 * @API
 * 
 * @param ra 
 * @param dec
 * @param radiusRaDeg the radius along the ra axis in degrees
 * @param radiusDecDeg the radius along the dec axis in degrees
 * @param rotationDeg the rotation angle in degrees
 *   
 */
A.ellipse = function (ra, dec, radiusRaDeg, radiusDecDeg, rotationDeg, options) {
    return new Ellipse([ra, dec], radiusRaDeg, radiusDecDeg, rotationDeg, options);
};

// @API
A.graphicOverlay = function (options) {
    return new Overlay(options);
};

// Create a new image survey layer
//
// One can attach multiple surveys to 1 layer.
// Those survey colors are blended together.
// Layers are overlaid to each other
A.imageSurveyLayer = function(name) {
    return new ImageSurveyLayer(name);
}

// @API
A.catalog = function (options) {
    return new Catalog(options);
};

// @API
A.catalogHiPS = function (rootURL, options) {
    return new ProgressiveCat(rootURL, null, null, options);
};

// @API
/*
 * return a Box GUI element to insert content
 */
Aladin.prototype.box = function (options) {
    var box = new Box(options);
    box.$parentDiv.appendTo(this.aladinDiv);

    return box;
};

// @API
/*
 * show popup at ra, dec position with given title and content
 */
Aladin.prototype.showPopup = function (ra, dec, title, content) {
    this.view.catalogForPopup.removeAll();
    var marker = A.marker(ra, dec, { popupTitle: title, popupDesc: content, useMarkerDefaultIcon: false });
    this.view.catalogForPopup.addSources(marker);
    this.view.catalogForPopup.show();

    this.view.popup.setTitle(title);
    this.view.popup.setText(content);
    this.view.popup.setSource(marker);
    this.view.popup.show();
};

// @API
/*
 * hide popup
 */
Aladin.prototype.hidePopup = function () {
    this.view.popup.hide();
};

// @API
/*
 * return a URL allowing to share the current view
 */
Aladin.prototype.getShareURL = function () {
    var radec = this.getRaDec();
    var coo = new Coo();
    coo.prec = 7;
    coo.lon = radec[0];
    coo.lat = radec[1];

    return 'https://aladin.unistra.fr/AladinLite/?target=' + encodeURIComponent(coo.format('s')) +
        '&fov=' + this.getFov()[0].toFixed(2) + '&survey=' + encodeURIComponent(this.getBaseImageLayer().id || this.getBaseImageLayer().rootUrl);
};

// @API
/*
 * return, as a string, the HTML embed code
 */
Aladin.prototype.getEmbedCode = function () {
    var radec = this.getRaDec();
    var coo = new Coo();
    coo.prec = 7;
    coo.lon = radec[0];
    coo.lat = radec[1];

    var survey = this.getBaseImageLayer().id;
    var fov = this.getFov()[0];
    var s = '';
    s += '<link rel="stylesheet" href="https://aladin.unistra.fr/AladinLite/api/v2/latest/aladin.min.css" />\n';
    s += '<script type="text/javascript" src="https://code.jquery.com/jquery-1.9.1.min.js" charset="utf-8"></script>\n';
    s += '<div id="aladin-lite-div" style="width:400px;height:400px;"></div>\n';
    s += '<script type="text/javascript" src="https://aladin.unistra.fr/AladinLite/api/v2/latest/aladin.min.js" charset="utf-8"></script>\n';
    s += '<script type="text/javascript">\n';
    s += 'var aladin = A.aladin("#aladin-lite-div", {survey: "' + survey + 'P/DSS2/color", fov: ' + fov.toFixed(2) + ', target: "' + coo.format('s') + '"});\n';
    s += '</script>';
    return s;
};

// @API
/*
 * Creates remotely a HiPS from a FITS image URL and displays it
 */
Aladin.prototype.displayFITS = function (url, layerName, options, successCallback, errorCallback) {
    options = options || {};
    var data = { url: url };
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
    let self = this;
    $.ajax({
        url: 'https://alasky.unistra.fr/cgi/fits2HiPS',
        data: data,
        method: 'GET',
        dataType: 'json',
        success: function (response) {
            if (response.status != 'success') {
                console.error('An error occured: ' + response.message);
                if (errorCallback) {
                    errorCallback(response.message);
                }
                return;
            }
            var label = options.label || "FITS image";
            var meta = response.data.meta;

            (async () => {
                let survey = await Aladin.createImageSurvey(response.data.url);
                var transparency = (options && options.transparency) || 1.0;
    
                var executeDefaultSuccessAction = true;
                if (successCallback) {
                    executeDefaultSuccessAction = successCallback(meta.ra, meta.dec, meta.fov);
                }
                if (executeDefaultSuccessAction === true) {
                    self.webglAPI.setCenter(meta.ra, meta.dec);
                    self.setFoV(meta.fov);
                }
                // TODO! set an image survey once the already loaded surveys
                // are READY! Otherwise it can lead to some congestion and avoid
                // downloading the base tiles of the other surveys loading!
                // This has to be fixed in the backend but a fast fix is just to wait
                // before setting a new image survey
                
                    self.setImageSurvey(survey, layerName)
                    // set transparency
                    self.setOpacityLayer(transparency, layerName)

            })();
        }
    });

};

// @API
/*
 * Creates remotely a HiPS from a JPEG or PNG image with astrometry info
 * and display it
 */
Aladin.prototype.displayJPG = Aladin.prototype.displayPNG = function (url, layerName, options, successCallback, errorCallback) {
    options = options || {};
    options.color = true;
    options.label = "JPG/PNG image";
    options.outputFormat = 'png';
    this.displayFITS(url, layerName, options, successCallback, errorCallback);
};

Aladin.prototype.setReduceDeformations = function (reduce) {
    this.reduceDeformations = reduce;
    this.view.requestRedraw();
}

// API
A.footprintsFromSTCS = function (stcs) {
    var footprints = Overlay.parseSTCS(stcs);

    return footprints;
}

// API
A.MOCFromURL = function (url, options, successCallback) {
    var moc = new MOC(options);
    moc.dataFromFITSURL(url, successCallback);

    return moc;
};

// API
A.MOCFromJSON = function (jsonMOC, options) {
    var moc = new MOC(options);
    moc.dataFromJSON(jsonMOC);

    return moc;
};


// TODO: try first without proxy, and then with, if param useProxy not set
// API
A.catalogFromURL = function (url, options, successCallback, useProxy) {
    var catalog = A.catalog(options);
    // TODO: should be self-contained in Catalog class
    Catalog.parseVOTable(url, function (sources) {
        catalog.addSources(sources);
        if (successCallback) {
            successCallback(sources);
        }
    },
        catalog.maxNbSources, useProxy,
        catalog.raField, catalog.decField
    );

    return catalog;
};

// API
// @param target: can be either a string representing a position or an object name, or can be an object with keys 'ra' and 'dec' (values being in decimal degrees)
A.catalogFromSimbad = function (target, radius, options, successCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'Simbad';
    }
    var url = URLBuilder.buildSimbadCSURL(target, radius);
    return A.catalogFromURL(url, options, successCallback, false);
};

// API
A.catalogFromNED = function (target, radius, options, successCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'NED';
    }
    var url;
    if (target && (typeof target === "object")) {
        if ('ra' in target && 'dec' in target) {
            url = URLBuilder.buildNEDPositionCSURL(target.ra, target.dec, radius);
        }
    }
    else {
        var isObjectName = /[a-zA-Z]/.test(target);
        if (isObjectName) {
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

// API
A.catalogFromVizieR = function (vizCatId, target, radius, options, successCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'VizieR:' + vizCatId;
    }
    var url = URLBuilder.buildVizieRCSURL(vizCatId, target, radius, options);
    console.log(url);
    return A.catalogFromURL(url, options, successCallback, false);
};

// API
A.catalogFromSkyBot = function (ra, dec, radius, epoch, queryOptions, options, successCallback) {
    queryOptions = queryOptions || {};
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'SkyBot';
    }
    var url = URLBuilder.buildSkyBotCSURL(ra, dec, radius, epoch, queryOptions);
    return A.catalogFromURL(url, options, successCallback, false);
};

A.hipsDefinitionFromURL = function(url, successCallback) {
    HiPSDefinition.fromURL(url, successCallback);
};


A.init = Promise.all([import('@fxpineau/healpix'), import('../core/pkg')]).then(async (values) => {
    let [hpxAPI, webglAPI] = values;

    // HEALPix library
    Aladin.wasmLibs.hpx = hpxAPI;
    // WebGL library
    Aladin.wasmLibs.webgl = webglAPI;
});

// this is ugly for sure and there must be a better way using Webpack magic
window.A = A;

