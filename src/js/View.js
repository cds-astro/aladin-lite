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

import { Aladin } from "./Aladin.js";
import A from "./A.js";
import { Popup } from "./Popup.js";
import { HealpixGrid } from "./HealpixGrid.js";
import { ProjectionEnum } from "./ProjectionEnum.js";
import { Utils } from "./Utils";
import { GenericPointer } from "./GenericPointer.js";
import { Stats } from "./libs/Stats.js";
import { Circle } from "./Circle.js";
import { Ellipse } from "./Ellipse.js";
import { Polyline } from "./Polyline.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";
import { WebGLCtx } from "./WebGL.js";
import { Logger } from "./Logger.js";
import { ALEvent } from "./events/ALEvent.js";
import { ColorCfg } from "./ColorCfg.js";
import { Footprint } from "./Footprint.js";
import { Selector } from "./Selector.js";
import $ from 'jquery';

export let View = (function () {

    /** Constructor */
    function View(aladin, location, fovDiv, cooFrame, zoom) {
        this.aladin = aladin;
        // Add a reference to the WebGL API
        this.options = aladin.options;
        this.aladinDiv = this.aladin.aladinDiv;
        this.popup = new Popup(this.aladinDiv, this);
        this.createCanvases();
        this.loadingState = false;

        let self = this;
        // Init the WebGL context
        // At this point, the view has been created so the image canvas too
        try {
            // Start our Rust application. You can find `WebClient` in `src/lib.rs`
            // The Rust part should also create a new WebGL2 or WebGL1 context depending on the WebGL2 brower support.
            const webglCtx = new WebGLCtx(Aladin.wasmLibs.core, this.aladinDiv.id);
            this.aladin.wasm = webglCtx.webclient;
            this.wasm = this.aladin.wasm;

            ALEvent.AL_USE_WASM.listenedBy(document.body, function (e) {
                let callback = e.detail.callback;

                callback(self.wasm);
            });

            // Retrieve all the possible colormaps
            ColorCfg.COLORMAPS = this.wasm.getAvailableColormapList();
        } catch (e) {
            // For browsers not supporting WebGL2:
            // 1. Print the original exception message in the console
            console.error(e)
            // 2. Add a more explicite message to the end user
            alert("Problem initializing Aladin Lite. Please contact the support by contacting Matthieu Baumann (baumannmatthieu0@gmail.com) or Thomas Boch (thomas.boch@astro.unistra.fr). You can also open an issue on the Aladin Lite github repository here: https://github.com/cds-astro/aladin-lite. Message error:" + e)
        }

        // Attach the drag and drop events to the view
        this.aladinDiv.ondrop = (event) => {
            const files = Utils.getDroppedFilesHandler(event);

            files.forEach((file) => {
                const url = URL.createObjectURL(file);

                // Consider other cases
                try {
                    const image = self.aladin.createImageFITS(
                        url,
                        file.name,
                        undefined,
                        (ra, dec, fov, _) => {
                            // Center the view around the new fits object
                            aladin.gotoRaDec(ra, dec);
                            aladin.setFoV(fov * 1.1);
                        },
                        undefined
                    );
                    self.setOverlayImageLayer(image, file.name)
                } catch(e) {
                    let moc = A.MOCFromURL(url);
                    self.aladin.addMOC(moc);

                    console.error("Only valid fits files supported (i.e. containig a WCS)", e)
                    throw e;
                }
            })
        };

        this.aladinDiv.ondragover = Utils.dragOverHandler;

        this.location = location;
        this.fovDiv = fovDiv;
        this.mustClearCatalog = true;
        this.mode = View.PAN;

        this.minFOV = this.maxFOV = null; // by default, no restriction

        this.healpixGrid = new HealpixGrid();
        this.then = Date.now();

        var lon, lat;
        lon = lat = 0;
        this.projection = ProjectionEnum.SIN;

        // Prev time of the last frame
        this.zoomFactor = this.wasm.getClipZoomFactor();

        this.viewCenter = { lon: lon, lat: lat }; // position of center of view

        if (cooFrame) {
            this.cooFrame = cooFrame;
        } else {
            this.cooFrame = CooFrameEnum.GAL;
        }

        // Frame setting
        this.changeFrame(this.cooFrame);

        this.selector = new Selector();
        this.selectMode = 'rect';
        this.selectCallbackFn = undefined;

        // Zoom starting setting
        const si = 500000.0;
        const alpha = 40.0;
        this.fovLimit = undefined;
        let initialFov = zoom || 180.0;
        this.pinchZoomParameters = {
            isPinching: false, // true if a pinch zoom is ongoing
            initialFov: undefined,
            initialDistance: undefined,
            initialAccDelta: Math.pow(si / initialFov, 1.0 / alpha)
        };
        this.setZoom(initialFov);
        // current reference image survey displayed
        this.imageLayers = new Map();

        this.overlayLayers = [];
        // current catalogs displayed
        this.catalogs = [];
        // a dedicated catalog for the popup
        var c = document.createElement('canvas');
        c.width = c.height = 24;
        var ctx = c.getContext('2d');
        ctx.lineWidth = 6.0;
        ctx.beginPath();
        ctx.strokeStyle = '#eee';
        ctx.arc(12, 12, 8, 0, 2 * Math.PI, true);
        ctx.stroke();
        ctx.lineWidth = 3.0;
        ctx.beginPath();
        ctx.strokeStyle = '#c38';
        ctx.arc(12, 12, 8, 0, 2 * Math.PI, true);
        ctx.stroke();
        this.catalogForPopup = A.catalog({ shape: c, sourceSize: 24 });
        this.catalogForPopup.hide();
        this.catalogForPopup.setView(this);
        this.overlayForPopup = A.graphicOverlay({color: '#ee2345', lineWidth: 3});
        this.overlayForPopup.hide();
        this.overlayForPopup.setView(this);

        // overlays (footprints for instance)
        this.overlays = [];
        // MOCs
        this.mocs = [];
        // reference to all overlay layers (= catalogs + overlays + mocs)
        this.allOverlayLayers = []

        this.empty = true;

        //this.fixLayoutDimensions();
        this.promises = [];
        this.firstHiPS = true;
        this.curNorder = 1;
        this.realNorder = 1;
        this.imageLayersBeingQueried = new Map();

        // some variables for mouse handling
        this.dragging = false;
        this.dragCoo = null;
        this.rightclickx = null;
        this.rightclicky = null;
        this.selectedLayer = 'base';

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
        let resizeObserver = new ResizeObserver(() => {
            self.fixLayoutDimensions();
            self.requestRedraw();
        });

        this.throttledPositionChanged = Utils.throttle(
            () => {
                var posChangedFn = this.aladin.callbacksByEventName && this.aladin.callbacksByEventName['positionChanged'];
                if (typeof posChangedFn === 'function') {
                    var pos = this.aladin.pix2world(this.width / 2, this.height / 2);
                    if (pos !== undefined) {
                        posChangedFn({
                            ra: pos[0],
                            dec: pos[1],
                            dragging: true
                        });
                    }
                }
            },
            View.CALLBACKS_THROTTLE_TIME_MS,
        );

        this.throttledZoomChanged = Utils.throttle(
            () => {
                const fov = this.fov;
                // trigger callback only if FoV (zoom) has changed !
                if (fov !== this.oldFov) {
                    const fovChangedFn = this.aladin.callbacksByEventName['zoomChanged'];
                    (typeof fovChangedFn === 'function') && fovChangedFn(fov);

                    // finally, save fov value
                    this.oldFov = fov;
                }
            },
            View.CALLBACKS_THROTTLE_TIME_MS,
        );

        resizeObserver.observe(this.aladinDiv);
        //$(window).resize(() => {
        self.fixLayoutDimensions();
            //self.requestRedraw();
        //});
        // in some contexts (Jupyter notebook for instance), the parent div changes little time after Aladin Lite creation
        // this results in canvas dimension to be incorrect.
        // The following line tries to fix this issue
        //setTimeout(function () {
        //var computedWidth = $(self.aladinDiv).width();
        //var computedHeight = $(self.aladinDiv).height();

        //if (self.width !== computedWidth || self.height === computedHeight) {
        //self.fixLayoutDimensions();
        // As the WebGL backend has been resized correctly by
        // the previous call, we can get the zoom factor from it

        //self.setZoom(self.fov); // needed to force recomputation of displayed FoV
        //}

        self.requestRedraw();
        //}, 1000);

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
    View.prototype.createCanvases = function () {
        var a = $(this.aladinDiv);
        a.find('.aladin-imageCanvas').remove();
        a.find('.aladin-gridCanvas').remove();
        a.find('.aladin-catalogCanvas').remove();

        // canvas to draw the images
        this.imageCanvas = $("<canvas class='aladin-imageCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        this.gridCanvas = $("<canvas class='aladin-gridCanvas'></canvas>").appendTo(this.aladinDiv)[0];

        // canvas to draw the catalogs
        this.catalogCanvas = $("<canvas class='aladin-catalogCanvas'></canvas>").appendTo(this.aladinDiv)[0];
    };

    // called at startup and when window is resized
    // The WebGL backend is resized
    View.prototype.fixLayoutDimensions = function () {
        Utils.cssScale = undefined;

        var computedWidth = $(this.aladinDiv).width();
        var computedHeight = $(this.aladinDiv).height();

        this.width = Math.max(computedWidth, 1);
        this.height = Math.max(computedHeight, 1); // this prevents many problems when div size is equal to 0

        this.cx = this.width / 2;
        this.cy = this.height / 2;

        this.largestDim = Math.max(this.width, this.height);
        this.smallestDim = Math.min(this.width, this.height);
        this.ratio = this.largestDim / this.smallestDim;

        this.mouseMoveIncrement = 160 / this.largestDim;

        // reinitialize 2D context
        this.imageCtx = this.imageCanvas.getContext("webgl2");
        //this.aladinDiv.style.width = this.width + "px";
        //this.aladinDiv.style.height = this.height + "px";

        this.wasm.resize(this.width, this.height);

        this.catalogCtx = this.catalogCanvas.getContext("2d");
        this.catalogCtx.canvas.width = this.width;
        this.catalogCtx.canvas.height = this.height;

        this.gridCtx = this.gridCanvas.getContext("2d");
        this.gridCtx.canvas.width = this.width;
        this.gridCtx.canvas.height = this.height;

        pixelateCanvasContext(this.imageCtx, this.aladin.options.pixelateCanvas);

        // change logo
        if (!this.logoDiv) {
            this.logoDiv = $(this.aladinDiv).find('.aladin-logo')[0];
        }
        if (this.width > 800) {
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

    var pixelateCanvasContext = function (ctx, pixelateFlag) {
        var enableSmoothing = !pixelateFlag;
        ctx.imageSmoothingEnabled = enableSmoothing;
        ctx.webkitImageSmoothingEnabled = enableSmoothing;
        ctx.mozImageSmoothingEnabled = enableSmoothing;
        ctx.msImageSmoothingEnabled = enableSmoothing;
        ctx.oImageSmoothingEnabled = enableSmoothing;
    }

    View.prototype.startSelection = function(mode, callbackFn) {
        this.setMode(View.SELECT);

        this.selectMode = mode;
        this.selectCallbackFn = callbackFn;
    }

    View.prototype.finishSelection = function() {
        this.setMode(View.PAN);
    }

    View.prototype.setMode = function (mode) {
        this.mode = mode;
        if (this.mode == View.SELECT) {
            this.setCursor('crosshair');
        }
        else if (this.mode == View.TOOL_SIMBAD_POINTER) {
            this.popup.hide();
            this.catalogCanvas.style.cursor = '';
            $(this.catalogCanvas).addClass('aladin-sp-cursor');
        }
        else {
            this.setCursor('default');
        }
    };

    View.prototype.setCursor = function (cursor) {
        if (this.catalogCanvas.style.cursor == cursor) {
            return;
        }
        if (this.mode == View.TOOL_SIMBAD_POINTER) {
            return;
        }
        this.catalogCanvas.style.cursor = cursor;
    };

    /**
     * return dataURL string corresponding to the current view
     */
    View.prototype.getCanvasDataURL = async function (imgType, width, height) {
        const loadImage = function (url) {
            return new Promise((resolve, reject) => {
                var image = new Image()
                image.src = url
                image.onload = () => resolve(image)
                image.onerror = () => reject(new Error('could not load image'))
            })
        }

        return loadImage("data:image/png;base64,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")
            .then((img) => {
                imgType = imgType || "image/png";

                const canvas = this.wasm.canvas();

                var c = document.createElement('canvas');
                let dpi = window.devicePixelRatio;
                c.width = width || (this.width * dpi);
                c.height = height || (this.height * dpi);

                var ctx = c.getContext('2d');

                ctx.drawImage(canvas, 0, 0, c.width, c.height);
                ctx.drawImage(this.catalogCanvas, 0, 0, c.width, c.height);

                const offX = c.width - img.width;
                const offY = c.height - img.height;
                ctx.drawImage(img, offX, offY);

                return c.toDataURL(imgType);
            });
    };


    View.prototype.setActiveHiPSLayer = function (layer) {
        if (!this.imageLayers.has(layer)) {
            throw layer + ' does not exists. So cannot be selected';
        }
        this.selectedLayer = layer;
    };

    var createListeners = function (view) {
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
            hasTouchEvents = true;
        }

        // various listeners
        let onDblClick = function (e) {
            const xymouse = Utils.relMouseCoords(view.imageCanvas, e);

            // deselect all the selected sources with Select panel
            view.hideSelectedObjects()

            try {
                const lonlat = view.wasm.screenToWorld(xymouse.x, xymouse.y);
                var radec = view.wasm.viewToICRSCooSys(lonlat[0], lonlat[1]);
                view.pointTo(radec[0], radec[1], { forceAnimation: true });
            }
            catch (err) {
                return;
            }

        };
        if (!hasTouchEvents) {
            $(view.catalogCanvas).dblclick(onDblClick);
        }

        // prevent default context menu from appearing (potential clash with right-click cuts control)
        $(view.catalogCanvas).bind("contextmenu", function (e) {
            // do something here...
            e.preventDefault();
        }, false);


        let cutMinInit = null
        let cutMaxInit = null;

        $(view.catalogCanvas).bind("mousedown touchstart", function (e) {
            e.preventDefault();
            e.stopPropagation();

            const xymouse = Utils.relMouseCoords(view.imageCanvas, e);

            if (e.which === 3 || e.button === 2) {
                view.rightClick = true;
                view.rightClickTimeStart = Date.now();
                view.rightclickx = xymouse.x;
                view.rightclicky = xymouse.y;

                const imageLayer = view.imageLayers.get(view.selectedLayer);
                if (imageLayer) {
                    // Take as start cut values what is inside the properties
                    // If the cuts are not defined in the metadata of the survey
                    // then we take what has been defined by the user
                    if (imageLayer.imgFormat === "fits") {
                        cutMinInit = imageLayer.properties.minCutout || imageLayer.getColorCfg().minCut || 0.0;
                        cutMaxInit = imageLayer.properties.maxCutout || imageLayer.getColorCfg().maxCut || 1.0;
                    } else {
                        cutMinInit = imageLayer.getColorCfg().minCut || 0.0;
                        cutMaxInit = imageLayer.getColorCfg().maxCut || 1.0;
                    }
                }

                return;
            }

            // zoom pinching
            if (e.type === 'touchstart' && e.originalEvent && e.originalEvent.targetTouches && e.originalEvent.targetTouches.length == 2) {
                view.dragging = false;

                view.pinchZoomParameters.isPinching = true;
                //var fov = view.aladin.getFov();
                //view.pinchZoomParameters.initialFov = Math.max(fov[0], fov[1]);
                var fov = view.wasm.getFieldOfView();
                view.pinchZoomParameters.initialFov = fov;
                view.pinchZoomParameters.initialDistance = Math.sqrt(Math.pow(e.originalEvent.targetTouches[0].clientX - e.originalEvent.targetTouches[1].clientX, 2) + Math.pow(e.originalEvent.targetTouches[0].clientY - e.originalEvent.targetTouches[1].clientY, 2));

                view.fingersRotationParameters.initialViewAngleFromCenter = view.wasm.getRotationAroundCenter();
                view.fingersRotationParameters.initialFingerAngle = Math.atan2(e.originalEvent.targetTouches[1].clientY - e.originalEvent.targetTouches[0].clientY, e.originalEvent.targetTouches[1].clientX - e.originalEvent.targetTouches[0].clientX) * 180.0 / Math.PI;

                return;
            }

            view.dragCoo = xymouse;

            view.dragging = true;
            if (view.mode == View.PAN) {
                view.setCursor('move');
            }
            else if (view.mode == View.SELECT) {
                view.selector.start(view.dragCoo, view.selectMode, view.selectCallbackFn);
            }

            view.wasm.pressLeftMouseButton(view.dragCoo.x, view.dragCoo.y);

            // false disables default browser behaviour like possibility to touch hold for context menu.
            // To disable text selection use css user-select: none instead of putting this value to false
            return true;
        });

        $(view.catalogCanvas).bind("mouseup", function (e) {
            if (view.rightClick) {

                const rightClickDurationMs = Date.now() - view.rightClickTimeStart;
                if (rightClickDurationMs<300) {
                    view.aladin.contextMenu && view.aladin.contextMenu._showMenu(e);
                }

                view.rightClick = false;
                view.rightclickx = null;
                view.rightclicky = null;
                view.rightClickTimeStart = undefined;

                return;
            }
        });

        $(view.catalogCanvas).bind("click mouseout touchend touchcancel", function (e) { // reacting on 'click' rather on 'mouseup' is more reliable when panning the view
            if ((e.type === 'touchend' || e.type === 'touchcancel') && view.pinchZoomParameters.isPinching) {
                view.pinchZoomParameters.isPinching = false;
                view.pinchZoomParameters.initialFov = view.pinchZoomParameters.initialDistance = undefined;

                return;
            }
            if ((e.type === 'touchend' || e.type === 'touchcancel') && view.fingersRotationParameters.rotationInitiated) {
                view.fingersRotationParameters.initialViewAngleFromCenter = undefined;
                view.fingersRotationParameters.initialFingerAngle = undefined;
                view.fingersRotationParameters.rotationInitiated = false;

                return;
            }

            var wasDragging = view.realDragging === true;
            var selectionHasEnded = view.mode === View.SELECT && view.dragging;

            if (view.dragging) { // if we were dragging, reset to default cursor
                view.setCursor('default');
                view.dragging = false;

                if (wasDragging) {
                    view.realDragging = false;
                }
            } // end of "if (view.dragging) ... "

            if (selectionHasEnded) {
                view.selector.finish(view.dragCoo);

                view.aladin.fire(
                    'selectend'
                );

                view.requestRedraw();

                return;
            }

            view.mustClearCatalog = true;
            view.dragCoo = null;
            const xymouse = Utils.relMouseCoords(view.imageCanvas, e);

            if (e.type === "mouseout" || e.type === "touchend" || e.type === "touchcancel") {
                view.updateLocation(xymouse.x, xymouse.y, true);

                if (e.type === "mouseout") {
                    if (view.mode === View.TOOL_SIMBAD_POINTER) {
                        view.setMode(View.PAN);
                    }

                    return;
                }
            }

            if (view.mode == View.TOOL_SIMBAD_POINTER) {
                // call Simbad pointer or Planetary features
                GenericPointer(view, e);

                return; // when in TOOL_SIMBAD_POINTER mode, we do not call the listeners
            }

            // popup to show ?
            var objs = view.closestObjects(xymouse.x, xymouse.y, 5);
            if (!wasDragging && objs) {
                view.hideSelectedObjects();

                var o = objs[0];

                // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
                if (o.marker) {
                    // could be factorized in Source.actionClicked
                    view.popup.setTitle(o.popupTitle);
                    view.popup.setText(o.popupDesc);
                    view.popup.setSource(o);
                    view.popup.show();
                }
                else {
                    if (view.lastClickedObject) {
                        view.lastClickedObject.actionOtherObjectClicked && view.lastClickedObject.actionOtherObjectClicked();
                    }
                }

                // show measurements
                if (o.actionClicked) {
                    o.actionClicked();
                }

                var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                (typeof objClickedFunction === 'function') && objClickedFunction(o, xymouse);

                if (o.isFootprint()) {
                    var footprintClickedFunction = view.aladin.callbacksByEventName['footprintClicked'];
                    if (typeof footprintClickedFunction === 'function' && o != view.lastClickedObject) {
                        var ret = footprintClickedFunction(o, xymouse);
                    }
                }

                view.lastClickedObject = o;
            } else {
                if (!wasDragging) {
                    // Deselect objects if any
                    view.hideSelectedObjects();

                    // If there is a past clicked object
                    if (view.lastClickedObject) {
                        //view.aladin.measurementTable.hide();
                        //view.aladin.sodaForm.hide();
                        view.popup.hide();

                        // Deselect the last clicked object
                        if (view.lastClickedObject instanceof Ellipse || view.lastClickedObject instanceof Circle || view.lastClickedObject instanceof Polyline) {
                            view.lastClickedObject.deselect();
                        } else {
                            // Case where lastClickedObject is a Source
                            view.lastClickedObject.actionOtherObjectClicked();
                        }

                        var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                        (typeof objClickedFunction === 'function') && objClickedFunction(null, xymouse);

                        view.lastClickedObject = null;
                    }
                }
            }

            // call listener of 'click' event
            var onClickFunction = view.aladin.callbacksByEventName['click'];
            if (typeof onClickFunction === 'function') {
                var pos = view.aladin.pix2world(xymouse.x, xymouse.y);
                if (pos !== undefined) {
                    onClickFunction({ ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y, isDragging: wasDragging });
                }
            }

            // TODO : remplacer par mecanisme de listeners
            // on avertit les catalogues progressifs
            view.refreshProgressiveCats();

            //view.requestRedraw();
            view.wasm.releaseLeftButtonMouse(xymouse.x, xymouse.y);
        });

        var lastHoveredObject; // save last object hovered by mouse
        var lastMouseMovePos = null;
        $(view.catalogCanvas).bind("mousemove touchmove", function (e) {
            e.preventDefault();
            const xymouse = Utils.relMouseCoords(view.imageCanvas, e);

            if (view.rightClick) {
                var onRightClickMoveFunction = view.aladin.callbacksByEventName['rightClickMove'];
                if (typeof onRightClickMoveFunction === 'function') {
                    onRightClickMoveFunction(xymouse.x, xymouse.y);

                    // do not process further
                    return;
                }

                if(view.selectedLayer) {
                    let selectedLayer = view.imageLayers.get(view.selectedLayer);
                    // We try to match DS9 contrast adjustment behaviour with right click
                    const cs = {
                        x: view.catalogCanvas.clientWidth * 0.5,
                        y: view.catalogCanvas.clientHeight * 0.5,
                    };
                    const cx = (xymouse.x - cs.x) / view.catalogCanvas.clientWidth;
                    const cy = -(xymouse.y - cs.y) / view.catalogCanvas.clientHeight;

                    const offset = (cutMaxInit - cutMinInit) * cx;

                    const lr = offset + (1.0 - 2.0 * cy) * cutMinInit;
                    const rr = offset + (1.0 + 2.0 * cy) * cutMaxInit;

                    if (lr <= rr) {
                        selectedLayer.setCuts(lr, rr)
                    }
                }

                return;
            }

            if (e.type === 'touchmove' && view.pinchZoomParameters.isPinching && e.originalEvent && e.originalEvent.touches && e.originalEvent.touches.length == 2) {
                // rotation
                var currentFingerAngle = Math.atan2(e.originalEvent.targetTouches[1].clientY - e.originalEvent.targetTouches[0].clientY, e.originalEvent.targetTouches[1].clientX - e.originalEvent.targetTouches[0].clientX) * 180.0 / Math.PI;
                var fingerAngleDiff = view.fingersRotationParameters.initialFingerAngle - currentFingerAngle;
                // rotation is initiated when angle is equal or greater than 7 degrees
                if (!view.fingersRotationParameters.rotationInitiated && Math.abs(fingerAngleDiff) >= 7) {
                    view.fingersRotationParameters.rotationInitiated = true;
                    view.fingersRotationParameters.initialFingerAngle = currentFingerAngle;
                    fingerAngleDiff = 0;
                }

                if (view.fingersRotationParameters.rotationInitiated) {
                    let rotation = view.fingersRotationParameters.initialViewAngleFromCenter;
                    if (!view.wasm.getLongitudeReversed()) {
                        // spatial survey case
                        rotation += fingerAngleDiff;
                    } else {
                        // planetary survey case
                        rotation -= fingerAngleDiff;
                    }
                    view.wasm.setRotationAroundCenter(rotation);
                }

                // zoom
                const dist = Math.sqrt(Math.pow(e.originalEvent.touches[0].clientX - e.originalEvent.touches[1].clientX, 2) + Math.pow(e.originalEvent.touches[0].clientY - e.originalEvent.touches[1].clientY, 2));
                const fov = Math.min(Math.max(view.pinchZoomParameters.initialFov * view.pinchZoomParameters.initialDistance / dist, 0.00002777777), view.fovLimit);
                view.setZoom(fov);

                return;
            }

            if (!view.dragging && !view.moving) {
                view.updateObjectsLookup();
            }

            if (!view.dragging || hasTouchEvents) {
                // update location box
                view.updateLocation(xymouse.x, xymouse.y, false);
            }

            if (!view.dragging) {
                // call listener of 'mouseMove' event
                var onMouseMoveFunction = view.aladin.callbacksByEventName['mouseMove'];
                if (typeof onMouseMoveFunction === 'function') {
                    var pos = view.aladin.pix2world(xymouse.x, xymouse.y);
                    if (pos !== undefined) {
                        onMouseMoveFunction({ ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y });
                    }
                    // send null ra and dec when we go out of the "sky"
                    else if (lastMouseMovePos != null) {
                        onMouseMoveFunction({ ra: null, dec: null, x: xymouse.x, y: xymouse.y });
                    }
                    lastMouseMovePos = pos;
                }

                if (!view.dragging && !view.mode == View.SELECT) {
                    // closestObjects is very costly, we would like to not do it
                    // especially if the objectHovered function is not defined.
                    var closest = view.closestObjects(xymouse.x, xymouse.y, 5);

                    if (closest) {
                        let o = closest[0];
                        var objHoveredFunction = view.aladin.callbacksByEventName['objectHovered'];
                        var footprintHoveredFunction = view.aladin.callbacksByEventName['footprintHovered'];

                        view.setCursor('pointer');
                        if (typeof objHoveredFunction === 'function' && o != lastHoveredObject) {
                            var ret = objHoveredFunction(o, xymouse);
                        }

                        if (o.isFootprint()) {
                            if (typeof footprintHoveredFunction === 'function' && o != lastHoveredObject) {
                                var ret = footprintHoveredFunction(o, xymouse);
                            }
                        }

                        lastHoveredObject = o;
                    } else {
                        view.setCursor('default');
                        var objHoveredStopFunction = view.aladin.callbacksByEventName['objectHoveredStop'];
                        if (lastHoveredObject) {
                            // Redraw the scene if the lastHoveredObject is a footprint (e.g. circle or polygon)
                            if (lastHoveredObject.isFootprint()) {
                                view.requestRedraw();
                            }

                            if (typeof objHoveredStopFunction === 'function') {
                                // call callback function to notify we left the hovered object
                                var ret = objHoveredStopFunction(lastHoveredObject, xymouse);
                            }
                        }

                        lastHoveredObject = null;
                    }
                }
                if (e.type === "mousemove") {
                    return;
                }
            }

            if (!view.dragging) {
                return;
            }

            var s1 = view.dragCoo, s2 = xymouse;
            // update drag coo with the new position
            view.dragCoo = xymouse;

            if (view.mode == View.SELECT) {
                view.requestRedraw();
                return;
            }

            view.realDragging = true;

            view.wasm.moveMouse(s1.x, s1.y, s2.x, s2.y);
            view.wasm.goFromTo(s1.x, s1.y, s2.x, s2.y);

            const [ra, dec] = view.wasm.getCenter();
            view.viewCenter.lon = ra;
            view.viewCenter.lat = dec;

            // Apply position changed callback after the move
            view.throttledPositionChanged();
        }); //// endof mousemove ////

        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }
        var eventCount = 0;
        var eventCountStart;
        var isTouchPad;
        var oldTime = 0;
        var newTime = 0;

        $(view.catalogCanvas).on('wheel', function (event) {
            event.preventDefault();
            event.stopPropagation();

            if (view.rightClick) {
                return;
            }

            var delta = event.deltaY;
            // this seems to happen in context of Jupyter notebook --> we have to invert the direction of scroll
            // hope this won't trigger some side effects ...
            if (event.hasOwnProperty('originalEvent')) {
                delta = -event.originalEvent.deltaY;
            }

            // See https://stackoverflow.com/questions/10744645/detect-touchpad-vs-mouse-in-javascript
            // for detecting the use of a touchpad
            var isTouchPadDefined = isTouchPad || typeof isTouchPad !== "undefined";
            if (!isTouchPadDefined) {
                if (eventCount === 0) {
                    eventCountStart = new Date().getTime();
                }

                eventCount++;

                if (new Date().getTime() - eventCountStart > 100) {
                    if (eventCount > 10) {
                        isTouchPad = true;
                    } else {
                        isTouchPad = false;
                    }
                    isTouchPadDefined = true;
                }
            }

            // The value of the field of view is determined
            // inside the backend
            const triggerZoom = (amount) => {
                if (delta > 0.0) {
                    view.increaseZoom(amount);
                } else {
                    view.decreaseZoom(amount);
                }
            };

            if (isTouchPadDefined) {
                if (isTouchPad) {
                    // touchpad
                    newTime = new Date().getTime();

                    //if ( newTime - oldTime > 20 ) {
                        triggerZoom(0.002);
                        oldTime = new Date().getTime();
                    //}
                } else {
                    // mouse
                    triggerZoom(0.007);
                }
            }

            if (!view.debounceProgCatOnZoom) {
                var self = view;
                view.debounceProgCatOnZoom = Utils.debounce(function () {
                    self.refreshProgressiveCats();
                    self.drawAllOverlays();
                }, 300);
            }

            view.debounceProgCatOnZoom();
            view.throttledZoomChanged();

            return false;
        });
    };

    var init = function (view) {
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#aladin-statsDiv').length > 0) {
            $('#aladin-statsDiv')[0].appendChild(stats.domElement);
        }

        view.stats = stats;

        createListeners(view);

        view.displayHpxGrid = false;
        view.displayCatalog = false;
        view.displayReticle = true;
    };

    View.prototype.updateLocation = function (mouseX, mouseY, isViewCenterPosition) {
        if (isViewCenterPosition) {
            this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);
        } else {
            let radec = this.wasm.screenToWorld(mouseX, mouseY); // This is given in the frame of the view
            if (radec) {
                if (radec[0] < 0) {
                    radec = [radec[0] + 360.0, radec[1]];
                }

                this.location.update(radec[0], radec[1], this.cooFrame, false);
            }
        }
    }

    View.prototype.requestRedrawAtDate = function (date) {
        this.dateRequestDraw = date;
    };

    View.prototype.getViewParams = function () {
        var resolution = this.width > this.height ? this.fov / this.width : this.fov / this.height;
        return {
            fov: [this.width * resolution, this.height * resolution],
            width: this.width,
            height: this.height
        };
    };

    View.FPS_INTERVAL = 1000 / 140;

    /**
     * redraw the whole view
     */
    View.prototype.redraw = function () {
        // request another frame

        // Elapsed time since last loop
        const now = Date.now();
        const elapsedTime = now - this.then;

        // If enough time has elapsed, draw the next frame
        //if (elapsedTime >= View.FPS_INTERVAL) {
            // Get ready for next frame by setting then=now, but also adjust for your
            // specified fpsInterval not being a multiple of RAF's interval (16.7ms)

            // Drawing code
            try {
                this.moving = this.wasm.update(elapsedTime);
            } catch (e) {
                console.warn(e)
            }

            ////// 2. Draw catalogues////////
            const isViewRendering = this.wasm.isRendering();
            if (isViewRendering || this.needRedraw) {
                this.drawAllOverlays();
            }
            this.needRedraw = false;

            this.then = now;
            //this.then = now % View.FPS_INTERVAL;
            requestAnimFrame(this.redraw.bind(this));
        //}
    };

    View.prototype.drawAllOverlays = function () {
        var catalogCtx = this.catalogCtx;
        var catalogCanvasCleared = false;
        if (this.mustClearCatalog) {
            catalogCtx.clearRect(0, 0, this.width, this.height);
            catalogCanvasCleared = true;
            this.mustClearCatalog = false;
        }

        if (this.catalogs && this.catalogs.length > 0 && this.displayCatalog && (!this.dragging || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            // TODO : do not clear every time
            //// clear canvas ////
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            for (var i = 0; i < this.catalogs.length; i++) {
                var cat = this.catalogs[i];
                cat.draw(catalogCtx, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
            }
        }
        // draw popup catalog
        if (this.catalogForPopup.isShowing && this.catalogForPopup.sources.length > 0) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            this.catalogForPopup.draw(catalogCtx, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);

            // draw popup overlay layer
            if (this.overlayForPopup.isShowing) {
                this.overlayForPopup.draw(catalogCtx, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor);
            }
        }

        ////// 3. Draw overlays////////
        var overlayCtx = this.catalogCtx;
        if (this.overlays && this.overlays.length > 0 && (!this.dragging || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            for (var i = 0; i < this.overlays.length; i++) {
                this.overlays[i].draw(overlayCtx);
            }
        }

        // Redraw HEALPix grid
        var healpixGridCtx = catalogCtx;
        if (this.displayHpxGrid) {
            if (!catalogCanvasCleared) {
                catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            var cornersXYViewMapAllsky = this.getVisibleCells(3);
            var cornersXYViewMapHighres = null;
            if (this.curNorder >= 3) {
                if (this.curNorder == 3) {
                    cornersXYViewMapHighres = cornersXYViewMapAllsky;
                }
                else {
                    cornersXYViewMapHighres = this.getVisibleCells(this.curNorder);
                }
            }
            if (cornersXYViewMapHighres && this.curNorder > 3) {
                this.healpixGrid.redraw(healpixGridCtx, cornersXYViewMapHighres, this.fov, this.curNorder);
            }
            else {
                this.healpixGrid.redraw(healpixGridCtx, cornersXYViewMapAllsky, this.fov, 3);
            }
        }

        ////// 4. Draw reticle ///////
        // TODO: reticle should be placed in a static DIV, no need to waste a canvas
        var reticleCtx = catalogCtx;
        if (this.mode == View.SELECT) {
            // VIEW mode, we do not want to display the reticle in this
            // but draw a selection box
            if (this.dragging) {
                if (!catalogCanvasCleared) {
                    reticleCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }

                this.selector.draw(reticleCtx, this.dragCoo)
            }
        } else {
            // Normal modes
            if (this.displayReticle) {
                if (!catalogCanvasCleared) {
                    catalogCtx.clearRect(0, 0, this.width, this.height);
                    catalogCanvasCleared = true;
                }

                if (!this.reticleCache) {
                    // build reticle image
                    var c = document.createElement('canvas');
                    var s = this.options.reticleSize;
                    c.width = s;
                    c.height = s;
                    var ctx = c.getContext('2d');
                    ctx.lineWidth = 2;
                    ctx.strokeStyle = this.options.reticleColor;
                    ctx.beginPath();
                    ctx.moveTo(s / 2, s / 2 + (s / 2 - 1));
                    ctx.lineTo(s / 2, s / 2 + 2);
                    ctx.moveTo(s / 2, s / 2 - (s / 2 - 1));
                    ctx.lineTo(s / 2, s / 2 - 2);

                    ctx.moveTo(s / 2 + (s / 2 - 1), s / 2);
                    ctx.lineTo(s / 2 + 2, s / 2);
                    ctx.moveTo(s / 2 - (s / 2 - 1), s / 2);
                    ctx.lineTo(s / 2 - 2, s / 2);

                    ctx.stroke();

                    this.reticleCache = c;
                }
                reticleCtx.drawImage(this.reticleCache, this.width / 2 - this.reticleCache.width / 2, this.height / 2 - this.reticleCache.height / 2);
            }
        }

        ////// 5. Draw all-sky ring /////
        if (this.projection == ProjectionEnum.SIN && this.fov >= 60 && this.aladin.options['showAllskyRing'] === true) {
            if (!catalogCanvasCleared) {
                reticleCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
            }

            reticleCtx.strokeStyle = this.aladin.options['allskyRingColor'];
            var ringWidth = this.aladin.options['allskyRingWidth'];
            reticleCtx.lineWidth = ringWidth;
            reticleCtx.beginPath();
            const maxCxCy = this.cy;
            const radius = (maxCxCy - (ringWidth / 2.0) + 1) / this.zoomFactor;
            reticleCtx.arc(this.cx, this.cy, radius, 0, 2 * Math.PI, true);
            reticleCtx.stroke();
        }
    };

    View.prototype.refreshProgressiveCats = function () {
        if (!this.catalogs) {
            return;
        }

        for (var i = 0; i < this.catalogs.length; i++) {
            if (this.catalogs[i].type == 'progressivecat') {
                this.catalogs[i].loadNeededTiles();
            }
        }
    };

    View.prototype.getVisiblePixList = function (norder) {
        var pixList = [];
        let centerWorldPosition = this.wasm.screenToWorld(this.cx, this.cy);
        const [lon, lat] = this.wasm.viewToICRSCooSys(centerWorldPosition[0], centerWorldPosition[1]);

        var radius = this.fov * 0.5 * this.ratio;
        this.wasm.queryDisc(norder, lon, lat, radius).forEach(x => pixList.push(Number(x)));

        return pixList;
    };

    View.prototype.hideSelectedObjects = function() {
        this.aladin.measurementTable.hide();

        if (this.selectedObjects) {
            this.selectedObjects.forEach((objList) => {
                objList.forEach((o) => o.deselect())
            });

            this.selectedObjects = null;
        }

        this.requestRedraw();
        //}
    }

    View.prototype.showSelectedObjects = function() {
        this.hideSelectedObjects();

        this.selectedObjects = this.getSelectedObjects();

        if (this.selectedObjects.length > 0) {
            this.selectedObjects.forEach((objListPerCatalog) => {
                objListPerCatalog.forEach((obj) => obj.select())
            });

            let saveTable = false;

            let tables = this.selectedObjects.map((objList) => {
                // Get the catalog containing that list of objects
                let catalog = objList[0].getCatalog();
                
                let rows = objList.map((o) => {
                    if (o instanceof Footprint) {
                        return o.source;
                    } else {
                        return o;
                    }
                });
                let table = {
                    'name': catalog.name,
                    'color': catalog.color,
                    'rows': rows,
                    'fields': catalog.fields,
                    'fieldsClickedActions': catalog.fieldsClickedActions,
                };

                if (catalog.isObsCore && catalog.isObsCore()) {
                    // If the source is obscore, save the table state inside the measurement table
                    // This is used to go back from a possible datalink table to the obscore one
                    saveTable = true;
                }

                return table;
            })

            this.aladin.measurementTable.showMeasurement(tables, {save: saveTable});
        }
    }

    View.prototype.getVisibleCells = function (norder) {
        return this.wasm.getVisibleCells(norder);
    };

    // Called for touchmove events
    // initialAccDelta must be consistent with fovDegrees here
    View.prototype.setZoom = function (fovDegrees) {
        this.wasm.setFieldOfView(fovDegrees);
        this.updateZoomState();
    };

    View.prototype.increaseZoom = function (amount) {
        const si = 500000.0;
        const alpha = 40.0;

        let initialAccDelta = this.pinchZoomParameters.initialAccDelta + amount;
        let new_fov = si / Math.pow(initialAccDelta, alpha);

        if (new_fov < 0.00002777777) {
            new_fov = 0.00002777777;
        }

        this.pinchZoomParameters.initialAccDelta = initialAccDelta;
        this.setZoom(new_fov);
    }

    View.prototype.decreaseZoom = function (amount) {
        const si = 500000.0;
        const alpha = 40.0;

        let initialAccDelta = this.pinchZoomParameters.initialAccDelta - amount;

        if (initialAccDelta <= 0.0) {
            initialAccDelta = 1e-3;
        }

        let new_fov = si / Math.pow(initialAccDelta, alpha);

        if (new_fov >= this.fovLimit) {
            new_fov = this.fovLimit;
        }

        this.pinchZoomParameters.initialAccDelta = initialAccDelta;
        this.setZoom(new_fov);
    }

    View.prototype.setRotation = function(rotation) {
        this.wasm.setRotationAroundCenter(rotation);
    }

    View.prototype.setGridConfig = function (gridCfg) {
        this.wasm.setGridConfig(gridCfg);

        // send events
        if (gridCfg) {
            if (gridCfg.hasOwnProperty('enabled')) {
                if (gridCfg.enabled === true) {
                    ALEvent.COO_GRID_ENABLED.dispatchedTo(this.aladinDiv);
                }
                else {
                    ALEvent.COO_GRID_DISABLED.dispatchedTo(this.aladinDiv);
                }
            }
            if (gridCfg.color) {
                ALEvent.COO_GRID_UPDATED.dispatchedTo(this.aladinDiv, { color: gridCfg.color, opacity: gridCfg.opacity });
            }
        }

        this.requestRedraw();
    };

    View.prototype.updateZoomState = function () {
        // Get the new zoom values from the backend
        this.zoomFactor = this.wasm.getClipZoomFactor();
        let fov = this.wasm.getFieldOfView();

        // Update the pinch zoom parameters consequently
        const si = 500000.0;
        const alpha = 40.0;
        this.pinchZoomParameters.initialAccDelta = Math.pow(si / fov, 1.0 / alpha);

        // Save it
        this.fov = fov;
        this.computeNorder();

        // Update the lower left FoV div
        if (isNaN(this.fov)) {
            this.fovDiv.html("FoV:");
            return;
        }
        var fovStr;
        /*if (this.projection == ProjectionEnum.SIN && fov >= 180.0) {
            fov = 180.0;
        } else if (fov >= 360.0) {
            fov = 360.0;
        }*/
        if (this.projection.fov <= fov) {
            fov = this.projection.fov;
        }

        if (fov > 1) {
            fovStr = Math.round(fov * 100) / 100 + "°";
        }
        else if (fov * 60 > 1) {
            fovStr = Math.round(fov * 60 * 100) / 100 + "'";
        }
        else {
            fovStr = Math.round(fov * 3600 * 100) / 100 + '"';
        }
        this.fovDiv.html("FoV: " + fovStr);
    };

    /**
     * compute and set the norder corresponding to the current view resolution
     */
    View.prototype.computeNorder = function () {
        var norder = this.wasm.getNOrder();

        this.realNorder = norder;
        // here, we force norder to 3 (otherwise, the display is "blurry" for too long when zooming in)
        if (this.fov <= 50 && norder <= 2) {
            norder = 3;
        }

        this.curNorder = norder;
    };

    View.prototype.untaintCanvases = function () {
        this.createCanvases();
        createListeners(this);
        this.fixLayoutDimensions();
    };

    View.prototype.setOverlayImageLayer = function (imageLayer, layer = "overlay") {
        // register its promise
        this.imageLayersBeingQueried.set(layer, imageLayer);

        this.addImageLayer(imageLayer, layer);

        return imageLayer;
    };

    View.prototype.addLayer = function(imageLayer) {
        const layerName = imageLayer.layer;
        // Check whether this layer already exist
        const idxOverlayLayer = this.overlayLayers.findIndex(overlayLayer => overlayLayer == layerName);
        if (idxOverlayLayer == -1) {
            this.overlayLayers.push(layerName);
        }

        if (this.options.log) {
            Logger.log("setImageLayer", imageLayer.url);
        }

        // Find the toppest layer
        const toppestLayer = this.overlayLayers[this.overlayLayers.length - 1];
        this.selectedLayer = toppestLayer;

        // Remove the existant layer if there is one
        let existantImageLayer = this.imageLayers.get(layerName);
        if (existantImageLayer) {
            existantImageLayer.added = false;
        }

        this.imageLayers.set(layerName, imageLayer);
        ALEvent.HIPS_LAYER_ADDED.dispatchedTo(this.aladinDiv, { layer: imageLayer });
    }

    View.prototype.addImageLayer = function (imageLayer, layer) {
        let self = this;
        // start the query
        const imageLayerPromise = imageLayer.query;

        this.promises.push(imageLayerPromise);

        // All image layer promises must be completed (fullfilled or rejected)
        Promise.allSettled(this.promises)
            .then(() => imageLayerPromise)
            // The promise is resolved and we now have access
            // to the image layer objet (whether it is an ImageSurvey or an ImageFITS)
            .then((imageLayer) => {
                // Add to the backend
                const promise = imageLayer.add(layer);

                self.loadingState = true;
                ALEvent.LOADING_STATE.dispatchedTo(this.aladinDiv, { loading: true });

                return promise;
            })
            .then((imageLayer) => {
                // If the image layer has successfuly been added
                this.empty = false;
                if (imageLayer.children) {
                    imageLayer.children.forEach((imageLayer) => {
                        this.addLayer(imageLayer);
                    })
                } else {
                    this.addLayer(imageLayer);
                }
            })
            .catch((e) => {
                throw e;
            })
            .finally(() => {
                // Loading state is over
                self.loadingState = false;
                ALEvent.LOADING_STATE.dispatchedTo(this.aladinDiv, { loading: false });

                self.imageLayersBeingQueried.delete(layer);

                // Remove the settled promise
                let idx = this.promises.findIndex(p => p == imageLayerPromise);
                this.promises.splice(idx, 1);

                const noMoreLayersToWaitFor = this.promises.length === 0;

                if (noMoreLayersToWaitFor) {
                    if (self.empty) {
                        // no promises to launch!
                        const idxServiceUrl = Math.round(Math.random());
                        const dssUrl = Aladin.DEFAULT_OPTIONS.surveyUrl[idxServiceUrl]

                        self.aladin.setBaseImageLayer(dssUrl);
                    } else {
                        // there is surveys that have been queried
                        // rename the first overlay layer to "base"
                        self.renameLayer(this.overlayLayers[0], "base");
                    }
                }
            })
    }

    // The survey at layer must have been added to the view!
    View.prototype.renameLayer = function(layer, newLayer) {
        if (layer === newLayer) {
            return;
        }

        // Throw an exception if either the first or the second layers are not in the stack
        this.wasm.renameLayer(layer, newLayer);

        let imageLayer = this.imageLayers.get(layer);
        imageLayer.layer = newLayer;

        // Change in overlaylayers
        const idx = this.overlayLayers.findIndex(overlayLayer => overlayLayer == layer);
        this.overlayLayers[idx] = newLayer;
        // Change in imageLayers
        this.imageLayers.delete(layer);
        this.imageLayers.set(newLayer, imageLayer);

        // Change the selected layer if this is the one renamed
        if (this.selectedLayer === layer) {
            this.selectedLayer = newLayer;
        }

        // Tell the layer hierarchy has changed
        ALEvent.HIPS_LAYER_RENAMED.dispatchedTo(this.aladinDiv, { layer: layer, newLayer: newLayer });
    }

    View.prototype.swapLayers = function(firstLayer, secondLayer) {
        // Throw an exception if either the first or the second layers are not in the stack
        this.wasm.swapLayers(firstLayer, secondLayer);

        // Swap in overlaylayers
        const idxFirstLayer = this.overlayLayers.findIndex(overlayLayer => overlayLayer == firstLayer);
        const idxSecondLayer = this.overlayLayers.findIndex(overlayLayer => overlayLayer == secondLayer);

        const tmp = this.overlayLayers[idxFirstLayer];
        this.overlayLayers[idxFirstLayer] = this.overlayLayers[idxSecondLayer];
        this.overlayLayers[idxSecondLayer] = tmp;

        // Tell the layer hierarchy has changed
        ALEvent.HIPS_LAYER_SWAP.dispatchedTo(this.aladinDiv, { firstLayer: firstLayer, secondLayer: secondLayer });
    }

    View.prototype.removeImageLayer = function (layer) {
        // Get the survey to remove to dissociate it from the view
        let imageLayer = this.imageLayers.get(layer);
        if (imageLayer === undefined) {
            // there is nothing to remove
            return;
        }

        // Update the backend
        if (imageLayer.added) {
            this.wasm.removeLayer(layer);
        }

        // Get the survey to remove to dissociate it from the view
        imageLayer.added = false;

        const idxOverlaidLayer = this.overlayLayers.findIndex(overlaidLayer => overlaidLayer == layer);
        if (idxOverlaidLayer == -1) {
            // layer not found
            return;
        }

        // Delete it
        this.imageLayers.delete(layer);

        // Remove it from the layer stack
        this.overlayLayers.splice(idxOverlaidLayer, 1);

        if (this.overlayLayers.length === 0) {
            this.empty = true;
            this.selectedLayer = "base";
        } else {
            // find the toppest layer
            if (this.selectedLayer === layer) {
                const toppestLayer = this.overlayLayers[this.overlayLayers.length - 1];
                this.selectedLayer = toppestLayer;
            }
        }

        ALEvent.HIPS_LAYER_REMOVED.dispatchedTo(this.aladinDiv, { layer: layer });

        // check if there are no more surveys
        const noMoreLayersToWaitFor = this.promises.length === 0;
        if (noMoreLayersToWaitFor && this.empty) {
            // no promises to launch!
            const idxServiceUrl = Math.round(Math.random());
            const dssUrl = Aladin.DEFAULT_OPTIONS.surveyUrl[idxServiceUrl]
            this.aladin.setBaseImageLayer(dssUrl);
        }
    };

    View.prototype.setHiPSUrl = function (pastUrl, newUrl) {
        try {
            this.wasm.setHiPSUrl(pastUrl, newUrl);
        } catch(e) {
            console.error(e)
        }
    }

    View.prototype.getImageLayer = function (layer = "base") {
        let imageLayerQueried = this.imageLayersBeingQueried.get(layer);
        let imageLayer = this.imageLayers.get(layer);

        return imageLayerQueried || imageLayer;
    };

    View.prototype.requestRedraw = function () {
        this.needRedraw = true;
    };

    View.prototype.setProjection = function (projectionName) {
        this.fovLimit = 1000.0;
        /*
            TAN: {id: 1, fov: 180},
            STG: {id: 2, fov: 360},
            SIN: {id: 3, fov: 180},
            ZEA: {id: 4, fov: 360},
            FEYE: {id: 5, fov: 190},
            AIR: {id: 6, fov: 360},
            //AZP: {fov: 180},
            ARC: {id: 7, fov: 360},
            NCP: {id: 8, fov: 180},
            // Cylindrical
            MER: {id: 9, fov: 360},
            CAR: {id: 10, fov: 360},
            CEA: {id: 11, fov: 360},
            CYP: {id: 12, fov: 360},
            // Pseudo-cylindrical
            AIT: {id: 13, fov: 360},
            PAR: {id: 14, fov: 360},
            SFL: {id: 15, fov: 360},
            MOL: {id: 16, fov: 360},
            // Conic
            COD: {id: 17, fov: 360},
            // Hybrid
            HPX: {id: 19, fov: 360},
        */
        switch (projectionName) {
            // Zenithal (TAN, STG, SIN, ZEA, FEYE, AIR, AZP, ARC, NCP)
            case "TAN":
                this.projection = ProjectionEnum.TAN;
                this.fovLimit = 180.0;
                break;
            case "STG":
                this.projection = ProjectionEnum.STG;
                break;
            case "SIN":
                this.projection = ProjectionEnum.SIN;
                break;
            case "ZEA":
                this.projection = ProjectionEnum.ZEA;
                break;
            case "FEYE":
                this.projection = ProjectionEnum.FEYE;
                break;
            case "AIR":
                this.projection = ProjectionEnum.AIR;
                break;
            case "ARC":
                this.projection = ProjectionEnum.ARC;
                break;
            case "NCP":
                this.projection = ProjectionEnum.NCP;
                break;
            case "ARC":
                this.projection = ProjectionEnum.ARC;
                break;
            case "ZEA":
                this.projection = ProjectionEnum.ZEA;
                break;
            // Pseudo-cylindrical (AIT, MOL, PAR, SFL)
            case "MOL":
                this.projection = ProjectionEnum.MOL;
                break;
            case "AIT":
                this.projection = ProjectionEnum.AIT;
                break;
            case "PAR":
                this.projection = ProjectionEnum.PAR;
                break;
            case "SFL":
                this.projection = ProjectionEnum.SFL;
                break;
            // Cylindrical (MER, CAR, CEA, CYP)
            case "MER":
                this.projection = ProjectionEnum.MER;
                this.fovLimit = 360.0;
                break;
            case "CAR":
                this.projection = ProjectionEnum.CAR;
                this.fovLimit = 360.0;
                break;
            case "CEA":
                this.projection = ProjectionEnum.CEA;
                this.fovLimit = 360.0;
                break;
            case "CYP":
                this.projection = ProjectionEnum.CYP;
                this.fovLimit = 360.0;
                break;
            // Conic
            case "COD":
                this.projection = ProjectionEnum.COD;
                break;
            // Hybrid
            case "HPX":
                this.projection = ProjectionEnum.HPX;
                this.fovLimit = 360.0;
                break;
            default:
                break;
        }
        // Change the projection here
        this.wasm.setProjection(projectionName);
        this.updateZoomState();

        this.requestRedraw();
    };

    View.prototype.changeFrame = function (cooFrame) {
        this.cooFrame = cooFrame;

        // Set the new frame to the backend
        if (this.cooFrame.system == CooFrameEnum.SYSTEMS.GAL) {
            this.wasm.setCooSystem(Aladin.wasmLibs.core.CooSystem.GAL);
        }
        else if (this.cooFrame.system == CooFrameEnum.SYSTEMS.J2000) {
            this.wasm.setCooSystem(Aladin.wasmLibs.core.CooSystem.ICRS);
        }

        // Set the grid label format
        if (this.cooFrame.label == "J2000d") {
            this.setGridConfig({fmt: "HMS"});
        }
        else {
            this.setGridConfig({fmt: "DMS"});
        }

        // Get the new view center position (given in icrs)
        let [ra, dec] = this.wasm.getCenter();
        this.viewCenter.lon = ra;
        this.viewCenter.lat = dec;
        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        this.requestRedraw();
    };

    View.prototype.showHealpixGrid = function (show) {
        this.displayHpxGrid = show;

        if (!this.displayHpxGrid) {
            this.mustClearCatalog = true;
        }

        this.requestRedraw();
    };

    View.prototype.showSurvey = function (show) {
        this.getImageLayer().setAlpha(show ? 1.0 : 0.0);

        this.requestRedraw();
    };

    View.prototype.showCatalog = function (show) {
        this.displayCatalog = show;

        if (!this.displayCatalog) {
            this.mustClearCatalog = true;
        }
        this.requestRedraw();
    };

    View.prototype.showReticle = function (show) {
        this.displayReticle = show;

        if (!this.displayReticle) {
            this.mustClearCatalog = true;
        }

        this.requestRedraw();
    };

    /**
     *
     * @API Point to a specific location in ICRS
     *
     * @param ra ra expressed in ICRS J2000 frame
     * @param dec dec expressed in ICRS J2000 frame
     * @param options
     *
     */
    View.prototype.pointTo = function (ra, dec, options) {
        options = options || {};
        ra = parseFloat(ra);
        dec = parseFloat(dec);

        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        this.viewCenter.lon = ra;
        this.viewCenter.lat = dec;
        this.location.update(this.viewCenter.lon, this.viewCenter.lat, this.cooFrame, true);

        // Put a javascript code here to do some animation
        this.wasm.setCenter(this.viewCenter.lon, this.viewCenter.lat);

        this.requestRedraw();

        var self = this;
        setTimeout(function () { self.refreshProgressiveCats(); }, 1000);

        // Apply position changed callback after the move
        self.throttledPositionChanged();
    };
    View.prototype.makeUniqLayerName = function (name) {
        if (!this.layerNameExists(name)) {
            return name;
        }
        for (var k = 1; ; ++k) {
            var newName = name + '_' + k;
            if (!this.layerNameExists(newName)) {
                return newName;
            }
        }
    };
    View.prototype.layerNameExists = function (name) {
        var c = this.allOverlayLayers;
        for (var k = 0; k < c.length; k++) {
            if (name == c[k].name) {
                return true;
            }
        }
        return false;
    };

    View.prototype.removeLayers = function () {
        this.catalogs = [];
        this.overlays = [];
        this.mocs = [];
        this.allOverlayLayers = [];
        this.requestRedraw();
    };

    View.prototype.removeLayer = function (layer) {
        let indexToDelete = this.allOverlayLayers.indexOf(layer);
        this.allOverlayLayers.splice(indexToDelete, 1);

        if (layer.type == 'catalog' || layer.type == 'progressivecat') {
            indexToDelete = this.catalogs.indexOf(layer);
            this.catalogs.splice(indexToDelete, 1);
        }
        else if (layer.type == 'moc') {
            indexToDelete = this.mocs.indexOf(layer);

            let moc = this.mocs.splice(indexToDelete, 1);
            // remove from aladin lite backend
            moc[0].delete();
        }
        else if (layer.type == 'overlay') {
            indexToDelete = this.overlays.indexOf(layer);
            this.overlays.splice(indexToDelete, 1);
        }

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.dispatchedTo(this.aladinDiv, { layer: layer });

        this.requestRedraw();
    };

    View.prototype.addCatalog = function (catalog) {
        catalog.name = this.makeUniqLayerName(catalog.name);
        this.allOverlayLayers.push(catalog);
        this.catalogs.push(catalog);
        if (catalog.type == 'catalog') {
            catalog.setView(this);
        }
        else if (catalog.type == 'progressivecat') {
            catalog.init(this);
        }
    };

    View.prototype.addOverlay = function (overlay) {
        overlay.name = this.makeUniqLayerName(overlay.name);
        this.overlays.push(overlay);
        this.allOverlayLayers.push(overlay);
        overlay.setView(this);
    };

    View.prototype.addMOC = function (moc) {
        moc.name = this.makeUniqLayerName(moc.name);
        moc.setView(this);
    };

    View.prototype.getSelectedObjects = function () {
        let {x, y, w, h} = this.selector.getBBox(this.dragCoo);

        if (w < 0) {
            x = x + w;
            w = -w;
        }
        if (h < 0) {
            y = y + h;
            h = -h;
        }
        var objList = [];
        var cat, sources, s;
        var footprints, f;
        var objListPerCatalog = [];
        if (this.catalogs) {
            for (var k = 0; k < this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l = 0; l < sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }
                    if (s.x >= x && s.x <= x + w && s.y >= y && s.y <= y + h) {
                        objListPerCatalog.push(s);
                    }
                }
                // footprints
                footprints = cat.getFootprints();
                if (footprints) {
                    for (var l = 0; l < footprints.length; l++) {
                        f = footprints[l];
                        if (f.intersectsBBox(x, y, w, h, this)) {
                            objListPerCatalog.push(f);
                        }
                    }
                }

                if (objListPerCatalog.length > 0) {
                    objList.push(objListPerCatalog);
                }
                objListPerCatalog = [];
            }
        }
        return objList;

    };

    // update objLookup, lookup table
    View.prototype.updateObjectsLookup = function () {
        this.objLookup = [];

        var cat, sources, s, xRounded, yRounded;
        if (this.catalogs) {
            for (var k = 0; k < this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l = 0; l < sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y || cat.readOnly) {
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

    View.prototype.closestFootprints = function (footprints, ctx, x, y) {
        if (!footprints) {
            return null;
        }

        let closest = null;

        footprints.forEach((footprint) => {
            // Hidden footprints are not considered
            let lineWidth = footprint.getLineWidth();

            footprint.setLineWidth(6.0);
            if (footprint.isShowing && footprint.isInStroke(ctx, this, x, y)) {
                closest = footprint;
            }
            footprint.setLineWidth(lineWidth);

            if (closest) {
                return closest;
            }
        })

        return closest;
    };

    // return closest object within a radius of maxRadius pixels. maxRadius is an integer
    View.prototype.closestObjects = function (x, y, maxRadius) {
        // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
        var overlay;
        var canvas = this.catalogCanvas;
        var ctx = canvas.getContext("2d");
        // this makes footprint selection easier as the catch-zone is larger
        //let pastLineWidth = ctx.lineWidth;

        if (this.overlays) {
            for (var k = 0; k < this.overlays.length; k++) {
                overlay = this.overlays[k];

                let closest = this.closestFootprints(overlay.overlayItems, ctx, x, y);
                if (closest) {
                    //ctx.lineWidth = pastLineWidth;
                    return [closest];
                }
            }
        }

        // Catalogs can also have footprints
        if (this.catalogs) {
            for (var k = 0; k < this.catalogs.length; k++) {
                let catalog = this.catalogs[k];

                let closest = this.closestFootprints(catalog.footprints, ctx, x, y);
                if (closest) {
                    //ctx.lineWidth = pastLineWidth;
                    return [closest];
                }
            }
        }

        if (!this.objLookup) {
            //ctx.lineWidth = pastLineWidth;
            return null;
        }

        //ctx.lineWidth = pastLineWidth;

        var closest, dist;
        for (var r = 0; r <= maxRadius; r++) {
            closest = dist = null;
            for (var dx = -maxRadius; dx <= maxRadius; dx++) {
                if (!this.objLookup[x + dx]) {
                    continue;
                }
                for (var dy = -maxRadius; dy <= maxRadius; dy++) {
                    if (this.objLookup[x + dx][y + dy]) {
                        var d = dx * dx + dy * dy;
                        if (!closest || d < dist) {
                            closest = this.objLookup[x + dx][y + dy];
                            dist = d;
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
