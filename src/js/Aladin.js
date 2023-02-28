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
import { Logger } from "./Logger.js";
import { Catalog } from "./Catalog.js";
import { ProgressiveCat } from "./ProgressiveCat.js";
import { Sesame } from "./Sesame.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { MeasurementTable } from "./MeasurementTable.js";
import { Location } from "./Location.js";
import { Source } from "./Source.js";
import { ImageSurvey } from "./ImageSurvey.js";
import { Coo } from "./libs/astro/coo.js";
import { CooConversion } from "./CooConversion.js";
import { URLBuilder } from "./URLBuilder.js";
import { HiPSDefinition } from "./HiPSDefinition.js";
import { AladinLogo } from "./gui/AladinLogo.js";
import { ProjectionSelector } from "./gui/ProjectionSelector";
import { Stack } from "./gui/Stack.js";
import { CooGrid } from "./gui/CooGrid.js";
import { ALEvent } from "./events/ALEvent.js";
import { Color } from './Color.js';
import { ColorCfg } from './ColorCfg.js';
import { ImageFITS } from "./ImageFITS.js";

import $ from 'jquery';

// Import aladin css inside the project
import './../css/aladin.css';


export let Aladin = (function () {



    // Constructor
    var Aladin = function (aladinDiv, requestedOptions) {
        // check that aladinDiv exists, stop immediately otherwise
        if ($(aladinDiv).length == 0) {
            return;
        }
        this.wasm = null;
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
        let cooFrame = CooFrameEnum.fromString(options.cooFrame, CooFrameEnum.J2000);

        // locationDiv is the div where we write the position
        const locationDiv = $('<div class="aladin-location">'
            + (options.showFrame ? '<select class="aladin-frameChoice"><option value="' + CooFrameEnum.J2000.label + '" '
                + (cooFrame == CooFrameEnum.J2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.J2000d.label + '" '
                + (cooFrame == CooFrameEnum.J2000d ? 'selected="selected"' : '') + '>J2000d</option><option value="' + CooFrameEnum.GAL.label + '" '
                + (cooFrame == CooFrameEnum.GAL ? 'selected="selected"' : '') + '>GAL</option></select>' : '')
            + '<span class="aladin-clipboard" title="Copy coordinates to clipboard"></span>'
            + '<span class="aladin-location-text"></span>'
            + '</div>')
            .appendTo(aladinDiv);
        const copyCoo = locationDiv.find('.aladin-clipboard');
        copyCoo.hide();
        copyCoo.click(function() {
            let copyTextEl = locationDiv[0].querySelector('.aladin-location-text');
            var r = document.createRange();
            r.selectNode(copyTextEl);
            window.getSelection().removeAllRanges();
            window.getSelection().addRange(r);
            try {
                let successful = document.execCommand('copy');
                let msg = successful ? 'successful' : 'unsuccessful';
                console.log('Copying text command was ' + msg);
            } catch (err) {
                console.log('Oops, unable to copy');
            }
            window.getSelection().removeAllRanges();
        });
        locationDiv.mouseenter(function() {
            copyCoo.show();
        });
        locationDiv.mouseleave(function() {
            copyCoo.hide();
        });

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
        new AladinLogo(aladinDiv);

        // we store the boxes
        this.boxes = [];

        // measurement table
        this.measurementTable = new MeasurementTable(aladinDiv);



        var location = new Location(locationDiv.find('.aladin-location-text'));

        // set different options
        this.view = new View(this, location, fovDiv, cooFrame, options.fov);
        this.cacheSurveys = new Map();
        // Stack GUI
        this.stack = new Stack(this.aladinDiv, this, this.view);
        this.coogrid = new CooGrid(this.aladinDiv, this, this.view);

        this.boxes.push(this.stack);
        this.boxes.push(this.coogrid);

        // Grid
        var color, opacity;
        if (options.gridOptions) {
            color = options.gridOptions.color && Color.hexToRgb(options.gridOptions.color);
            opacity = options.gridOptions.opacity;
        } else {
            color = {r:0.0, g:1.0, b:0.0};
            opacity = 1.0;
        }

        this.view.setGridConfig({
            color: color,
            opacity: opacity,
        });

        if (options && options.showCooGrid) {
            this.view.setGridConfig({
                enabled: true,
            });
            this.view.showCooGrid = true;
        }

        if (options && (options.showProjectionControl === undefined || options.showProjectionControl === true)) {
            // Projection selector
            new ProjectionSelector(aladinDiv, this);
        }

        // Set the projection
        let projection = (options && options.projection) || 'SIN';
        this.setProjection(projection)

        let top_px = 30;

        // layers control panel
        // TODO : valeur des checkbox en fonction des options
        if (options.showLayersControl) {
            // button to show Stack interface
            var d = $('<div class="aladin-layersControl-container" style="top: ' + top_px + 'px" title="Manage layers"><div class="aladin-layersControl"></div></div>');
            d.appendTo(aladinDiv);

            if (options.expandLayersControl) {
                self.hideBoxes();
                self.showLayerBox();
            }

            // we return false so that the default event is not submitted, and to prevent event bubbling
            d.click(function () {
                self.hideBoxes();
                self.showLayerBox();
                return false;
            });
            top_px += 38;
        }

        // goto control panel
        if (options.showGotoControl) {
            var d = $('<div class="aladin-gotoControl-container" style="top: ' + top_px + 'px" title="Go to position"><div class="aladin-gotoControl"></div></div>');
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

            // Unfocus the keyboard on android devices (maybe it concerns all smartphones) when the user click on enter
            input.on("change", function () {
                input.blur();
            });

            // TODO : classe GotoBox
            d.click(function () {
                self.hideBoxes();
                input.val('');
                input.removeClass('aladin-unknownObject');
                gotoBox.show();
                input.blur();

                return false;
            });
            gotoBox.find('.aladin-closeBtn').click(function () { self.hideBoxes(); input.blur(); return false; });
            top_px += 38;
        }

        // simbad pointer tool
        if (options.showSimbadPointerControl) {
            var d = $('<div class="aladin-simbadPointerControl-container" style="top: ' + top_px + 'px" title="SIMBAD pointer"><div class="aladin-simbadPointerControl"></div></div>');
            d.appendTo(aladinDiv);

            d.click(function () {
                self.view.setMode(View.TOOL_SIMBAD_POINTER);
            });
            top_px += 38;
        }

        // Coo grid pointer tool
        if (options.showCooGridControl) {
            var d = $('<div class="aladin-cooGridControl-container" style="top: ' + top_px + 'px" title="Coo grid"><div class="aladin-cooGridControl"></div></div>');
            d.appendTo(aladinDiv);

            d.click(function () {
                self.hideBoxes(); self.showCooGridBox(); return false;
            });
            top_px += 38;
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
            top_px += 38;
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


        // Add the image layers
        // For that we check the survey key of options
        // It can be given as a single string or an array of strings
        // for multiple blending surveys

        if (options.survey) {
            if (Array.isArray(options.survey)) {
                let i = 0;
                options.survey.forEach((rootURLOrId) => {
                    if (i == 0) {
                        this.setBaseImageLayer(rootURLOrId);
                    } else {
                        this.setOverlayImageLayer(rootURLOrId, Utils.uuidv4());
                    }
                    i++;
                });
            } else {                
                this.setBaseImageLayer(options.survey);
            }
        } else {
            const idxServiceUrl = Math.round(Math.random());
            const dssUrl = Aladin.DEFAULT_OPTIONS.surveyUrl[idxServiceUrl]

            this.setBaseImageLayer(dssUrl);
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
        //if (typeof Vue != "undefined") {
            //this.discoverytree = new DiscoveryTree(this);
        //}

        this.view.redraw();

        // go to full screen ?
        if (options.fullScreen) {
            // strange behaviour to wait for a sec
            self.toggleFullscreen(self.options.realFullscreen);
        }
    };

    /**** CONSTANTS ****/
    Aladin.VERSION = "3.0-beta0";

    Aladin.JSONP_PROXY = "https://alasky.cds.unistra.fr/cgi/JSONProxy";
    //Aladin.JSONP_PROXY = "https://alaskybis.unistra.fr/cgi/JSONProxy";

    Aladin.URL_PREVIEWER = 'https://aladin.cds.unistra.fr/AladinLite/';

    // access to WASM libraries
    Aladin.wasmLibs = {};
    Aladin.DEFAULT_OPTIONS = {
        surveyUrl: ["https://alaskybis.u-strasbg.fr/DSS/DSSColor", "https://alasky.u-strasbg.fr/DSS/DSSColor"],
        survey: "CDS/P/DSS2/color",
        target: "0 +0",
        cooFrame: "J2000",
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
        let self = this;

        realFullscreen = Boolean(realFullscreen);

        this.fullScreenBtn.toggleClass('aladin-maximize aladin-restore');
        var isInFullscreen = this.fullScreenBtn.hasClass('aladin-restore');
        this.fullScreenBtn.attr('title', isInFullscreen ? 'Restore original size' : 'Full screen');
        //$(this.aladinDiv).toggleClass('aladin-fullscreen');
        if (this.aladinDiv.classList.contains('aladin-fullscreen')) {
            this.aladinDiv.classList.remove('aladin-fullscreen');
        } else {
            this.aladinDiv.classList.add('aladin-fullscreen');
        }

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
        
        // Delay the fixLayoutDimensions layout for firefox
        setTimeout(function () {
            self.view.fixLayoutDimensions();
        }, 1000);

        // force call to zoomChanged callback
        var fovChangedFn = self.callbacksByEventName['zoomChanged'];
        (typeof fovChangedFn === 'function') && fovChangedFn(self.view.fov);

        var fullScreenToggledFn = self.callbacksByEventName['fullScreenToggled'];
        (typeof fullScreenToggledFn === 'function') && fullScreenToggledFn(isInFullscreen);
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
        if (requestedSurveyId && ImageSurvey.getSurveyInfoFromId(requestedSurveyId)) {
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

    Aladin.prototype.setProjection = function (projection) {
        if (!projection) {
            return;
        }
        this.view.setProjection(projection);
        ALEvent.PROJECTION_CHANGED.dispatchedTo(this.aladinDiv, {projection: projection});
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
            // Convert from view coo sys to icrsj2000
            const [ra, dec] = this.wasm.viewToICRSJ2000CooSys(coo.lon, coo.lat);
            this.view.pointTo(ra, dec, options);

            (typeof successCallback === 'function') && successCallback(this.getRaDec());
        }
        // ask resolution by Sesame
        else {
            var self = this;
            Sesame.resolve(targetName,
                function (data) { // success callback
                    // Location given in icrs at J2000
                    const coo = data.Target.Resolver;
                    self.view.pointTo(coo.jradeg, coo.jdedeg, options);

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

        setTimeout(function () { doAnimation(aladin); }, 10);

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
        let radec = this.wasm.getCenter(); // This is given in the frame of the view
        // We must convert it to ICRSJ2000
        const radec_j2000 = this.wasm.viewToICRSJ2000CooSys(radec[0], radec[1]);

        if (radec_j2000[0]<0) {
            return [radec_j2000[0] + 360.0, radec_j2000[1]];
        }

        return radec_j2000;
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

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(this.aladinDiv, {layer: catalog});
        
    };
    Aladin.prototype.addOverlay = function (overlay) {
        this.view.addOverlay(overlay);

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(this.aladinDiv, {layer: overlay});
    };
    Aladin.prototype.addMOC = function (moc) {
        this.view.addMOC(moc);
    };

    // @API
    Aladin.prototype.findLayerByUUID = function(uuid) {
        const result = this.view.allOverlayLayers.filter(layer => layer.uuid===uuid);
        if (result.length==0) {
            return null;
        }

        return result[0];
    }

    // @API
    Aladin.prototype.removeLayer = function(layer) {
        this.view.removeLayer(layer);
    };

    // @oldAPI
    Aladin.prototype.createImageSurvey = function(id, name, rootUrl, cooFrame, maxOrder, options = {}) {
        let cfg = this.cacheSurveys.get(id);
        if (!cfg) {
            // Add the cooFrame and maxOrder given by the user
            // to the list of options passed to the ImageSurvey constructor
            if (cooFrame) {
                options.cooFrame = cooFrame;
            }

            if (maxOrder) {
                options.maxOrder = maxOrder;
            }
    
            cfg = {id, name, rootUrl, options};
            this.cacheSurveys.set(id, cfg);
        } else {
            cfg = Utils.clone(cfg)
        }

        return new ImageSurvey(cfg.id, cfg.name, cfg.rootUrl, this.view, cfg.options);
    };

    Aladin.prototype.createImageFITS = function(url, name, options = {}, successCallback = undefined, errorCallback = undefined) {
        try {
            url = new URL(url);
        } catch(e) {
            // The url could be created
            url = Utils.getAbsoluteURL(url)
            url = new URL(url);
        }

        // Check the protocol, for http ones, use a CORS compatible proxy
        if (Utils.requestCORSIfNotSameOrigin(url)) {
            // http(s) protocols and not in localhost
            let proxiedUrl = new URL(Aladin.JSONP_PROXY);
            proxiedUrl.searchParams.append("url", url);

            url = proxiedUrl;
        }

        let cfg = this.cacheSurveys.get(url);
        if (!cfg) {
            cfg = {url, name, options, successCallback, errorCallback}
            this.cacheSurveys.set(url, cfg);
        } else {
            cfg = Utils.clone(cfg)
        }

        return new ImageFITS(cfg.url, cfg.name, this.view, cfg.options, cfg.successCallback, cfg.errorCallback);
    };

    Aladin.prototype.newImageSurvey = function(rootUrlOrId, options) {
        const idOrUrl = rootUrlOrId;
        // Check if the survey has already been added
        // Create a new ImageSurvey
        const name = idOrUrl;

        try {
            const url = new URL(rootUrlOrId).toString()    
            
            // Valid URL case
            const id = url;
            return this.createImageSurvey(id, name, url, null, null, options);
        } catch (e) {
            // Valid ID case
            const id = idOrUrl;
            return this.createImageSurvey(id, name, undefined, null, null, options);
        }
    }

    // @param imageSurvey : ImageSurvey object or image survey identifier
    // @api
    // @old

    Aladin.prototype.setImageLayer = function(imageLayer) {
        this.setBaseImageLayer(imageLayer);
    };

    Aladin.prototype.setImageSurvey = Aladin.prototype.setImageLayer;

    // @param imageSurvey : ImageSurvey object or image survey identifier
    // @api
    // @old
    Aladin.prototype.setBackgroundColor = function(rgb) {
        let color;
        if (typeof rgb === "string") {
            var rgb = rgb.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/);

            var r = parseInt(rgb[1]);
            var g = parseInt(rgb[2]);
            var b = parseInt(rgb[3]);

            color = { r: r, g: g, b: b };
        } else {
            color = rgb;
        }
        this.wasm.setBackgroundColor(color);
    };

    // @api
    Aladin.prototype.removeImageLayer = function(layer) {
        this.view.removeImageLayer(layer);
    };

    // @api
    Aladin.prototype.setBaseImageLayer = function(idOrSurvey) {
        return this.setOverlayImageLayer(idOrSurvey, "base");
    };

    // @api
    Aladin.prototype.getBaseImageLayer = function () {
        return this.view.getImageLayer("base");
    };

    // @api
    Aladin.prototype.setOverlayImageLayer = function (idOrUrlOrImageLayer, layer = "overlay") {
        let imageLayer;
        // 1. User gives an ID
        if (typeof idOrUrlOrImageLayer === "string") {
            const idOrUrl = idOrUrlOrImageLayer;
            // Check if the survey has already been added
            // Create a new ImageSurvey
            let isUrl = false;
            if (idOrUrl.includes("http")) {
                isUrl = true;
            }
            const name = idOrUrl;

            if (isUrl) {
                const url = idOrUrl;
                const id = url;
                // Url
                imageLayer = this.createImageSurvey(id, name, url, null, null);
            } else {
                const id = idOrUrl;
                // ID
                imageLayer = this.createImageSurvey(id, name, undefined, null, null);
            }
        // 2. User gives a non resolved promise
        } else {
            imageLayer = idOrUrlOrImageLayer;
        }

        return this.view.setOverlayImageLayer(imageLayer, layer);
    };

    // @api
    Aladin.prototype.getOverlayImageLayer = function(layer = "overlay") {
        const survey = this.view.getImageLayer(layer);
        return survey;
    };

    // @api
    Aladin.prototype.increaseZoom = function () {
        this.view.increaseZoom(0.01);
    };

    Aladin.prototype.decreaseZoom = function () {
        this.view.decreaseZoom(0.01);
    };

    Aladin.prototype.setRotation = function(rotation) {
        this.view.setRotation(rotation);
    }

    // @api
    // Set the current layer that is targeted
    // Rightclicking for changing the cuts is done the targeted layer
    Aladin.prototype.setActiveHiPSLayer = function (layer) {
        this.view.setActiveHiPSLayer(layer);
    }

    Aladin.prototype.getActiveHiPSLayer = function () {
        return this.view.selectedLayer;
    }

    // Get the list of image layer overlays
    Aladin.prototype.getImageOverlays = function () {
        return this.view.overlayLayers;
    }

    // Get the list of overlays
    Aladin.prototype.getOverlays = function () {
        return this.view.allOverlayLayers;
    }

    // Get the list of overlays
    Aladin.prototype.getImageOverlays = function () {
        return this.view.overlayLayers;
    }

    Aladin.prototype.isHpxGridDisplayed = function () {
        return this.view.displayHpxGrid;
    }

    Aladin.prototype.isReticleDisplayed = function () {
        return this.view.displayReticle;
    }

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

    Aladin.prototype.addListener = function(alEventName, customFn) {
        new ALEvent(alEventName).listenedBy(this.aladinDiv, customFn);
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
        this.stack.show();
    };

    Aladin.prototype.showCooGridBox = function () {
        this.coogrid.show();
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
        (async () => {
            var w = window.open();
            w.document.write('<img src="' + await this.getViewDataURL() + '" width="' + this.view.width + 'px">');
            w.document.title = 'Aladin Lite snapshot';
        })();
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
    Aladin.prototype.getViewDataURL = async function (options) {
        var options = options || {};
        // support for old API signature
        if (typeof options !== 'object') {
            var imgFormat = options;
            options = { format: imgFormat };
        }
        const canvasDataURL = await this.view.getCanvasDataURL(options.format, options.width, options.height);
        return canvasDataURL;
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

        try {
            const [ra, dec] = this.wasm.screenToWorld(x, y);

            if (ra < 0) {
                return [ra + 360.0, dec];
            }

            return [ra, dec];
        } catch (e) {
            return undefined;
        }
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

        try {
            return this.wasm.worldToScreen(ra, dec);
        } catch (e) {
            return undefined;
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
                let radec = this.wasm.screenToWorld(x1 + step / nbSteps * (x2 - x1), y1 + step / nbSteps * (y2 - y1));
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
console.log("jkjkjkj")

//// New API ////
// For developers using Aladin lite: all objects should be created through the API, 
// rather than creating directly the corresponding JS objects
// This facade allows for more flexibility as objects can be updated/renamed harmlessly

//@API
A.aladin = function (divSelector, options) {
    return new Aladin($(divSelector)[0], options);
};

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

    return Aladin.URL_PREVIEWER + '?target=' + encodeURIComponent(coo.format('s')) +
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
    let s = '';
    const NL = "\n";
    s += '<div id="aladin-lite-div" style="width:400px;height:400px;"></div>' + NL;
    s += '<script src="https://aladin.cds.unistra.fr/AladinLite/api/v3/latest/aladin.js" charset="utf-8"></script>' + NL;
    s += '<script>' + NL;
    s += "let aladin;" + NL + "A.init.then(() => {" + NL + "   aladin = A.aladin('#aladin-lite-div', {survey: 'P/DSS2/color', fov: " + fov.toFixed(2) + ', target: "' + coo.format('s') + '"});' + NL + '});' + NL;
    s += '</script>';

    return s;
};

// @API
/*
 * Creates remotely a HiPS from a FITS image URL and displays it
 */
Aladin.prototype.displayFITS = function (url, options, successCallback, errorCallback, layer = "base") {
    const imageFits = this.createImageFITS(url, url, options, successCallback, errorCallback);
    this.setOverlayImageLayer(imageFits, layer)
};

// @API
/*
 * Creates remotely a HiPS from a JPEG or PNG image with astrometry info
 * and display it
 */
Aladin.prototype.displayJPG = Aladin.prototype.displayPNG = function (url, options, successCallback, errorCallback) {
    options = options || {};
    options.color = true;
    options.label = "JPG/PNG image";
    options.outputFormat = 'png';

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

    const request = ( url, params = {}, method = 'GET' ) => {
        let options = {
            method
        };
        if ( 'GET' === method ) {
            url += '?' + ( new URLSearchParams( params ) ).toString();
        } else {
            options.body = JSON.stringify( params );
        }
        
        return fetch( url, options ).then( response => response.json() );
    };
    const get = ( url, params ) => request( url, params, 'GET' );

    get('https://alasky.unistra.fr/cgi/fits2HiPS', data)
        .then(async (response) => {
            if (response.status != 'success') {
                console.error('An error occured: ' + response.message);
                if (errorCallback) {
                    errorCallback(response.message);
                }
                return;
            }
            var label = options.label || "FITS image";
            var meta = response.data.meta;

            const survey = self.createImageSurvey(response.data.url, label);
            self.setOverlayImageLayer(survey, "overlay");

            var transparency = (options && options.transparency) || 1.0;

            var executeDefaultSuccessAction = true;
            if (successCallback) {
                executeDefaultSuccessAction = successCallback(meta.ra, meta.dec, meta.fov);
            }
            if (executeDefaultSuccessAction === true) {
                self.wasm.setCenter(meta.ra, meta.dec);
                self.setFoV(meta.fov);
            }

            // TODO! set an image survey once the already loaded surveys
            // are READY! Otherwise it can lead to some congestion and avoid
            // downloading the base tiles of the other surveys loading!
            // This has to be fixed in the backend but a fast fix is just to wait
            // before setting a new image survey
        });
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

A.getAvailableListOfColormaps = function() {
    return ColorCfg.COLORMAPS;
};

A.init = (async () => {
    const isWebGL2Supported = document
        .createElement('canvas')
        .getContext('webgl2');

    // Check for webgl2 support
    if (isWebGL2Supported) {
        const module = await import('./../../pkg-webgl2');
        Aladin.wasmLibs.core = module;
    } else {
        // WebGL1 not supported
        // According to caniuse, https://caniuse.com/webgl2, webgl2 is supported by 89% of users
        throw "WebGL2 not supported by your browser";
    }
})();

window.A = A;
