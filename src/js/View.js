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

import { Aladin }   from "./Aladin.js";
import { Popup }          from "./Popup.js";
import { HealpixGrid }    from "./HealpixGrid.js";
import { ProjectionEnum } from "./ProjectionEnum.js";
import { Projection }     from "./libs/astro/projection.js";
import { AladinUtils }    from "./AladinUtils.js";
import { Utils }          from "./Utils.js";
import { SimbadPointer }  from "./SimbadPointer.js";
import { Stats }          from "./libs/Stats.js";
import { ColorMap } from "./ColorMap.js";
import { Footprint } from "./Footprint.js";
import { Circle } from "./Circle.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";
import { WebGLCtx } from "./WebGL.js";
import { Logger } from "./Logger.js";
import { ALEvent } from "./events/ALEvent.js";

export let View = (function() {

    /** Constructor */
    function View (aladin, location, fovDiv, cooFrame, zoom) {
        this.aladin = aladin;
        // Add a reference to the WebGL API
        this.options = aladin.options;
        this.aladinDiv = this.aladin.aladinDiv;
        this.popup = new Popup(this.aladinDiv, this);
        this.webGL2Support = WebGLCtx.checkForWebGL2Support();
        this.createCanvases();
        // Init the WebGL context
        // At this point, the view has been created so the image canvas too
        try {
            // Start our Rust application. You can find `WebClient` in `src/lib.rs`
            // The Rust part should also create a new WebGL2 or WebGL1 context depending on the WebGL2 brower support.
            this.aladin.webglAPI = new WebGLCtx.init(Aladin.wasmLibs.webgl, this.aladinDiv.id);
        } catch(e) {
            // For browsers not supporting WebGL2:
            // 1. Print the original exception message in the console
            console.error(e)
            // 2. Add a more explicite message to the end user
            alert("Problem initializing Aladin Lite. Please contact the support by contacting Matthieu Baumann (baumannmatthieu0@gmail.com) or Thomas Boch (thomas.boch@astro.unistra.fr). You can also open an issue on the Aladin Lite github repository here: https://github.com/cds-astro/aladin-lite. Message error:" + e)
        }

        this.location = location;
        this.fovDiv = fovDiv;
        this.mustClearCatalog = true;
        //this.imageSurveysToSet = [];
        this.mode = View.PAN;
        
        this.minFOV = this.maxFOV = null; // by default, no restriction
        this.fov_limit = 180.0;
        
        this.healpixGrid = new HealpixGrid();
        this.then = Date.now();
        
        var lon, lat;
        lon = lat = 0;
        
        this.projection = new Projection(lon, lat);
        this.projection.setProjection(ProjectionEnum.SIN);
        //this.zoomLevel = 0;
        // Prev time of the last frame
        this.prev = 0;
        //this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
        this.zoomFactor = this.aladin.webglAPI.getClipZoomFactor();

        this.viewCenter = {lon: lon, lat: lat}; // position of center of view

        if (cooFrame) {
            this.cooFrame = cooFrame;
        } else {
            this.cooFrame = CooFrameEnum.GAL;
        }

        // Frame setting
        this.changeFrame(this.cooFrame);

        // Zoom starting setting
        const si = 500000.0;
        const alpha = 40.0;

        let initialFov = zoom || 360.0;
        this.pinchZoomParameters = {
            isPinching: false, // true if a pinch zoom is ongoing
            initialFov: undefined,
            initialDistance: undefined,
            initialAccDelta: Math.pow(si / initialFov, 1.0/alpha)
        };
        this.setZoom(initialFov);
        // current reference image survey displayed
        this.imageSurveys = new Map();
        this.imageSurveysWaitingList = new Map();
        this.imageSurveysIdx = new Map();

        this.overlayLayers = [];
        // current catalogs displayed
        this.catalogs = [];
        // a dedicated catalog for the popup
        var c = document.createElement('canvas');
        c.width = c.height = 24;
        var ctx= c.getContext('2d');
        ctx.lineWidth = 6.0;
        ctx.beginPath();
        ctx.strokeStyle = '#eee';
        ctx.arc(12, 12, 8, 0, 2*Math.PI, true);
        ctx.stroke();
        ctx.lineWidth = 3.0;
        ctx.beginPath();
        ctx.strokeStyle = '#c38';
        ctx.arc(12, 12, 8, 0, 2*Math.PI, true);
        ctx.stroke();
        this.catalogForPopup = A.catalog({shape: c, sourceSize: 24});
        //this.catalogForPopup = A.catalog({sourceSize: 18, shape: 'circle', color: '#c38'});
        this.catalogForPopup.hide();
        this.catalogForPopup.setView(this);
        // overlays (footprints for instance)
        this.overlays = [];
        // MOCs
        this.mocs = [];
        // reference to all overlay layers (= catalogs + overlays + mocs)
        this.allOverlayLayers = []
        

        
        this.fixLayoutDimensions();
        
        this.firstHiPS = true;
        this.curNorder = 1;
        this.realNorder = 1;
        
        // some variables for mouse handling
        this.dragging = false;
        this.dragx = null;
        this.dragy = null;
        this.rightclickx = null;
        this.rightclicky = null;
        this.selectedSurveyLayer = 'base';

        this.needRedraw = true;

        // two-fingers rotation
        this.fingersRotationParameters = {
            initialViewAngleFromCenter: undefined,
            initialFingerAngle: undefined,
            rotationInitiated: false
        }

        this.fadingLatestUpdate = null;
        
        this.dateRequestRedraw = null;
        
        
        init(this);
        

        // listen to window resize and reshape canvases
        this.resizeTimer = null;
        var self = this;
        $(window).resize(function() {
            self.fixLayoutDimensions(self);
            self.requestRedraw();
        });

        // in some contexts (Jupyter notebook for instance), the parent div changes little time after Aladin Lite creation
        // this results in canvas dimension to be incorrect.
        // The following line tries to fix this issue
        setTimeout(function() {
            var computedWidth = $(self.aladinDiv).width();
            var computedHeight = $(self.aladinDiv).height();

            if (self.width!==computedWidth || self.height===computedHeight) {
                self.fixLayoutDimensions();
                // As the WebGL backend has been resized correctly by
                // the previous call, we can get the zoom factor from it
                
                self.setZoom(self.fov); // needed to force recomputation of displayed FoV
            }
        }, 1000);

    };
    
    // different available modes
    View.PAN = 0;
    View.SELECT = 1;
    View.TOOL_SIMBAD_POINTER = 2;
        
    
    // TODO: should be put as an option at layer level    
    View.DRAW_SOURCES_WHILE_DRAGGING = true;
    View.DRAW_MOCS_WHILE_DRAGGING = true;

    View.CALLBACKS_THROTTLE_TIME_MS = 100; // minimum time between two consecutive callback calls

    
    // (re)create needed canvases
    View.prototype.createCanvases = function() {
        var a = $(this.aladinDiv);
        a.find('.aladin-imageCanvas').remove();
        a.find('.aladin-catalogCanvas').remove();
        
        // canvas to draw the images
        this.imageCanvas = $("<canvas class='aladin-imageCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the catalogs
        this.catalogCanvas = $("<canvas class='aladin-catalogCanvas'></canvas>").appendTo(this.aladinDiv)[0];
    };
    
    // called at startup and when window is resized
    // The WebGL backend is resized
    View.prototype.fixLayoutDimensions = function() {
        Utils.cssScale = undefined;

        var computedWidth = $(this.aladinDiv).width();
        var computedHeight = $(this.aladinDiv).height();

        this.width = Math.max(computedWidth, 1);
        this.height = Math.max(computedHeight, 1); // this prevents many problems when div size is equal to 0

        this.cx = this.width/2;
        this.cy = this.height/2;
        
        this.largestDim = Math.max(this.width, this.height);
        this.smallestDim = Math.min(this.width, this.height);
        this.ratio = this.largestDim/this.smallestDim;

        this.mouseMoveIncrement = 160/this.largestDim;

        // reinitialize 2D context
        this.imageCtx = this.imageCanvas.getContext(this.webGL2Support ? "webgl2" : "webgl");
        this.aladin.webglAPI.resize(this.width, this.height);
        this.catalogCtx = this.catalogCanvas.getContext("2d");

        this.catalogCtx.canvas.width = this.width;        
        this.catalogCtx.canvas.height = this.height;

        pixelateCanvasContext(this.imageCtx, this.aladin.options.pixelateCanvas);

        // change logo
        if (!this.logoDiv) {
            this.logoDiv = $(this.aladinDiv).find('.aladin-logo')[0];
        }
        if (this.width>800) {
            $(this.logoDiv).removeClass('aladin-logo-small');
            $(this.logoDiv).addClass('aladin-logo-large');
            $(this.logoDiv).css('width', '90px');
        }
        else {
            $(this.logoDiv).addClass('aladin-logo-small');
            $(this.logoDiv).removeClass('aladin-logo-large');
            $(this.logoDiv).css('width', '32px');
        }

        this.computeNorder();
    };

    var pixelateCanvasContext = function(ctx, pixelateFlag) {
        var enableSmoothing = ! pixelateFlag;
        ctx.imageSmoothingEnabled = enableSmoothing;
        ctx.webkitImageSmoothingEnabled = enableSmoothing;
        ctx.mozImageSmoothingEnabled = enableSmoothing;
        ctx.msImageSmoothingEnabled = enableSmoothing;
        ctx.oImageSmoothingEnabled = enableSmoothing;
    }
    

    View.prototype.setMode = function(mode) {
        this.mode = mode;
        if (this.mode==View.SELECT) {
            this.setCursor('crosshair');
        }
        else if (this.mode==View.TOOL_SIMBAD_POINTER) {
            this.popup.hide();
            this.catalogCanvas.style.cursor = '';
            $(this.catalogCanvas).addClass('aladin-sp-cursor');
        }
        else {
            this.setCursor('default');
        }
    };
    
    View.prototype.setCursor = function(cursor) {
        if (this.catalogCanvas.style.cursor==cursor) {
            return;
        }
        if (this.mode==View.TOOL_SIMBAD_POINTER) {
            return;
        }
        this.catalogCanvas.style.cursor = cursor;
    };

    
    
    /**
     * return dataURL string corresponding to the current view
     */
    View.prototype.getCanvasDataURL = function(imgType, width, height) {
        imgType = imgType || "image/png"; 
        var c = document.createElement('canvas');
        width = width || this.width;
        height = height || this.height;
        c.width = width;
        c.height = height;
        var ctx = c.getContext('2d');

        //ctx.drawImage(this.imageCanvas, 0, 0, c.width, c.height);
        const canvas = this.aladin.webglAPI.canvas();
        ctx.drawImage(canvas, 0, 0, c.width, c.height);
        ctx.drawImage(this.catalogCanvas, 0, 0, c.width, c.height);

        return c.toDataURL(imgType);
        //return c.toDataURL("image/jpeg", 0.01); // setting quality only works for JPEG (?)
    };


    View.prototype.setActiveHiPSLayer = function(layer) {
        if (!this.imageSurveys.has(layer)) {
            throw layer + ' does not exists. So cannot be selected';
        }

        this.selectedSurveyLayer = layer;
    };

    View.prototype.updateFovDiv = function() {
        if (isNaN(this.fov)) {
            this.fovDiv.html("FoV:");
            return;
        }
        // update FoV value
        var fovStr;
        if (this.fov>1) {
            fovStr = Math.round(this.fov*100)/100 + "°";
        }
        else if (this.fov*60>1) {
            fovStr = Math.round(this.fov*60*100)/100 + "'";
        }
        else {
            fovStr = Math.round(this.fov*3600*100)/100 + '"';
        }
        this.fovDiv.html("FoV: " + fovStr);
    }
    
    var createListeners = function(view) {
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
            hasTouchEvents = true;
        }
        
        // various listeners
        let onDblClick = function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            /*if(view.aladin.webglAPI.posOnUi()) {
                return;
            }*/
            try {
                const lonlat = view.aladin.webglAPI.screenToWorld(xymouse.x, xymouse.y);
                var radec = view.aladin.webglAPI.viewToICRSJ2000CooSys(lonlat[0], lonlat[1]);
                view.pointTo(radec[0], radec[1], {forceAnimation: true});
            }
            catch(err) {
                return;
            }
            
        };
        if (! hasTouchEvents) {
            $(view.catalogCanvas).dblclick(onDblClick);
        }

        $(view.catalogCanvas).bind("contextmenu", function(e) {
            // do something here... 
            e.preventDefault(); 
        }, false);

        let cutMinInit = null
        let cutMaxInit = null;

        $(view.catalogCanvas).bind("mousedown touchstart", function(e) {
            e.preventDefault();
            e.stopPropagation();

            var xymouse = view.imageCanvas.relMouseCoords(e);

            if (e.which === 3 || e.button === 2) {
                view.rightClick = true;
                view.rightclickx = xymouse.x;
                view.rightclicky = xymouse.y;

                const survey = view.imageSurveys.get(view.selectedSurveyLayer);
                if (survey) {
                    // Take as start cut values what is inside the properties
                    // If the cuts are not defined in the metadata of the survey
                    // then we take what has been defined by the user
                    if (!survey.colored) {
                        if (survey.fits) {
                            // properties default cuts always refers to fits tiles
                            cutMinInit = survey.properties.minCutout || survey.options.minCut;
                            cutMaxInit = survey.properties.maxCutout || survey.options.maxCut;
                        } else {
                            cutMinInit = survey.options.minCut;
                            cutMaxInit = survey.options.maxCut;
                        }
                    } else {
                        // todo: contrast
                    }
                }

                return;
            }

            // zoom pinching
            if (e.type==='touchstart' && e.originalEvent && e.originalEvent.targetTouches && e.originalEvent.targetTouches.length==2) {
                view.dragging = false;

                view.pinchZoomParameters.isPinching = true;
                //var fov = view.aladin.getFov();
                //view.pinchZoomParameters.initialFov = Math.max(fov[0], fov[1]);
                var fov = view.aladin.webglAPI.getFieldOfView();
                view.pinchZoomParameters.initialFov = fov;
                view.pinchZoomParameters.initialDistance = Math.sqrt(Math.pow(e.originalEvent.targetTouches[0].clientX - e.originalEvent.targetTouches[1].clientX, 2) + Math.pow(e.originalEvent.targetTouches[0].clientY - e.originalEvent.targetTouches[1].clientY, 2));

                view.fingersRotationParameters.initialViewAngleFromCenter = view.aladin.webglAPI.getRotationAroundCenter();
                view.fingersRotationParameters.initialFingerAngle = Math.atan2(e.originalEvent.targetTouches[1].clientY - e.originalEvent.targetTouches[0].clientY, e.originalEvent.targetTouches[1].clientX - e.originalEvent.targetTouches[0].clientX) * 180.0 / Math.PI;

                return;
            }

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

            view.aladin.webglAPI.pressLeftMouseButton(view.dragx, view.dragy);
            return false; // to disable text selection
        });

        $(view.catalogCanvas).bind("mouseup", function(e) {
            if (view.rightClick) {
                view.rightClick = false;
                view.rightclickx = null;
                view.rightclicky = null;

                return;
            }
        });

        $(view.catalogCanvas).bind("click mouseout touchend", function(e) { // reacting on 'click' rather on 'mouseup' is more reliable when panning the view                 
            if (e.type==='touchend' && view.pinchZoomParameters.isPinching) {
                view.pinchZoomParameters.isPinching = false;
                view.pinchZoomParameters.initialFov = view.pinchZoomParameters.initialDistance = undefined;
    
                return;
            }
            if (e.type==='touchend' && view.fingersRotationParameters.rotationInitiated) {
                view.fingersRotationParameters.initialViewAngleFromCenter = undefined;
                view.fingersRotationParameters.initialFingerAngle = undefined;
                view.fingersRotationParameters.rotationInitiated = false;
    
                return;
            }

            var wasDragging = view.realDragging === true;
            var selectionHasEnded = view.mode===View.SELECT && view.dragging;

            if (view.dragging) { // if we were dragging, reset to default cursor
                view.setCursor('default');
                view.dragging = false;

                if (wasDragging) {
                    view.realDragging = false;
                
                    // call positionChanged one last time after dragging, with dragging: false
                    var posChangedFn = view.aladin.callbacksByEventName['positionChanged'];
                    if (typeof posChangedFn === 'function') {
                        var pos = view.aladin.pix2world(view.width/2, view.height/2);
                        if (pos !== undefined) {
                            posChangedFn({ra: pos[0], dec: pos[1], dragging: false});
                        }
                    }
                }
            } // end of "if (view.dragging) ... "

            if (selectionHasEnded) {
                view.aladin.fire('selectend', 
                                 view.getObjectsInBBox(view.selectStartCoo.x, view.selectStartCoo.y,
                                                       view.dragx-view.selectStartCoo.x, view.dragy-view.selectStartCoo.y));    

                view.requestRedraw();

                return;
            }

            view.mustClearCatalog = true;
            view.dragx = view.dragy = null;
            const xymouse = view.imageCanvas.relMouseCoords(e);

            if (e.type==="mouseout" || e.type==="touchend") {
                //view.requestRedraw();
                view.updateLocation(xymouse.x, xymouse.y, true);

                if (e.type==="mouseout") {
                    if (view.mode===View.TOOL_SIMBAD_POINTER) {
                        view.setMode(View.PAN);
                    }

                    return;
                }
            }

            if (view.mode==View.TOOL_SIMBAD_POINTER) {
                let radec = view.aladin.pix2world(xymouse.x, xymouse.y);

                // Convert from view to ICRSJ2000
                radec = view.aladin.webglAPI.viewToICRSJ2000CooSys(radec[0], radec[1]);

                view.setMode(View.PAN);
                view.setCursor('wait');
                if (radec) {
                    SimbadPointer.query(radec[0], radec[1], Math.min(1, 15 * view.fov / view.largestDim), view.aladin);
                } else {
                    console.log("Cannot unproject at the location you clicked on")
                }

                return; // when in TOOL_SIMBAD_POINTER mode, we do not call the listeners
            }

            // popup to show ?
            var objs = view.closestObjects(xymouse.x, xymouse.y, 5);
            if (! wasDragging && objs) {
                var o = objs[0];

                // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
                if (o instanceof Footprint || o instanceof Circle) {
                    o.dispatchClickEvent();
                }

                // display marker
                else if (o.marker) {
                    // could be factorized in Source.actionClicked
                    view.popup.setTitle(o.popupTitle);
                    view.popup.setText(o.popupDesc);
                    view.popup.setSource(o);
                    view.popup.show();
                }
                // show measurements
                else {
                    if (view.lastClickedObject) {
                        view.lastClickedObject.actionOtherObjectClicked && view.lastClickedObject.actionOtherObjectClicked();
                    }
                    o.actionClicked();
                }
                view.lastClickedObject = o;
                var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                (typeof objClickedFunction === 'function') && objClickedFunction(o);
            }
            else {
                if (view.lastClickedObject && ! wasDragging) {
                    view.aladin.measurementTable.hide();
                    view.popup.hide();

                    if (view.lastClickedObject instanceof Footprint) {
                        //view.lastClickedObject.deselect();
                    }
                    else {
                        view.lastClickedObject.actionOtherObjectClicked();
                    }

                    view.lastClickedObject = null;
                    var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                    (typeof objClickedFunction === 'function') && objClickedFunction(null);
                }
            }

            // call listener of 'click' event
            var onClickFunction = view.aladin.callbacksByEventName['click'];
            if (typeof onClickFunction === 'function') {
                var pos = view.aladin.pix2world(xymouse.x, xymouse.y);
                if (pos !== undefined) {
                    onClickFunction({ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y, isDragging: wasDragging});
                }
            }

            // TODO : remplacer par mecanisme de listeners
            // on avertit les catalogues progressifs
            view.refreshProgressiveCats();

            //view.requestRedraw();
            view.aladin.webglAPI.releaseLeftButtonMouse();
        });
        var lastHoveredObject; // save last object hovered by mouse
        var lastMouseMovePos = null;
        $(view.catalogCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();
            var xymouse = view.imageCanvas.relMouseCoords(e);

            if (view.rightClick && view.selectedSurveyLayer) {
                let selectedSurvey = view.imageSurveys.get(view.selectedSurveyLayer);
                if (!selectedSurvey.colored) {
                    // we try to match DS9 contrast adjustment behaviour with right click
                    const cs = {
                        x: view.catalogCanvas.clientWidth * 0.5,
                        y: view.catalogCanvas.clientHeight * 0.5,
                    };
                    const cx = (xymouse.x - cs.x) / view.catalogCanvas.clientWidth;
                    const cy = -(xymouse.y - cs.y) / view.catalogCanvas.clientHeight;

                    const offset = (cutMaxInit - cutMinInit) * cx;

                    const lr = offset + (1.0 - 2.0*cy)*cutMinInit;
                    const rr = offset + (1.0 + 2.0*cy)*cutMaxInit;
                    if (lr <= rr) {
                        selectedSurvey.setCuts([lr, rr])
                    }

                    return;
                }
            }

            if (e.type==='touchmove' && view.pinchZoomParameters.isPinching && e.originalEvent && e.originalEvent.touches && e.originalEvent.touches.length==2) {

                // rotation
                var currentFingerAngle = Math.atan2(e.originalEvent.targetTouches[1].clientY - e.originalEvent.targetTouches[0].clientY, e.originalEvent.targetTouches[1].clientX - e.originalEvent.targetTouches[0].clientX) * 180.0 / Math.PI;
                var fingerAngleDiff = view.fingersRotationParameters.initialFingerAngle - currentFingerAngle;
                // rotation is initiated when angle is equal or greater than 7 degrees
                if (! view.fingersRotationParameters.rotationInitiated && Math.abs(fingerAngleDiff)>=7) {
                    view.fingersRotationParameters.rotationInitiated = true;
                    view.fingersRotationParameters.initialFingerAngle = currentFingerAngle;
                    fingerAngleDiff = 0;
                }
                if (view.fingersRotationParameters.rotationInitiated) {
                    view.aladin.webglAPI.setRotationAroundCenter(fingerAngleDiff + view.fingersRotationParameters.initialViewAngleFromCenter);
                }

                // zoom
                const dist = Math.sqrt(Math.pow(e.originalEvent.touches[0].clientX - e.originalEvent.touches[1].clientX, 2) + Math.pow(e.originalEvent.touches[0].clientY - e.originalEvent.touches[1].clientY, 2));
                const fov = Math.min(Math.max(view.pinchZoomParameters.initialFov * view.pinchZoomParameters.initialDistance / dist, 0.00002777777), 360.0);
                view.setZoom(fov);

                return;
            }

            if (!view.dragging || hasTouchEvents) {
                // update location box
                view.updateLocation(xymouse.x, xymouse.y, false);
                // call listener of 'mouseMove' event
                var onMouseMoveFunction = view.aladin.callbacksByEventName['mouseMove'];
                if (typeof onMouseMoveFunction === 'function') {
                    var pos = view.aladin.pix2world(xymouse.x, xymouse.y);
                    if (pos !== undefined) {
                        onMouseMoveFunction({ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y});
                    }
                    // send null ra and dec when we go out of the "sky"
                    else if (lastMouseMovePos != null) {
                        onMouseMoveFunction({ra: null, dec: null, x: xymouse.x, y: xymouse.y});
                    }
                    lastMouseMovePos = pos;
                }


                if (!view.dragging && ! view.mode==View.SELECT) {
                    // objects under the mouse ?
                    var closest = view.closestObjects(xymouse.x, xymouse.y, 5);
                    if (closest) {
                        view.setCursor('pointer');
                        var objHoveredFunction = view.aladin.callbacksByEventName['objectHovered'];
                        if (typeof objHoveredFunction === 'function' && closest[0]!=lastHoveredObject) {
                            var ret = objHoveredFunction(closest[0]);
                        }
                        lastHoveredObject = closest[0];
        
                    }
                    else {
                        view.setCursor('default');
                        var objHoveredFunction = view.aladin.callbacksByEventName['objectHovered'];
                        if (typeof objHoveredFunction === 'function' && lastHoveredObject) {
                            lastHoveredObject = null;
                            // call callback function to notify we left the hovered object
                            var ret = objHoveredFunction(null);
                        }
                    }
                }
                if (!hasTouchEvents) {
                    return;
                }
            }

            if (! view.dragging) {
                return;
            }
            //var xoffset, yoffset;
            var s1, s2;
            if (e.originalEvent && e.originalEvent.targetTouches) {
                /*xoffset = e.originalEvent.targetTouches[0].clientX-view.dragx;
                yoffset = e.originalEvent.targetTouches[0].clientY-view.dragy;
                var xy1 = AladinUtils.viewToXy(e.originalEvent.targetTouches[0].clientX, e.originalEvent.targetTouches[0].clientY, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);*/
                s1 = {x: view.dragx, y: view.dragy};
                s2 = {x: e.originalEvent.targetTouches[0].clientX, y: e.originalEvent.targetTouches[0].clientY};
            }
            else {
                /*
                xoffset = e.clientX-view.dragx;
                yoffset = e.clientY-view.dragy;

                xoffset = xymouse.x-view.dragx;
                yoffset = xymouse.y-view.dragy;
                var xy1 = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);
                */
                //pos1 = view.projection.unproject(xy1.x, xy1.y);
                //pos2 = view.projection.unproject(xy2.x, xy2.y);

                /*pos1 = webglAPI.screenToWorld(view.dragx, view.dragy);
                pos2 = webglAPI.screenToWorld(xymouse.x, xymouse.y);

                if (pos2 == undefined)  {
                    return;
                }*/
                s1 = {x: view.dragx, y: view.dragy};
                s2 = {x: xymouse.x, y: xymouse.y};
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

            view.realDragging = true;

            //webglAPI.goFromTo(pos1[0], pos1[1], pos2[0], pos2[1]);
            view.aladin.webglAPI.goFromTo(s1.x, s1.y, s2.x, s2.y);
            //webglAPI.setCenter(pos2[0], pos2[1]);
            const [ra, dec] = view.aladin.webglAPI.getCenter();
            view.viewCenter.lon = ra;
            view.viewCenter.lat = dec;
            if (view.viewCenter.lon < 0.0) {
                view.viewCenter.lon += 360.0;
            }
        }); //// endof mousemove ////
        
        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }

        $(view.catalogCanvas).on('wheel', function(event) {            
            event.preventDefault();
            event.stopPropagation();

            if (view.rightClick) {
                return;
            }
            //var xymouse = view.imageCanvas.relMouseCoords(event);

            /*if(view.aladin.webglAPI.posOnUi()) {
                return;
            }*/
            //var xymouse = view.imageCanvas.relMouseCoords(event);
            //var level = view.zoomLevel;

            var delta = event.deltaY;
            // this seems to happen in context of Jupyter notebook --> we have to invert the direction of scroll
            // hope this won't trigger some side effects ...
            if (event.hasOwnProperty('originalEvent')) {
                delta = -event.originalEvent.deltaY;
            } 
            /*if (delta>0) {
                level += 1;
                //zoom
            }
            else {
                level -= 1;
                //unzoom
            }*/
            // The value of the field of view is determined
            // inside the backend
            if (delta > 0.0) {
                view.increaseZoom();
            } else {
                view.decreaseZoom();
            }

            if (!view.debounceProgCatOnZoom) {
                var self = view;
                view.debounceProgCatOnZoom = Utils.debounce(function() {
                    self.refreshProgressiveCats();
                    self.drawAllOverlays();
                }, 300);
            }
            view.debounceProgCatOnZoom();
            //view.setZoomLevel(level);
            //view.refreshProgressiveCats();

            return false;
        });
    };
    
    var init = function(view) {
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#aladin-statsDiv').length>0) {
            $('#aladin-statsDiv')[0].appendChild( stats.domElement );
        }
        
        view.stats = stats;

        createListeners(view);

        view.executeCallbacksThrottled = Utils.throttle(
            function() {
                var pos = view.aladin.pix2world(view.width/2, view.height/2);
                var fov = view.fov;
                if (pos===undefined || fov===undefined) {
                    return;
                }

                var ra = pos[0];
                var dec = pos[1];
                // trigger callback only if position has changed !
                if (ra!==this.ra || dec!==this.dec) {
                    var posChangedFn = view.aladin.callbacksByEventName['positionChanged'];
                    (typeof posChangedFn === 'function') && posChangedFn({ra: ra, dec: dec, dragging: true});
    
                    // finally, save ra and dec value
                    this.ra = ra;
                    this.dec = dec;
                }

                // trigger callback only if FoV (zoom) has changed !
                if (fov!==this.old_fov) {
                    var fovChangedFn = view.aladin.callbacksByEventName['zoomChanged'];
                    (typeof fovChangedFn === 'function') && fovChangedFn(fov);
    
                    // finally, save fov value
                    this.old_fov = fov;
                }

            },
            View.CALLBACKS_THROTTLE_TIME_MS);


        view.displayHpxGrid = false;
        view.displaySurvey = true;
        view.displayCatalog = false;
        view.displayReticle = true;

        // initial draw
        //view.fov = computeFov(view);
        //updateFovDiv(view);
        //view.redraw();
    };

    View.prototype.updateLocation = function(mouseX, mouseY, isViewCenterPosition) {
        if (!this.projection) {
            return;
        }

        if(isViewCenterPosition) {
            //const [ra, dec] = this.aladin.webglAPI.ICRSJ2000ToViewCooSys(this.viewCenter.lon, this.viewCenter.lat);
            this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);
        } else {
            let radec = this.aladin.webglAPI.screenToWorld(mouseX, mouseY); // This is given in the frame of the view
            if (radec) {
                if (radec[0]<0) {
                    radec = [radec[0] + 360.0, radec[1]];
                }

                this.location.update(radec[0], radec[1], this.cooFrame, false);
            }
        }
    }
    
    View.prototype.requestRedrawAtDate = function(date) {
        this.dateRequestDraw = date;
    };

    /**
     * Return the color of the lowest intensity pixel 
     * in teh current color map of the current background image HiPS
     */
    View.prototype.getBackgroundColor = function() {
        var white = 'rgb(255, 255, 255)';
        var black = 'rgb(0, 0, 0)';

        if (! this.imageSurvey) {
            return black;
        }

        var cm = this.imageSurvey.getColorMap();
        if (!cm) {
            return black;
        }
        if (cm.mapName == 'native' || cm.mapName == 'grayscale') {
            return cm.reversed ? white : black;
        }

        var idx = cm.reversed ? 255 : 0;
        var r = ColorMap.MAPS[cm.mapName].r[idx];
        var g = ColorMap.MAPS[cm.mapName].g[idx];
        var b = ColorMap.MAPS[cm.mapName].b[idx];

        return 'rgb(' + r + ',' + g + ',' + b + ')';
    };

    View.prototype.getViewParams = function() {
        var resolution = this.width > this.height ? this.fov / this.width : this.fov / this.height;
        return {
            fov: [this.width * resolution, this.height * resolution],   
            width: this.width,   
            height: this.height   
        };
    };

    /**
     * redraw the whole view
     */
    View.prototype.redraw = function() {
        // calc elapsed time since last loop
        // Put your drawing code here
        try {
            //var dt = now_update - this.prev;
            this.aladin.webglAPI.update(Date.now() - this.then);
        } catch(e) {
            console.error(e)
        }

        // check whether a catalog has been parsed and
        // is ready to be plot
        let catReady = this.aladin.webglAPI.isCatalogLoaded();
        if (catReady) {
            var callbackFn = this.aladin.callbacksByEventName['catalogReady'];
            (typeof callbackFn === 'function') && callbackFn();
        }

        try {
            this.aladin.webglAPI.render(this.needRedraw);
        } catch(e) {
            console.error("Error: ", e);
        }

        ////// 2. Draw catalogues////////
        const isViewRendering = this.aladin.webglAPI.isRendering();
        if (isViewRendering || this.needRedraw) {
            this.drawAllOverlays();
        }
        this.needRedraw = false;

        // objects lookup
        if (!this.dragging) {
            this.updateObjectsLookup();
        }

        this.then = Date.now();
        // request another frame
        requestAnimFrame(this.redraw.bind(this));
    };

    View.prototype.drawAllOverlays = function() {
        var catalogCtx = this.catalogCtx;
        var catalogCanvasCleared = false;
        if (this.mustClearCatalog) {
            catalogCtx.clearRect(0, 0, this.width, this.height);
            catalogCanvasCleared = true;
            this.mustClearCatalog = false;
        }

        if (this.catalogs && this.catalogs.length>0 && this.displayCatalog && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            // TODO : do not clear every time
            //// clear canvas ////
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            for (var i=0; i<this.catalogs.length; i++) {
                var cat = this.catalogs[i];
                cat.draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
            }
        }
        // draw popup catalog
        if (this.catalogForPopup.isShowing && this.catalogForPopup.sources.length>0) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            this.catalogForPopup.draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
        }

        ////// 3. Draw overlays////////
        var overlayCtx = this.catalogCtx;
        if (this.overlays && this.overlays.length>0 && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            for (var i=0; i<this.overlays.length; i++) {
                this.overlays[i].draw(overlayCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
            }
        }
        
        // Redraw HEALPix grid
        var healpixGridCtx = catalogCtx;
        if( this.displayHpxGrid ) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

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
            if (cornersXYViewMapHighres && this.curNorder>3) {
                this.healpixGrid.redraw(healpixGridCtx, cornersXYViewMapHighres, this.fov, this.curNorder);
            }
            else {
                this.healpixGrid.redraw(healpixGridCtx, cornersXYViewMapAllsky, this.fov, 3);
            }
        }


        // draw MOCs
        var mocCtx = catalogCtx;
        if (this.mocs && this.mocs.length>0 && (! this.dragging  || View.DRAW_MOCS_WHILE_DRAGGING)) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            for (var i=0; i<this.mocs.length; i++) {
                this.mocs[i].draw(mocCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.fov);
            }
        }

        ////// 4. Draw reticle ///////
        // TODO: reticle should be placed in a static DIV, no need to waste a canvas
        var reticleCtx = catalogCtx;
        if (this.mode==View.SELECT) {
            // VIEW mode, we do not want to display the reticle in this
            // but draw a selection box
            if (this.dragging) {
                if (! catalogCanvasCleared) {
                    reticleCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }

                reticleCtx.fillStyle = "rgba(100, 240, 110, 0.25)";
                var w = this.dragx - this.selectStartCoo.x;
                var h =  this.dragy - this.selectStartCoo.y;
                
                reticleCtx.fillRect(this.selectStartCoo.x, this.selectStartCoo.y, w, h);
            }
        } else {
            // Normal modes
            if (this.displayReticle) {
                if (! catalogCanvasCleared) {
                    catalogCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }
    
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
            }
        }  
        
        ////// 5. Draw all-sky ring /////
        if (this.projection.PROJECTION == ProjectionEnum.SIN && this.fov>=60 && this.aladin.options['showAllskyRing'] === true) {
            if (! catalogCanvasCleared) {
                reticleCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            reticleCtx.strokeStyle = this.aladin.options['allskyRingColor'];
            var ringWidth = this.aladin.options['allskyRingWidth'];
            reticleCtx.lineWidth = ringWidth;
            reticleCtx.beginPath();
            var maxCxCy = this.cx>this.cy ? this.cx : this.cy;
            reticleCtx.arc(this.cx, this.cy, (maxCxCy-(ringWidth/2.0)+1) / this.zoomFactor, 0, 2*Math.PI, true);
            reticleCtx.stroke();
        }
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

    View.prototype.getVisiblePixList = function(norder) {
        var pixList = [];
        let centerWorldPosition = this.aladin.webglAPI.screenToWorld(this.cx, this.cy);
        const [lon, lat] = this.aladin.webglAPI.viewToICRSJ2000CooSys(centerWorldPosition[0], centerWorldPosition[1]);

        var radius = this.fov*0.5*this.ratio;
        this.aladin.webglAPI.queryDisc(norder, lon, lat, radius).forEach(x => pixList.push(Number(x)));
        
        return pixList;
    };
    
    View.prototype.setAngleRotation = function(theta) {

    }

    // TODO: optimize this method !!
    View.prototype.getVisibleCells = function(norder) {
        var cells = []; // array to be returned
        var cornersXY = [];
        var nside = Math.pow(2, norder); // TODO : to be modified
        var npix = 12 * nside * nside;
        var ipixCenter = null;
        
        // build list of pixels
        var pixList = this.getVisiblePixList(norder)
        var ipix;
        var lon, lat;
        var corners;
        for (var ipixIdx=0, len=pixList.length; ipixIdx<len; ipixIdx++) {
            ipix = pixList[ipixIdx];
            if (ipix==ipixCenter && ipixIdx>0) { 
                continue;
            }
            var cornersXYView = [];
            //corners = HealpixCache.corners_nest(ipix, nside);
            corners = this.aladin.webglAPI.hpxNestedVertices(Math.log2(nside), ipix);

            for (var k=0; k<4; k++) {
                const lon = corners[k*2];
                const lat = corners[k*2 + 1];
                cornersXY[k] = this.aladin.webglAPI.worldToScreen(lon, lat);
            }

            if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
                continue;
            }

            for (var k=0; k<4; k++) {
                //cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
                cornersXYView[k] = {
                    vx: cornersXY[k][0],
                    vy: cornersXY[k][1],
                };
            }

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
//            if (this.fov<160) { // don't bother checking if fov is large enough
//                if ( ! AladinUtils.isHpxPixVisible(cornersXYView, this.width, this.height) ) {
//                    continue;
//                }
//            }
            // check if we have a pixel at the edge of the view in allsky projections
            //if (this.projection.PROJECTION!=ProjectionEnum.SIN && this.projection.PROJECTION!=ProjectionEnum.TAN) {
                /*var xdiff = cornersXYView[0].vx-cornersXYView[2].vx;
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
                }*/

                // New faster approach: when a vertex from a cell gets to the other side of the projection
                // its vertices order change from counter-clockwise to clockwise!
                // So if the vertices describing a cell are given in clockwise order
                // we know it crosses the projection, so we do not plot them!
                if (!AladinUtils.counterClockwiseTriangle(cornersXYView[0].vx, cornersXYView[0].vy, cornersXYView[1].vx, cornersXYView[1].vy, cornersXYView[2].vx, cornersXYView[2].vy) ||
                    !AladinUtils.counterClockwiseTriangle(cornersXYView[0].vx, cornersXYView[0].vy, cornersXYView[2].vx, cornersXYView[2].vy, cornersXYView[3].vx, cornersXYView[3].vy)) {
                    continue;
                }
            //}

            if (this.projection.PROJECTION == ProjectionEnum.HPX) {
                const triIdxInCollignonZone = ((p) => {
                    const x = ((p.vx / this.catalogCanvas.clientWidth) - 0.5) * this.zoomFactor;
                    const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;

                    const xZone = Math.floor((x + 0.5) * 4);
                    return xZone + 4 * (y > 0.0);
                });

                const isInCollignon = ((p) => {
                    const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;

                    return y < -0.25 || y > 0.25;
                });

                if (isInCollignon(cornersXYView[0]) && isInCollignon(cornersXYView[1]) && isInCollignon(cornersXYView[2]) && isInCollignon(cornersXYView[3])) {
                    const allVerticesInSameCollignonRegion = (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[1])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[2])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[3]));
                    if (!allVerticesInSameCollignonRegion) {
                        continue;
                    }
                }
            }
            
            cornersXYView.ipix = ipix;
            cells.push(cornersXYView);
        }
        
        return cells;
    };
    
    /*View.prototype.computeZoomFactor = function(level) {
        if (level>0) {
            return AladinUtils.getZoomFactorForAngle(180.0/Math.pow(1.35, level), this.projectionMethod);
        }
        else {
            return 1 + 0.1*level;
        }
    };*/
    /*View.prototype.computeZoomLevelFromFOV = function() {
        if (level>0) {
            return AladinUtils.getZoomFactorForAngle(180/Math.pow(1.15, level), this.projectionMethod);
        }
        else {
            return 1 + 0.1*level;
        }
    };*/
    
    // Called for touchmove events
    // initialAccDelta must be consistent with fovDegrees here
    View.prototype.setZoom = function(fovDegrees) {
        const si = 500000.0;
        const alpha = 40.0;
        this.pinchZoomParameters.initialAccDelta = Math.pow(si / fovDegrees, 1.0/alpha);
        /*if (fovDegrees<0) {
            return;
        }*/
        //const si = 500000.0;
        //const alpha = 40.0;

        // Erase the field of view state of the backend by
        this.aladin.webglAPI.setFieldOfView(fovDegrees);
        //var zoomLevel = Math.log(180/fovDegrees)/Math.log(1.15);
        //this.setZoomLevel(zoomLevel);
        this.updateZoomState();
        this.updateFovDiv();
    };

    View.prototype.increaseZoom = function() {
        const si = 500000.0;
        const alpha = 40.0;
        const amount = 0.005;

        this.pinchZoomParameters.initialAccDelta += amount;

        if (this.pinchZoomParameters.initialAccDelta <= 0.0) {
            this.pinchZoomParameters.initialAccDelta = 1e-3;
        }
        let new_fov = si / Math.pow(this.pinchZoomParameters.initialAccDelta, alpha);

        if (new_fov > 360.0) {
            new_fov = 360.0;
            //this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 
        if (new_fov < 0.00002777777) {
            new_fov = 0.00002777777;
            //this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        }

        this.setZoom(new_fov);
    }

    View.prototype.decreaseZoom = function() {
        const si = 500000.0;
        const alpha = 40.0;
        const amount = 0.005;

        this.pinchZoomParameters.initialAccDelta -= amount;

        if (this.pinchZoomParameters.initialAccDelta <= 0.0) {
            this.pinchZoomParameters.initialAccDelta = 1e-3;
        }

        let new_fov = si / Math.pow(this.pinchZoomParameters.initialAccDelta, alpha);
        if (new_fov > 360.0) {
            new_fov = 360.0;
            //this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 
        if (new_fov < 0.00002777777) {
            new_fov = 0.00002777777;
            //this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 

        this.setZoom(new_fov);
    }

    View.prototype.setGridConfig = function(gridCfg) {
        this.aladin.webglAPI.setGridConfig(gridCfg);

        // send events
        if (gridCfg) {
            if (gridCfg.hasOwnProperty('enabled')) {
                this.showCooGrid = true;

                if (gridCfg.enabled === true) {
                    ALEvent.COO_GRID_ENABLED.dispatchedTo(this.aladinDiv);
                }
                else {
                    ALEvent.COO_GRID_DISABLED.dispatchedTo(this.aladinDiv);
                }
            }
            if (gridCfg.color) {
                ALEvent.COO_GRID_UPDATED.dispatchedTo(this.aladinDiv, {color: gridCfg.color});
            }
        }
        this.requestRedraw();
    };

    View.prototype.updateZoomState = function() {
        this.zoomFactor = this.aladin.webglAPI.getClipZoomFactor();
        this.fov = this.aladin.webglAPI.getFieldOfView();
        
        this.computeNorder();
    };
    
    /**
     * compute and set the norder corresponding to the current view resolution
     */
    View.prototype.computeNorder = function() {
        /*var resolution = this.fov / this.largestDim; // in degree/pixel
        var tileSize = 512; // TODO : read info from HpxImageSurvey.tileSize
        const calculateNSide = (pixsize) => {
            const NS_MAX = 536870912;
            const ORDER_MAX = 29;
        
            // Available nsides ..always power of 2 ..
            const NSIDELIST = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048,
                4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288,
                                       1048576, 2097152, 4194304, 8388608, 16777216, 33554432,
                                       67108864, 134217728,  268435456, 536870912];

            let res = 0;
            const pixelArea = pixsize * pixsize;
            const degrad = 180. / Math.PI;
            const skyArea = 4. * Math.PI * degrad * degrad * 3600. * 3600.;
            const castToInt = function (x) {
                if (x > 0) {
                    return Math.floor(x);
                }
                else {
                    return Math.ceil(x);
                }
            };
            const npixels = castToInt(skyArea / pixelArea);
            const nsidesq = npixels / 12;
            const nside_req = Math.sqrt(nsidesq);
            var mindiff = NS_MAX;
            var indmin = 0;
            for (var i = 0; i < NSIDELIST.length; i++) {
                if (Math.abs(nside_req - NSIDELIST[i]) <= mindiff) {
                    mindiff = Math.abs(nside_req - NSIDELIST[i]);
                    res = NSIDELIST[i];
                    indmin = i;
                }
                if ((nside_req > res) && (nside_req < NS_MAX))
                    res = NSIDELIST[indmin + 1];
                if (nside_req > NS_MAX) {
                    console.log("nside cannot be bigger than " + NS_MAX);
                    return NS_MAX;
                }
    
            }
            return res;
        };*/

        //var nside = calculateNSide(3600*tileSize*resolution); // 512 = size of a "tile" image
        //var norder = Math.log(nside)/Math.log(2);
        //norder = Math.max(norder, 1);

        var norder = this.aladin.webglAPI.getNOrder();

        this.realNorder = norder;
        // here, we force norder to 3 (otherwise, the display is "blurry" for too long when zooming in)
        if (this.fov<=50 && norder<=2) {
            norder = 3;
        }

        // that happens if we do not wish to display tiles coming from Allsky.[jpg|png]
        if (this.imageSurvey && norder<=2 && this.imageSurvey.minOrder>2) {
            norder = this.imageSurvey.minOrder;
        }

        if (this.imageSurvey && norder>this.imageSurvey.maxOrder) {
            norder = this.imageSurvey.maxOrder;
        }

        // should never happen, as calculateNSide will return something <=HealpixIndex.ORDER_MAX
        if (norder>29) {
            norder = 29;
        }
            
        this.curNorder = norder;
    };

    View.prototype.untaintCanvases = function() {
        this.createCanvases();
        createListeners(this);
        this.fixLayoutDimensions();
    };

    View.prototype.setBaseImageLayer = function(baseSurveyPromise) {
        this.setOverlayImageSurvey(baseSurveyPromise, "base");
    };

    View.prototype.setOverlayImageSurvey = function(survey, layer = "overlay") {
        const surveyIdx = this.imageSurveysIdx.get(layer) || 0;
        const newSurveyIdx = surveyIdx + 1;
        this.imageSurveysIdx.set(layer, newSurveyIdx);
        survey.orderIdx = newSurveyIdx;

        // Check whether this layer already exist
        const idxOverlaySurveyFound = this.overlayLayers.findIndex(overlayLayer => overlayLayer == layer);

        if (idxOverlaySurveyFound == -1) {
            if (layer === "base") {
                // insert at the beginning
                this.overlayLayers.splice(0, 0, layer);
            } else {
                this.overlayLayers.push(layer);
            }
        } else {
            // find the survey by layer and erase it by the new value
            this.overlayLayers[ idxOverlaySurveyFound ] = layer;
        }

        /// async part
        if (this.options.log && survey.properties) {
            Logger.log("setImageLayer", survey.properties.url);
        }

        survey.added = true;
        survey.layer = layer;
        survey.existedBefore = false;

        const pastSurvey = this.imageSurveys.get(layer);
        if (pastSurvey && pastSurvey.ready && pastSurvey.added) {
            survey.existedBefore = true;
        }

        this.imageSurveys.set(layer, survey);

        if (survey.ready) {
            this.commitSurveysToBackend(survey, layer);
        }
    };

    View.prototype.buildSortedImageSurveys = function() {
        let sortedImageSurveys = [];

        this.overlayLayers.forEach((overlaidLayer) => {
            sortedImageSurveys.push(
                this.imageSurveys.get(overlaidLayer)
            );
        });

        return sortedImageSurveys;
    }

    View.prototype.updateImageLayerStack = function() {
        try {
            let surveys = this.buildSortedImageSurveys()
                .filter(s => s !== undefined && s.properties )
                .map(s => {
                    //let {backend, ...survey} = s;
                    //return survey;
                    return {
                        layer: s.layer,
                        properties: s.properties,
                        meta: s.meta,
                        // rust accepts it in upper case whereas the js API handles 'jpeg', 'png' or 'fits' in lower case
                        imgFormat: s.options.imgFormat.toUpperCase(),
                    };
                });
            this.aladin.webglAPI.setImageSurveys(surveys);
        } catch(e) {
            console.error(e)
        }
    };

    View.prototype.removeImageSurvey = function(layer) {
        this.imageSurveys.delete(layer);

        const idxOverlaidSurveyFound = this.overlayLayers.findIndex(overlaidLayer => overlaidLayer == layer);
        if (idxOverlaidSurveyFound == -1) {
            // layer not found
            return;
        }

        // Remove it from the layer stack
        this.overlayLayers.splice(idxOverlaidSurveyFound, 1);

        // Update the backend
        this.updateImageLayerStack();

        if (this.selectedSurveyLayer === layer) {
            this.selectedSurveyLayer = null;
        }

        ALEvent.HIPS_LAYER_REMOVED.dispatchedTo(this.aladinDiv, {layer: layer});
    };

    View.prototype.commitSurveysToBackend = function(survey, layer = "base") {
        //const layerAlreadyContained = this.imageSurveys.has(layer); true

        try {
            this.updateImageLayerStack();

            if (survey.existedBefore) {
                //if (this.selectedSurveyLayer && this.selectedSurveyLayer === layer) {
                //    this.selectedSurveyLayer = layer;
                //}
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.aladinDiv, {survey: survey});
            } else {
                survey.existedBefore = true;
                ALEvent.HIPS_LAYER_ADDED.dispatchedTo(this.aladinDiv, {survey: survey});
            }
        } catch(e) {
            // En error occured while loading the HiPS
            // Remove it from the View
            // - First, from the image dict
            this.imageSurveys.delete(layer);
            
            // Tell the survey object that it is not linked to the view anymore
            survey.added = false;

            // Finally delete the layer
            const idxOverlaidSurveyFound = this.overlayLayers.findIndex(overlaidLayer => overlaidLayer == layer);
            if (idxOverlaidSurveyFound >= 0) {    
                // Remove it from the layer stack
                this.overlayLayers.splice(idxOverlaidSurveyFound, 1);
            }

            throw 'Error loading the HiPS ' + survey + ':' + e;
        }
    }

    View.prototype.getImageSurvey = function(layer = "base") {
        const survey = this.imageSurveys.get(layer);
        return survey;
    };

    View.prototype.getImageSurveyMeta = function(layer = "base") {
        try {
            return this.aladin.webglAPI.getImageSurveyMeta(layer);
        } catch(e) {
            console.error(e);
        }
    };

    View.prototype.setImageSurveyMeta = function(layer = "base", meta) {
        try {
            this.aladin.webglAPI.setImageSurveyMeta(layer, meta);
        } catch(e) {
            console.error(e);
        }
    };

    /*View.prototype.setImageSurveysLayer = function(surveys, layer) {
        this.imageSurveys.set(layer, new Map());

        surveys.forEach(survey => {
            const url = survey.properties.url;
            survey.layer = layer;
            
            this.imageSurveys.get(layer).set(url, survey);
        });

        // Then we send the current surveys to the backend
        this.setHiPS();
    };*/

    /*View.prototype.removeImageSurveysLayer = function (layer) {
        this.imageSurveys.delete(layer);

        this.setHiPS();
    };*/

    /*View.prototype.moveImageSurveysLayerForward = function(layer) {
        this.aladin.webglAPI.moveImageSurveysLayerForward(layer);
    }*/

    View.prototype.requestRedraw = function() {
        this.needRedraw = true;
    };
    
    View.prototype.setProjection = function(projectionName) {
        switch (projectionName) {
            case "AIT":
                this.projection.setProjection(ProjectionEnum.AITOFF);
                //this.projectionMethod = ProjectionEnum.AITOFF;
                break;
            case "HPX":
                this.projection.setProjection(ProjectionEnum.HPX);
                //this.projectionMethod = ProjectionEnum.HPX;
                break;
            case "TAN":
                this.projection.setProjection(ProjectionEnum.TAN);
                //this.projectionMethod = ProjectionEnum.TAN;
                break;
            case "ARC":
                this.projection.setProjection(ProjectionEnum.ARC);
                //this.projectionMethod = ProjectionEnum.ARC;
                break;
            case "MER":
                this.projection.setProjection(ProjectionEnum.MERCATOR);
                //this.projectionMethod = ProjectionEnum.MERCATOR;
                break;
            case "MOL":
                this.projection.setProjection(ProjectionEnum.MOL);
                //this.projectionMethod = ProjectionEnum.MOL;
                break;
            case "SIN":
            default:
                this.projection.setProjection(ProjectionEnum.SIN);
                //this.projectionMethod = ProjectionEnum.SIN;
        }
        // Change the projection here
        this.aladin.webglAPI = this.aladin.webglAPI.setProjection(projectionName, this.width, this.height);

        this.requestRedraw();
    };

    View.prototype.changeFrame = function(cooFrame) {
        this.cooFrame = cooFrame;

        // Set the new frame to the backend
        if (this.cooFrame.system == CooFrameEnum.SYSTEMS.GAL) {
            this.aladin.webglAPI.setCooSystem(Aladin.wasmLibs.webgl.CooSystem.GAL);
        }
        else if (this.cooFrame.system == CooFrameEnum.SYSTEMS.J2000) {
            this.aladin.webglAPI.setCooSystem(Aladin.wasmLibs.webgl.CooSystem.ICRSJ2000);
        }

        // Get the new view center position (given in icrsj2000)
        let [ra, dec] = this.aladin.webglAPI.getCenter();
        this.viewCenter.lon = ra;
        this.viewCenter.lat = dec;
        if (this.viewCenter.lon < 0.0) {
            this.viewCenter.lon += 360.0;
        }
        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        this.requestRedraw();
    };

    View.prototype.showHealpixGrid = function(show) {
        this.displayHpxGrid = show;

        if (!this.displayHpxGrid) {
            this.mustClearCatalog = true;
        }

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

        if (!this.displayReticle) {
            this.mustClearCatalog = true;
        }

        this.requestRedraw();
    };

    /**
     * 
     * @API Point to a specific location in ICRSJ2000
     * 
     * @param ra ra expressed in ICRS J2000 frame
     * @param dec dec expressed in ICRS J2000 frame
     * @param options
     *   
     */
    View.prototype.pointTo = function(ra, dec, options) {
        options = options || {};
        ra = parseFloat(ra);
        dec = parseFloat(dec);

        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        this.viewCenter.lon = ra;
        this.viewCenter.lat = dec;
        if (this.viewCenter.lon < 0.0) {
            this.viewCenter.lon += 360.0;
        }
        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        // Put a javascript code here to do some animation
        //this.projection.setCenter(this.viewCenter.lon, this.viewCenter.lat);
        this.aladin.webglAPI.setCenter(this.viewCenter.lon, this.viewCenter.lat);
        
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
        var c = this.allOverlayLayers;
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
        this.mocs = [];
        this.allOverlayLayers = [];
        this.requestRedraw();
    };

    View.prototype.removeLayer = function(layer) {
        let indexToDelete = this.allOverlayLayers.indexOf(layer);
        this.allOverlayLayers.splice(indexToDelete, 1);

        if (layer.type=='catalog' || layer.type=='progressivecat') {
            indexToDelete = this.catalogs.indexOf(layer);
            this.catalogs.splice(indexToDelete, 1);
        }
        else if (layer.type=='moc') {
            indexToDelete = this.mocs.indexOf(layer);
            this.mocs.splice(indexToDelete, 1);
        }
        else if (layer.type=='overlay') {
            indexToDelete = this.overlays.indexOf(layer);
            this.overlays.splice(indexToDelete, 1);
        }

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.dispatchedTo(this.aladinDiv, {layer: layer});

        this.requestRedraw();
    };

    View.prototype.addCatalog = function(catalog) {
        catalog.name = this.makeUniqLayerName(catalog.name);
        this.allOverlayLayers.push(catalog);
        this.catalogs.push(catalog);
        if (catalog.type=='catalog') {
            catalog.setView(this);
        }
        else if (catalog.type=='progressivecat') {
            catalog.init(this);
        }
    };
    View.prototype.addOverlay = function(overlay) {
        overlay.name = this.makeUniqLayerName(overlay.name);
        this.overlays.push(overlay);
        this.allOverlayLayers.push(overlay);
        overlay.setView(this);
    };
    
    View.prototype.addMOC = function(moc) {
        moc.name = this.makeUniqLayerName(moc.name);
        this.mocs.push(moc);
        this.allOverlayLayers.push(moc);
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

        var cat, sources, s, xRounded, yRounded;
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

                    xRounded = Math.round(s.x);
                    yRounded = Math.round(s.y);

                    if (typeof this.objLookup[xRounded] === 'undefined') {
                        this.objLookup[xRounded] = [];
                    }
                    if (typeof this.objLookup[xRounded][yRounded] === 'undefined') {
                        this.objLookup[xRounded][yRounded] = [];
                    }
                    this.objLookup[xRounded][yRounded].push(s);
                }       
            }           
        }     
    };

    // return closest object within a radius of maxRadius pixels. maxRadius is an integer
    View.prototype.closestObjects = function(x, y, maxRadius) {

        // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
        var overlay;
        var canvas=this.catalogCanvas;
        var ctx = canvas.getContext("2d");
        // this makes footprint selection easier as the catch-zone is larger
        ctx.lineWidth = 6;

        if (this.overlays) {
            for (var k=0; k<this.overlays.length; k++) {
                overlay = this.overlays[k];
                for (var i=0; i<overlay.overlays.length;i++){

                    // test polygons first
                    var footprint = overlay.overlays[i];
                    var pointXY = [];
                    for(var j=0;j<footprint.polygons.length;j++){

                        /*var xy = AladinUtils.radecToViewXy(footprint.polygons[j][0], footprint.polygons[j][1],
                                this.projection,
                                this.cooFrame,
                                this.width, this.height,
                                this.largestDim,
                                this.zoomFactor);*/
                        var xy = AladinUtils.radecToViewXy(footprint.polygons[j][0], footprint.polygons[j][1], this);
                        if (! xy) {
                            continue;
                        }
                        pointXY.push({
                            x: xy[0],
                            y: xy[1]
                        });
                    }
                    for(var l=0; l<pointXY.length-1;l++){

                        ctx.beginPath();                        // new segment
                        ctx.moveTo(pointXY[l].x, pointXY[l].y);     // start is current point
                        ctx.lineTo(pointXY[l+1].x, pointXY[l+1].y); // end point is next
                        if (ctx.isPointInStroke(x, y)) {        // x,y is on line?
                            closest = footprint;
                            return [closest];
                        }
                    }
                }

                // test Circles
                for (var i=0; i<overlay.overlay_items.length; i++) {
                    if (overlay.overlay_items[i] instanceof Circle) {
                        overlay.overlay_items[i].draw(ctx, this, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, true);

                        if (ctx.isPointInStroke(x, y)) {
                            closest = overlay.overlay_items[i];
                            return [closest];
                        }
                    }
                }
            }
        }






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
                        var d = dx*dx + dy*dy;
                        if (!closest) {
                            closest = this.objLookup[x+dx][y+dy];
                            dist = d;
                        }
                        else if (d<dist) {
                            dist = d;
                            closest = this.objLookup[x+dx][y+dy];
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
