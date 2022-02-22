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
import { HpxImageSurvey } from "./HpxImageSurvey.js";
import { ProjectionEnum } from "./ProjectionEnum.js";
import { Projection }     from "./libs/astro/projection.js";
import { Coo }            from "./libs/astro/coo.js";
import { AladinUtils }    from "./AladinUtils.js";
import { HealpixIndex }   from "./libs/healpix.js";
import { HealpixCache }   from "./HealpixCache.js";
import { SpatialVector }  from "./libs/healpix.js";
import { Utils }          from "./Utils.js";
import { SimbadPointer }  from "./SimbadPointer.js";
import { TileBuffer }     from "./TileBuffer.js";
import { Downloader }     from "./Downloader.js";
import { Stats }          from "./libs/Stats.js";
import { ColorMap } from "./ColorMap.js";
import { Footprint } from "./Footprint.js";
import { Circle } from "./Circle.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { CooConversion } from "./CooConversion.js";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";
import { ImageSurveyLayer } from "./ImageSurveyLayer.js";
import { WebGLCtx } from "./WebGL.js";

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
                console.log(e)
                // 2. Add a more explicite message to the end user
                alert("Problem initializing Aladin Lite. Please contact the support by contacting Matthieu Baumann (baumannmatthieu0@gmail.com) or Thomas Boch (thomas.boch@astro.unistra.fr). You can also open an issue on the Aladin Lite github repository here: https://github.com/cds-astro/aladin-lite")
            }

            this.location = location;
            this.fovDiv = fovDiv;
            this.mustClearCatalog = true;
            this.mustRedrawReticle = true;
            this.imageSurveysToSet = [];
            this.mode = View.PAN;
            
            this.minFOV = this.maxFOV = null; // by default, no restriction
            this.fov_limit = 180.0;
            
            this.healpixGrid = new HealpixGrid(this.imageCanvas);
            this.then = Date.now();
            
            var lon, lat;
            lon = lat = 0;
            
            this.projectionMethod = ProjectionEnum.SIN;
            this.projection = new Projection(lon, lat);
            this.projection.setProjection(this.projectionMethod);
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
            if (cooFrame.system === CooFrameEnum.SYSTEMS.GAL) {
                console.log()
                const GAL = Aladin.wasmLibs.webgl.GALCooSys();
                this.aladin.webglAPI.setCooSystem(GAL);
            } else {
                const ICRSJ2000 = Aladin.wasmLibs.webgl.ICRSJ2000CooSys();
                this.aladin.webglAPI.setCooSystem(ICRSJ2000);
            }

            if (zoom) {
                this.setZoom(zoom);
            }
            
            // current reference image survey displayed
            this.imageSurveys = new Map();
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
            
    
            
            this.tileBuffer = new TileBuffer(); // tile buffer is shared across different image surveys
            this.fixLayoutDimensions();
            
            this.firstHiPS = true;
            this.curNorder = 1;
            this.realNorder = 1;
            this.curOverlayNorder = 1;
            
            // some variables for mouse handling
            this.dragging = false;
            this.dragx = null;
            this.dragy = null;
            this.needRedraw = true;

            const si = 500000.0;
            const alpha = 40.0;
            // zoom pinching
            this.pinchZoomParameters = {
                isPinching: false, // true if a pinch zoom is ongoing
                initialFov: undefined,
                initialDistance: undefined,
                initialAccDelta: Math.pow(si / 180.0, 1.0/alpha)
            };

            // two-fingers rotation
            this.fingersRotationParameters = {
                initialViewAngleFromCenter: undefined,
                initialFingerAngle: undefined,
                rotationInitiated: false
            }
    
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
                    
                    self.updateZoomState(); // needed to force recomputation of displayed FoV
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


        //a.find('.aladin-webglCanvas').remove();
        a.find('.aladin-imageCanvas').remove();
        a.find('.aladin-catalogCanvas').remove();
        a.find('.aladin-reticleCanvas').remove();
        a.find('.aladin-gridCanvas').remove();

        // canvas to draw the images
        //this.webglCanvas = $("<canvas class='aladin-webglCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the overlays
                // canvas to draw the gui
                //this.guiCanvas = $("<canvas id='aladin-guiCanvas' style={width: 200px}≈></canvas>").appendTo(this.aladinDiv)[0];
        this.imageCanvas = $("<canvas class='aladin-imageCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the grid
        this.gridCanvas = $("<canvas class='aladin-gridCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the catalogs
        this.catalogCanvas = $("<canvas class='aladin-catalogCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the reticle
        this.reticleCanvas = $("<canvas class='aladin-reticleCanvas'></canvas>").appendTo(this.aladinDiv)[0];
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
        this.reticleCtx = this.reticleCanvas.getContext("2d");
        this.gridCtx = this.gridCanvas.getContext("2d");
        //this.guiCtx = this.guiCanvas.getContext("webgl2");

        //this.imageCtx.canvas.width = this.width;
        this.catalogCtx.canvas.width = this.width;
        this.reticleCtx.canvas.width = this.width;
        this.gridCtx.canvas.width = this.width;
        
        //this.imageCtx.canvas.height = this.height;
        this.catalogCtx.canvas.height = this.height;
        this.reticleCtx.canvas.height = this.height;
        this.gridCtx.canvas.height = this.height;

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
        //this.requestRedraw();
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
            this.reticleCanvas.style.cursor = '';
            $(this.reticleCanvas).addClass('aladin-sp-cursor');
        }
        else {
            this.setCursor('default');
        }
    };
    
    View.prototype.setCursor = function(cursor) {
        if (this.reticleCanvas.style.cursor==cursor) {
            return;
        }
        if (this.mode==View.TOOL_SIMBAD_POINTER) {
            return;
        }
        this.reticleCanvas.style.cursor = cursor;
    };

    
    
    /**
     * return dataURL string corresponding to the current view
     */
    View.prototype.getCanvasDataURL = function(imgType, width, height) {
        imgType = imgType || "image/png"; 
        var c = document.createElement('canvas');
        width = width || this.width;
        height = height || this.height;
        c.width = width;
        c.height = height;
        var ctx = c.getContext('2d');

        //ctx.drawImage(this.imageCanvas, 0, 0, c.width, c.height);
        const canvas = this.aladin.webglAPI.canvas();
        ctx.drawImage(canvas, 0, 0, c.width, c.height);
        ctx.drawImage(this.catalogCanvas, 0, 0, c.width, c.height);
        ctx.drawImage(this.reticleCanvas, 0, 0, c.width, c.height);
        ctx.drawImage(this.gridCanvas, 0, 0, c.width, c.height);

        return c.toDataURL(imgType);
        //return c.toDataURL("image/jpeg", 0.01); // setting quality only works for JPEG (?)
    };


    /**
     * Compute the FoV in degrees of the view and update mouseMoveIncrement
     * 
     * @param view
     * @returns FoV (array of 2 elements : width and height) in degrees
     */
/*   function computeFov(view) {
        var fov = doComputeFov(view, view.zoomFactor);
        
        
        view.mouseMoveIncrement = fov/view.imageCanvas.width;
            
        return fov;
    }

    function doComputeFov(view, zoomFactor) {
        // if zoom factor < 1, we view 180°
        var fov;
        if (view.zoomFactor<1) {
            fov = 180.0;
            //fov = 360;
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

        fov = Math.min(180.0, fov);
        
        return fov;
    }
    */
    function updateFovDiv(view) {
        if (isNaN(view.fov)) {
            view.fovDiv.html("FoV:");
            return;
        }
        // update FoV value
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
    }
    
    
    var createListeners = function(view) {
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
            hasTouchEvents = true;
        }
        
        // various listeners
        let onDblClick = function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if(view.aladin.webglAPI.posOnUi()) {
                return;
            }
            try {
                var lonlat = view.aladin.webglAPI.screenToWorld(xymouse.x, xymouse.y);
            }
            catch(err) {
                return;
            }
            var radec;
            /*if (view.aladin.webglAPI.cooSystem() === Aladin.wasmLibs.webgl.GALCooSys()) {
                radec = view.aladin.webglAPI.Gal2J2000(lonlat[0], lonlat[1]);
            } else {*/
                radec = lonlat;
            //}
            //var radec = view.aladin.webglAPI.;
            // convert to J2000 if needed
            /*if (view.cooFrame.system==CooFrameEnum.SYSTEMS.GAL) {
                radec = CooConversion.GalacticToJ2000([lonlat.ra, lonlat.dec]);
            }
            else {
                radec = lonlat;
            }*/
            
            view.pointTo(radec[0], radec[1], {forceAnimation: true});
        };
        if (! hasTouchEvents) {
            $(view.reticleCanvas).dblclick(onDblClick);
        }
        
        $(view.reticleCanvas).bind("mousedown touchstart", function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if(view.aladin.webglAPI.posOnUi()) {
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

        //$(view.reticleCanvas).bind("mouseup mouseout touchend", function(e) {
        $(view.reticleCanvas).bind("click mouseout touchend", function(e) { // reacting on 'click' rather on 'mouseup' is more reliable when panning the view
            var xymouse = view.imageCanvas.relMouseCoords(e);
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

                view.mustRedrawReticle = true; // pour effacer selection bounding box
                view.requestRedraw();

                return;
            }



            view.mustClearCatalog = true;
            view.mustRedrawReticle = true; // pour effacer selection bounding box
            view.dragx = view.dragy = null;



            if (e.type==="mouseout" || e.type==="touchend") {
                view.requestRedraw(true);
                updateLocation(view, view.width/2, view.height/2, true);


                if (e.type==="mouseout") {
                    if (view.mode===View.TOOL_SIMBAD_POINTER) {
                        view.setMode(View.PAN);
                    }

                    return;
                }
            }


            if (view.mode==View.TOOL_SIMBAD_POINTER) {
                var radec = view.aladin.pix2world(xymouse.x, xymouse.y);

                view.setMode(View.PAN);
                view.setCursor('wait');

                SimbadPointer.query(radec[0], radec[1], Math.min(1, 15 * view.fov / view.largestDim), view.aladin);

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

            view.requestRedraw(true);
            view.aladin.webglAPI.releaseLeftButtonMouse();
        });
        var lastHoveredObject; // save last object hovered by mouse
        var lastMouseMovePos = null;
        let p = null;
        $(view.reticleCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();
            var xymouse = view.imageCanvas.relMouseCoords(e);
            p = xymouse;
            if(view.aladin.webglAPI.posOnUi()) {

                return;
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
                var dist = Math.sqrt(Math.pow(e.originalEvent.touches[0].clientX - e.originalEvent.touches[1].clientX, 2) + Math.pow(e.originalEvent.touches[0].clientY - e.originalEvent.touches[1].clientY, 2));
                view.setZoom(view.pinchZoomParameters.initialFov * view.pinchZoomParameters.initialDistance / dist);

                return;
            }



            if (!view.dragging || hasTouchEvents) {
                // update location box
                updateLocation(view, xymouse.x, xymouse.y);
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
                //console.log(view.dragx, view.dragy)
                //console.log(xymouse)

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

            //view.viewCenter.lon += xoffset*view.mouseMoveIncrement/Math.cos(view.viewCenter.lat*Math.PI/180.0);
            /*
            view.viewCenter.lon += xoffset*view.mouseMoveIncrement;
            view.viewCenter.lat += yoffset*view.mouseMoveIncrement;
            */
            
            //view.viewCenter.lon = pos2.ra -  pos1.ra;
            //view.viewCenter.lat = pos2.dec - pos1.dec;
            //view.viewCenter.lon = pos2.ra;
            //view.viewCenter.lon = pos2.ra;

            
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
            view.realDragging = true;

            //webglAPI.goFromTo(pos1[0], pos1[1], pos2[0], pos2[1]);
            view.aladin.webglAPI.goFromTo(s1.x, s1.y, s2.x, s2.y);
            //webglAPI.setCenter(pos2[0], pos2[1]);
            let viewCenter = view.aladin.webglAPI.getCenter();
            view.viewCenter.lon = viewCenter[0];
            view.viewCenter.lat = viewCenter[1];


            //console.log(view.viewCenter);

            view.requestRedraw();
        }); //// endof mousemove ////
        
        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }

        $(view.reticleCanvas).on('wheel', function(event) {            
            event.preventDefault();
            event.stopPropagation();
            //var xymouse = view.imageCanvas.relMouseCoords(event);

            if(view.aladin.webglAPI.posOnUi()) {
                return;
            }
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
            const si = 500000.0;
            const alpha = 40.0;
            let off = 0.00001 * delta;

            view.pinchZoomParameters.initialAccDelta += off;

            if (view.pinchZoomParameters.initialAccDelta <= 0.0) {
                view.pinchZoomParameters.initialAccDelta = 1e-3;
            }

            let new_fov = si / Math.pow(view.pinchZoomParameters.initialAccDelta, alpha);
            if (new_fov > 1000.0) {
                new_fov = 1000.0;
                view.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
            } 
            if (new_fov < 2e-10) {
                new_fov = 2e-10;
                view.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
            } 

            view.setZoom(new_fov);

            if (! view.debounceProgCatOnZoom) {
                var self = view;
                view.debounceProgCatOnZoom = Utils.debounce(function() {self.refreshProgressiveCats();}, 300);
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

    function updateLocation(view, x, y, isViewCenterPosition) {
        if (!view.projection) {
            return;
        }
        //var xy = AladinUtils.viewToXy(x, y, view.width, view.height, view.largestDim, view.zoomFactor);

        var lonlat;
        try {
            lonlat = view.aladin.webglAPI.screenToWorld(x, y);
        } catch(err) {
        }
        
        if (lonlat) {
            // Convert it to galactic
            if (view.aladin.webglAPI.cooSystem() === Aladin.wasmLibs.webgl.GALCooSys()) {
                lonlat = view.aladin.webglAPI.J20002Gal(lonlat[0], lonlat[1]);
            }

            //console.log(view.aladin.webglAPI.readPixel(x, y, 'base'));
            view.location.update(lonlat[0], lonlat[1], view.cooFrame, isViewCenterPosition);
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

    View.prototype.setGridColor = function(r, g, b, a) {
        this.aladin.webglAPI.setGridColor(r, g, b, a);
    }

    /**
     * redraw the whole view
     */
    View.prototype.redraw = function() {
        // request another frame
    
        requestAnimationFrame(this.redraw.bind(this));
    
        // calc elapsed time since last loop
    
        this.now = Date.now();
        let elapsed = this.now - this.then;
        // if enough time has elapsed, draw the next frame
        const fpsInterval = 1000/60;
        //if (elapsed > fpsInterval) {
    
            // Get ready for next frame by setting then=now, but also adjust for your
            // specified fpsInterval not being a multiple of RAF's interval (16.7ms)
            this.then = this.now - (elapsed % fpsInterval);
    
            // Put your drawing code here
            var saveNeedRedraw = this.needRedraw;

            this.ready = this.aladin.webglAPI.isReady();
            if (this.imageSurveysToSet !== null && (this.firstHiPS || this.ready)) {
                try {
                    this.aladin.webglAPI.setImageSurveys(this.imageSurveysToSet);
                } catch(e) {
                    console.warn(e)
                }
    
                this.imageSurveysToSet = null;
                this.firstHiPS = false;
            }
            //var now_update = Date.now();
            try {
                //var dt = now_update - this.prev;
                this.aladin.webglAPI.update(elapsed, this.needRedraw);
            } catch(e) {
                console.error(e)
            }
            // This is called at each frame
            // Better way is to give this function
            // to Rust so that the backend executes it
            // only when necessary, i.e. during the zoom
            // animation
            updateFovDiv(this);
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
    
            var imageCtx = this.imageCtx;
            //////// 1. Draw images ////////
            /*if (imageCtx.start2D) {
                imageCtx.start2D();
            }*/
            //// clear canvas ////
            // TODO : do not need to clear if fov small enough ?
            /*imageCtx.clearRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
            ////////////////////////
        
            var bkgdColor = this.getBackgroundColor();    
            // fill with background of the same color than the first color map value (lowest intensity)
            if (this.projectionMethod==ProjectionEnum.SIN) {
                if (this.fov>=60) {
                    imageCtx.fillStyle = bkgdColor;
                    imageCtx.beginPath();
                    var maxCxCy = this.cx>this.cy ? this.cx : this.cy;
                    imageCtx.arc(this.cx, this.cy, maxCxCy * this.zoomFactor, 0, 2*Math.PI, true);
                    imageCtx.fill();
                }
                // pour eviter les losanges blancs qui apparaissent quand les tuiles sont en attente de chargement
                else {
                    imageCtx.fillStyle = bkgdColor;
                    imageCtx.fillRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
                }
            }
            else if (this.projectionMethod==ProjectionEnum.AITOFF) {
                if (imageCtx.ellipse) {
                    imageCtx.fillStyle = bkgdColor;
                    imageCtx.beginPath();
                    imageCtx.ellipse(this.cx, this.cy, 2.828*this.cx*this.zoomFactor, this.cx*this.zoomFactor*1.414, 0, 0, 2*Math.PI);
                    imageCtx.fill();
                }
            }*/
            /*if (imageCtx.finish2D) {
                imageCtx.finish2D();
            }*/
    
            
            this.projection.setCenter(this.viewCenter.lon, this.viewCenter.lat);
            // do we have to redo that every time? Probably not
            //this.projection.setProjection(this.projectionMethod);
    
    
            // ************* Draw allsky tiles (low resolution) *****************
    
            var cornersXYViewMapHighres = null;
            // Pour traitement des DEFORMATIONS --> TEMPORAIRE, draw deviendra la methode utilisee systematiquement
    
            /*if (this.imageSurvey && this.imageSurvey.isReady && this.displaySurvey) {
                    if (this.aladin.reduceDeformations==null) {
                        this.imageSurvey.draw(imageCtx, this, !this.dragging, this.curNorder);
                    }
    
                    else {
                        this.imageSurvey.draw(imageCtx, this, this.aladin.reduceDeformations, this.curNorder);
                    }
            }*/
            /*
            else {
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
                    // TODO refactoring : should be a method of HpxImageSurvey
                    this.imageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curNorder);
                    if (this.curNorder>=3) {
                        this.imageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, this.curNorder);
                    }
                }
            }
            */
            
    
            // redraw overlay image survey
            // TODO : does not work if different frames 
            // TODO: use HpxImageSurvey.draw method !!
            if (this.overlayImageSurvey && this.overlayImageSurvey.isReady) {
                /*imageCtx.globalAlpha = this.overlayImageSurvey.getAlpha();
    
                if (this.aladin.reduceDeformations==null) {
                    this.overlayImageSurvey.draw(imageCtx, this, !this.dragging, this.curOverlayNorder);
                }
    
                else {
                    this.overlayImageSurvey.draw(imageCtx, this, this.aladin.reduceDeformations, this.curOverlayNorder);
                }*/
                /*
                if (this.fov>50) {
                    this.overlayImageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curOverlayNorder);
                }
                if (this.curOverlayNorder>=3) {
                    var norderOverlay = Math.min(this.curOverlayNorder, this.overlayImageSurvey.maxOrder);
                    if ( cornersXYViewMapHighres==null || norderOverlay != this.curNorder ) {
                        cornersXYViewMapHighres = this.getVisibleCells(norderOverlay);
                    }
                    this.overlayImageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, norderOverlay);
                }
                */
    
               //imageCtx.globalAlpha = 1.0;
    
            }
            
            // redraw HEALPix grid
            if( this.displayHpxGrid) {
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
                this.gridCtx.clearRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
                if (cornersXYViewMapHighres && this.curNorder>3) {
                    this.healpixGrid.redraw(this.gridCtx, cornersXYViewMapHighres, this.fov, this.curNorder);
                }
                else {
                    this.healpixGrid.redraw(this.gridCtx, cornersXYViewMapAllsky, this.fov, 3);
                }
            }
            
            // redraw coordinates grid
            /*if (this.showGrid) {
                if (this.cooGrid==null) {
                    this.cooGrid = new CooGrid();
                }
                
                this.cooGrid.redraw(this.gridCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.fov);
            }*/
             
    
    
            ///*
            ////// 2. Draw catalogues////////
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
                if (! catalogCanvasCleared) {
                    catalogCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }
                for (var i=0; i<this.catalogs.length; i++) {
                    var cat = this.catalogs[i];
                    //console.log( this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
                    cat.draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
                }
            }
            // draw popup catalog
            if (this.catalogForPopup.isShowing && this.catalogForPopup.sources.length>0) {
                if (! catalogCanvasCleared) {
                    catalogCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }
                this.catalogForPopup.draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
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
    
            //*/
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
            /*
            ////// 5. Draw all-sky ring /////
            if (this.projectionMethod==ProjectionEnum.SIN && this.fov>=60 && this.aladin.options['showAllskyRing'] === true) {
                        imageCtx.strokeStyle = this.aladin.options['allskyRingColor'];
                        var ringWidth = this.aladin.options['allskyRingWidth'];
                        imageCtx.lineWidth = ringWidth;
                        imageCtx.beginPath();
                        var maxCxCy = this.cx>this.cy ? this.cx : this.cy;
                        imageCtx.arc(this.cx, this.cy, (maxCxCy-(ringWidth/2.0)+1) * this.zoomFactor, 0, 2*Math.PI, true);
                        imageCtx.stroke();
            }
    
            
            // draw selection box
            if (this.mode==View.SELECT && this.dragging) {
                reticleCtx.fillStyle = "rgba(100, 240, 110, 0.25)";
                var w = this.dragx - this.selectStartCoo.x;
                var h =  this.dragy - this.selectStartCoo.y;
                
                reticleCtx.fillRect(this.selectStartCoo.x, this.selectStartCoo.y, w, h);
            }
            */
            
             // TODO : is this the right way?
             if (saveNeedRedraw==this.needRedraw) {
                 this.needRedraw = false;
             }
    
    
            // objects lookup
            if (!this.dragging) {
                this.updateObjectsLookup();
            }
        //}

        //this.prev = now_update;
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
            // if frame != frame image survey, we need to convert to survey frame system
            //var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
            //var radec = this.projection.unproject(xy.x, xy.y);
            let pos_world = this.aladin.webglAPI.screenToWorld(this.cx, this.cy);
            let radec = {
                ra: pos_world[0],
                dec: pos_world[1]
            };
            var lonlat = [];
            /*if (frameSurvey && frameSurvey.system != this.cooFrame.system) {
                if (frameSurvey.system==CooFrameEnum.SYSTEMS.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]);
                }
                else if (frameSurvey.system==CooFrameEnum.SYSTEMS.GAL) {
                    lonlat = CooConversion.J2000ToGalactic([radec.ra, radec.dec]);
                }
            }
            else {
                lonlat = [radec.ra, radec.dec];
            }*/
            lonlat = [radec.ra, radec.dec];
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
            var polar = HealpixIndex.utils.radecToPolar(lonlat[0], lonlat[1]);
            var ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
            pixList.unshift(ipixCenter);

        }

        return pixList;
    };
    
    View.prototype.setAngleRotation = function(theta) {

    }

    // TODO: optimize this method !!
    View.prototype.getVisibleCells = function(norder, frameSurvey) {
        if (! frameSurvey && this.imageSurvey) {
            frameSurvey = this.imageSurvey.cooFrame;
        }
        var cells = []; // array to be returned
        var cornersXY = [];
        var spVec = new SpatialVector();
        var nside = Math.pow(2, norder); // TODO : to be modified
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
            // if frame != frame image survey, we need to convert to survey frame system
            var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
            //var radec = this.projection.unproject(xy.x, xy.y);
            var radec = this.aladin.webglAPI.screenToWorld(this.cx, this.cy);
            var radec = {
                ra: radec[0],
                dec: radec[1],
            };
            var lonlat = [];
            if (frameSurvey && frameSurvey.system != this.cooFrame.system) {
                if (frameSurvey.system==CooFrameEnum.SYSTEMS.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]); 
                }
                else if (frameSurvey.system==CooFrameEnum.SYSTEMS.GAL) {
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
            var polar = HealpixIndex.utils.radecToPolar(lonlat[0], lonlat[1]);
            ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
            pixList.unshift(ipixCenter);
        }
        
        
        var ipix;
        var lon, lat;
        var corners;
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
                if (frameSurvey && frameSurvey.system != this.cooFrame.system) {
                    if (frameSurvey.system == CooFrameEnum.SYSTEMS.J2000) {
                        var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
                        lon = radec[0];
                        lat = radec[1];
                    }
                    else if (frameSurvey.system == CooFrameEnum.SYSTEMS.GAL) {
                        var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
                        lon = radec[0];
                        lat = radec[1];
                    }
                }
                else {
                    lon = spVec.ra();
                    lat = spVec.dec();
                }
                
                //cornersXY[k] = this.projection.project(lon, lat);
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
//            if (this.fov<160) { // don't bother checking if fov is large enough
//                if ( ! AladinUtils.isHpxPixVisible(cornersXYView, this.width, this.height) ) {
//                    continue;
//                }
//            }
            // check if we have a pixel at the edge of the view in allsky projections
            if (this.projection.PROJECTION!=ProjectionEnum.SIN && this.projection.PROJECTION!=ProjectionEnum.TAN) {
               /* console.log(this.largestDim);
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
                }*/

                // New faster approach: when a vertex from a cell gets to the other side of the projection
                // its vertices order change from counter-clockwise to clockwise!
                // So if the vertices describing a cell are given in clockwise order
                // we know it crosses the projection, so we do not plot them!
                if (!AladinUtils.counterClockwiseTriangle(cornersXYView[0].vx, cornersXYView[0].vy, cornersXYView[1].vx, cornersXYView[1].vy, cornersXYView[2].vx, cornersXYView[2].vy) ||
                    !AladinUtils.counterClockwiseTriangle(cornersXYView[0].vx, cornersXYView[0].vy, cornersXYView[2].vx, cornersXYView[2].vy, cornersXYView[3].vx, cornersXYView[3].vy)) {
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
        var nside = Math.pow(2, norder); // TODO : to be modified
        
        
        var cornersXYView = [];  // will be returned
        var corners = HealpixCache.corners_nest(ipix, nside);

        for (var k=0; k<4; k++) {
            spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
                
            // need for frame transformation ?
            if (this.imageSurvey && this.imageSurvey.cooFrame.system != this.cooFrame.system) {
                if (this.imageSurvey.cooFrame.system == CooFrameEnum.SYSTEMS.J2000) {
                    var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
                    lon = radec[0];
                    lat = radec[1];
                }
                else if (this.imageSurvey.cooFrame.system == CooFrameEnum.SYSTEMS.GAL) {
                    var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
                    lon = radec[0];
                    lat = radec[1];
                }
            }
            else {
                lon = spVec.ra();
                lat = spVec.dec();
            }
            //cornersXY[k] = this.projection.project(lon, lat);
            let xy = this.aladin.webglAPI.worldToScreen(lon, lat);
            cornersXYView[k] = {
                vx: xy.x,
                vy: xy.y
            };
        }
        
        if (cornersXYView[0] == null ||  cornersXYView[1] == null  ||  cornersXYView[2] == null ||  cornersXYView[3] == null ) {
            return null;
        }
        /*if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
            return null;
        }*/
        /*for (var k=0; k<4; k++) {
            cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
        }*/

        return cornersXYView;
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
    View.prototype.setZoom = function(fovDegrees) {
        if (fovDegrees<0) {
            return;
        }
        // Erase the field of view state of the backend by
        this.aladin.webglAPI.setFieldOfView(fovDegrees);
        //var zoomLevel = Math.log(180/fovDegrees)/Math.log(1.15);
        //this.setZoomLevel(zoomLevel);
        this.updateZoomState();
    };

    View.prototype.increaseZoom = function() {
        const si = 500000.0;
        const alpha = 40.0;
        const amount = 0.015;

        this.pinchZoomParameters.initialAccDelta += amount;

        if (this.pinchZoomParameters.initialAccDelta <= 0.0) {
            this.pinchZoomParameters.initialAccDelta = 1e-3;
        }

        let new_fov = si / Math.pow(this.pinchZoomParameters.initialAccDelta, alpha);
        if (new_fov > 1000.0) {
            new_fov = 1000.0;
            this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 
        if (new_fov < 2e-10) {
            new_fov = 2e-10;
            this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 

        this.setZoom(new_fov);
    }
    View.prototype.decreaseZoom = function() {
        const si = 500000.0;
        const alpha = 40.0;
        const amount = 0.015;

        this.pinchZoomParameters.initialAccDelta -= amount;

        if (this.pinchZoomParameters.initialAccDelta <= 0.0) {
            this.pinchZoomParameters.initialAccDelta = 1e-3;
        }

        let new_fov = si / Math.pow(this.pinchZoomParameters.initialAccDelta, alpha);
        if (new_fov > 1000.0) {
            new_fov = 1000.0;
            this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 
        if (new_fov < 2e-10) {
            new_fov = 2e-10;
            this.pinchZoomParameters.initialAccDelta = Math.pow(si / new_fov, 1.0/alpha);
        } 

        this.setZoom(new_fov);
    }
    View.prototype.setShowGrid = function(showGrid) {
        this.showGrid = showGrid;
        if (showGrid) {
            this.aladin.webglAPI.enableGrid();
        } else {
            this.aladin.webglAPI.disableGrid();
        }
        this.requestRedraw();
    };

    //View.prototype.setZoom = function(level) {
    View.prototype.updateZoomState = function() {
        /*let zoom = {"action": undefined};

        if (this.zoomLevel > level) {
            console.log("unzoom")
            zoom["action"] = "unzoom";
        } else if (this.zoomLevel < level) {
            zoom["action"] = "zoom";
        }*/

        /*if (this.minFOV || this.maxFOV) {
            var newFov = doComputeFov(this, this.computeZoomFactor(Math.max(-2, level)));
            if (this.maxFOV && newFov>this.maxFOV  ||  this.minFOV && newFov<this.minFOV)  {
                return;
            }
        }*/

        /*if (this.projectionMethod==ProjectionEnum.SIN) {
            //this.zoomLevel = Math.max(-2, level); // TODO : canvas freezes in firefox when max level is small
            this.zoomLevel = Math.max(-7, level); // TODO : canvas freezes in firefox when max level is small
        } else {
            this.zoomLevel = Math.max(-7, level); // TODO : canvas freezes in firefox when max level is small
        }*/
        //this.zoomLevel = Math.max(-7, level);
        
        /// Old
        /*this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
        this.fov = computeFov(this);

        if (this.zoomFactor >= 1.0) {
            this.aladin.webglAPI.setFieldOfView(this.fov);
        } else {
            //console.log("FOV, ", this.fov / this.zoomFactor);

            // zoom factor
            this.aladin.webglAPI.setFieldOfView(this.fov / this.zoomFactor);
        }*/
        this.zoomFactor = this.aladin.webglAPI.getClipZoomFactor();
        this.fov = this.aladin.webglAPI.getFieldOfView();

        // TODO: event/listener should be better
        //updateFovDiv(this);
        
        this.computeNorder();
        
        this.forceRedraw();
        //this.requestRedraw();
        // on avertit les catalogues progressifs

    };
    
    /**
     * compute and set the norder corresponding to the current view resolution
     */
    View.prototype.computeNorder = function() {
        var resolution = this.fov / this.largestDim; // in degree/pixel
        var tileSize = 512; // TODO : read info from HpxImageSurvey.tileSize
        var nside = HealpixIndex.calculateNSide(3600*tileSize*resolution); // 512 = size of a "tile" image
        var norder = Math.log(nside)/Math.log(2);
        norder = Math.max(norder, 1);
        this.realNorder = norder;

            
        // here, we force norder to 3 (otherwise, the display is "blurry" for too long when zooming in)
        if (this.fov<=50 && norder<=2) {
            norder = 3;
        }
           

        // that happens if we do not wish to display tiles coming from Allsky.[jpg|png]
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
    
    View.prototype.setOverlayImageSurvey = async function(idOrUrl) {
        /*if (! overlayImageSurvey) {
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
        });*/
        if (!idOrUrl) {
            return;
        }

        let overlaySurvey = await new HpxImageSurvey(idOrUrl);
        this.aladin.webglAPI.setOverlayHiPS(overlaySurvey);
    };

    View.prototype.setUnknownSurveyIfNeeded = function() {
        if (unknownSurveyId) {
            this.setImageSurvey(unknownSurveyId);
            unknownSurveyId = undefined;
        }
    }

    var unknownSurveyId = undefined;
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
    View.prototype.addImageSurvey = function(survey, layer) {
        // We wait for the HpxImageSurvey to complete
        // Register to the view
        const url = survey.properties.url;
        survey.layer = layer;

        this.imageSurveys.get(layer).set(url, survey);
        // Then we send the current surveys to the backend
        this.setHiPS();
    };

    View.prototype.setImageSurvey = function(survey, layer) {
        const url = survey.properties.url;
        survey.layer = layer;
        
        this.imageSurveys.set(layer, new Map());
        this.imageSurveys.get(layer).set(url, survey);
        // Then we send the current surveys to the backend
        this.setHiPS();
    };

    View.prototype.setImageSurveysLayer = function(surveys, layer) {
        this.imageSurveys.set(layer, new Map());

        surveys.forEach(survey => {
            const url = survey.properties.url;
            survey.layer = layer;
            
            this.imageSurveys.get(layer).set(url, survey);
        });

        // Then we send the current surveys to the backend
        this.setHiPS();
    };

    View.prototype.removeImageSurveysLayer = function (layer) {
        this.imageSurveys.delete(layer);

        this.setHiPS();
    };

    View.prototype.moveImageSurveysLayerForward = function(layer) {
        this.aladin.webglAPI.moveImageSurveysLayerForward(layer);
    }

    View.prototype.setHiPS = function() {
        let surveys = [];
        for (let layer of this.imageSurveys.values()) {
            for (let survey of layer.values()) {
                surveys.push(survey);
            }
        }

        this.imageSurveysToSet = surveys;
    };

    View.prototype.requestRedraw = function() {
        this.needRedraw = true;
    };
    
    View.prototype.changeProjection = function(projectionName) {
        switch (projectionName) {
            case "aitoff":
                this.projectionMethod = ProjectionEnum.AITOFF;
                break;
            case "tan":
                this.projectionMethod = ProjectionEnum.TAN;
                break;
            case "arc":
                this.projectionMethod = ProjectionEnum.ARC;
                break;
            case "mercator":
                this.projectionMethod = ProjectionEnum.MERCATOR;
                break;
            case "mollweide":
                this.projectionMethod = ProjectionEnum.MOL;
                break;
            case "sinus":
            default:
                this.projectionMethod = ProjectionEnum.SIN;
        }
        // Change the projection here
        this.projection.setProjection(this.projectionMethod);
        this.aladin.webglAPI = this.aladin.webglAPI.setProjection(projectionName);

        this.requestRedraw();
    };

    View.prototype.changeFrame = function(cooFrame) {
        var oldCooFrame = this.cooFrame;
        this.cooFrame = cooFrame;
        // recompute viewCenter
        console.log("change frame")
        if (this.cooFrame.system == CooFrameEnum.SYSTEMS.GAL && this.cooFrame.system != oldCooFrame.system) {
            var lb = CooConversion.J2000ToGalactic([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = lb[0];
            this.viewCenter.lat = lb[1]; 

            const GAL = Aladin.wasmLibs.webgl.GALCooSys();
            this.aladin.webglAPI.setCooSystem(GAL);
        }
        else if (this.cooFrame.system == CooFrameEnum.SYSTEMS.J2000 && this.cooFrame.system != oldCooFrame.system) {
            var radec = CooConversion.GalacticToJ2000([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = radec[0];
            this.viewCenter.lat = radec[1];

            const ICRSJ2000 = Aladin.wasmLibs.webgl.ICRSJ2000CooSys();
            this.aladin.webglAPI.setCooSystem(ICRSJ2000);
        }

        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        this.requestRedraw();
    };

    View.prototype.showHealpixGrid = function(show) {
        // Clear the grid ctx when not showing it
        if (!show) {
            this.gridCtx.clearRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
        }
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

    View.prototype.pointTo = function(ra, dec, options) {
        options = options || {};
        ra = parseFloat(ra);
        dec = parseFloat(dec);

        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        //if (this.cooFrame.system==CooFrameEnum.SYSTEMS.J2000) {
            this.viewCenter.lon = ra;
            this.viewCenter.lat = dec;
        //}
        /*else if (this.cooFrame.system==CooFrameEnum.SYSTEMS.GAL) {
            var lb = CooConversion.J2000ToGalactic([ra, dec]);
            this.viewCenter.lon = lb[0];
            this.viewCenter.lat = lb[1];
        }*/
        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        if (options && options.forceAnimation === false) {
            this.aladin.webglAPI.setCenter(this.viewCenter.lon, this.viewCenter.lat);
        } else if (options && options.forceAnimation === true) {
            this.aladin.webglAPI.moveToLocation(this.viewCenter.lon, this.viewCenter.lat)
        } else {
            if (this.fov > 30.0) {
                this.aladin.webglAPI.moveToLocation(this.viewCenter.lon, this.viewCenter.lat);
            } else {
                this.aladin.webglAPI.setCenter(this.viewCenter.lon, this.viewCenter.lat);
            }
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
