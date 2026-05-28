// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
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
import { HealpixGrid } from "./HealpixGrid.js";
import { ProjectionEnum } from "./ProjectionEnum.js";
import { Utils } from "./Utils";
import { GenericPointer } from "./GenericPointer.js";
import { Stats } from "./libs/Stats.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";
import { WebGLCtx } from "./WebGL.js";
import { ALEvent } from "./events/ALEvent.js";
import { Zoom } from './Zoom.js'
import { Footprint } from "./Footprint.js";
import { Selector } from "./Selector.js";
import { ObsCore } from "./vo/ObsCore.js";
import { HiPS } from "./HiPS.js";
import { Image } from "./Image.js";
import { Color } from "./Color.js";
import { SpectraDisplayer } from "./SpectraDisplayer.js";
import { DefaultActionsForContextMenu } from "./DefaultActionsForContextMenu.js";
import { Source } from "./Source.js";
import { blobToArrayBuffer, blobToDataURL, buildAvmFromWcs, injectAvmIntoPngBlob } from "./AvmUtils.js";
export let View = (function () {

    const normalizeImageType = function (imgType) {
        return imgType || "image/png";
    };

    const canvasToBlob = function (canvas, imgType) {
        return new Promise((resolve, reject) => {
            canvas.toBlob((blob) => {
                if (blob) {
                    resolve(blob);
                } else {
                    reject(new Error("Canvas toBlob failed"));
                }
            }, imgType);
        });
    };

    const exportCanvasBlob = async function (view, canvas, imgType) {
        const effectiveImgType = normalizeImageType(imgType);
        let blob = await canvasToBlob(canvas, effectiveImgType);

        if (effectiveImgType !== "image/png") {
            return blob;
        }

        const wcs = view.aladin.getViewWCS();
        if (!wcs || typeof wcs !== "object") {
            return blob;
        }

        const avm = buildAvmFromWcs(wcs, {
            rotation: view.aladin.getRotation(),
        });

        if (!avm) {
            return blob;
        }

        try {
            blob = await injectAvmIntoPngBlob(blob, avm);
        } catch (error) {
            console.warn("Could not inject AVM metadata into PNG export", error);
        }

        return blob;
    };

    /** Constructor */
    function View(aladin) {
        this.aladin = aladin;
        // Add a reference to the WebGL API
        this.options = aladin.options;
        this.aladinDiv = this.aladin.aladinDiv;
        this.createCanvases();

        let self = this;

        // reference to all overlay layers (= catalogs + overlays + mocs)
        this.allOverlayLayers = []
        // current catalogs displayed
        this.catalogs = [];
        // overlays (footprints for instance)
        this.overlays = [];
        // MOCs
        this.mocs = [];

        this.outsideFov = 0;
        this.outside = false;

        self.redrawClbk = this.redraw.bind(this);
        // Init the WebGL context
        // At this point, the view has been created so the image canvas too
        try {
            // Start our Rust application. You can find `WebClient` in `src/lib.rs`
            // The Rust part should also create a new WebGL2 or WebGL1 context depending on the WebGL2 brower support.
            const webglCtx = new WebGLCtx(Aladin.wasmLibs.core, this.aladinDiv);
            this.aladin.wasm = webglCtx.webclient;
            this.wasm = this.aladin.wasm;

            ALEvent.AL_USE_WASM.listenedBy(this.aladinDiv, function (e) {
                let callback = e.detail.callback;

                callback(self.wasm);
            });
        } catch (e) {
            // For browsers not supporting WebGL2:
            // 1. Print the original exception message in the console
            //console.error(e)
            // 2. Add a more explicite message to the end user
            console.error("Problem initializing Aladin Lite. Please contact the support by contacting Matthieu Baumann (matthieu.baumann@astro.unistra.fr) or Thomas Boch (thomas.boch@astro.unistra.fr). You can also open an issue on the Aladin Lite github repository here: https://github.com/cds-astro/aladin-lite. Message error:" + e)
        }

        this._defineProperties();

        // I realized debouncing a little bit the resize process
        // prevents the div from flickering in white !
        // Let's keep that like that (with a very small debounced time of 2ms).
        this.debounceResize = Utils.debounce(() => {
            self.wasm.resize(self.width, self.height);
            self.updateZoomState()

            if (self.spectraDisplayer) {
                self.spectraDisplayer.updateCanvas()
                self.spectraDisplayer._redraw()
            }
        }, 2);

        // Attach the drag and drop events to the view
        this.aladinDiv.ondrop = (event) => {
            const files = Utils.getDroppedFilesHandler(event);

            files.forEach((file) => {
                const url = URL.createObjectURL(file);

                // Consider other cases
                try {
                    const image = self.aladin.createImageFITS(
                        url,
                        {name: file.name},
                        (ra, dec, fov, _) => {
                            // Center the view around the new fits object
                            aladin.gotoRaDec(ra, dec);
                            aladin.setFoV(fov * 1.1);
                        }
                    );
                    self.aladin.setOverlayImageLayer(image, file.name)
                } catch(e) {
                    let moc = A.MOCFromURL(url);
                    self.aladin.addMOC(moc);

                    console.error("Only valid fits files supported (i.e. containig a WCS)", e)
                    throw e;
                }
            })
        };

        this.aladinDiv.ondragover = Utils.dragOverHandler;

        this.throttledPositionChanged = Utils.throttle(
            (dragging) => {
                var posChangedFn = this.aladin.callbacksByEventName && this.aladin.callbacksByEventName['positionChanged'];
                if (typeof posChangedFn === 'function') {
                    var pos = this.aladin.pix2world(this.width / 2, this.height / 2, 'icrs');
                    if (pos !== undefined) {
                        try {
                            posChangedFn({
                                ra: pos[0],
                                dec: pos[1],
                                dragging: dragging,
                                frame: 'ICRS'
                            });
                        } catch(e) {
                            console.error(e)
                        }
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

        this.throttledDivResized = Utils.throttle(
            () => {
                const resizeFn = self.aladin.callbacksByEventName['resizeChanged'];
                (typeof resizeFn === 'function') && resizeFn(self.width, self.height);
            },
            View.CALLBACKS_THROTTLE_TIME_MS,
        );

        this.debounceProgCatOnZoom = Utils.debounce(() => {
            self.refreshProgressiveCats();
            self.drawAllOverlays();
        }, 300);

        this.mustClearCatalog = true;
        this.mode = View.PAN;

        this.healpixGrid = new HealpixGrid();
        this.then = Date.now();

        var lon, lat;
        lon = lat = 0;

        // FoV init settings
        this.pinchZoomParameters = {
            isPinching: false, // true if a pinch zoom is ongoing
            initialZoomFactor: undefined,
            initialDistance: undefined,
        };

        // Projection definition
        const projName = (this.options && this.options.projection) || "SIN";
        this.setProjection(projName)

        // Then set the zoom properly once the projection is defined
        this.fov = this.options.fov || 180.0

        // Target position settings
        this.viewCenter = { ra: lon, dec: lat }; // position of center of view always in ICRS

        // Coo frame setting
        const cooFrame = CooFrameEnum.fromString(this.options.cooFrame, CooFrameEnum.ICRS);
        this.changeFrame(cooFrame);

        this.selector = new Selector(this, this.options.selector);
        this.manualSelection = (this.options && this.options.manualSelection) || false;

        // Selection mode
        this.selectionMode = View.SELECTION_MODE_EDGE;
        if (this.options.selectionMode === 'skewer') {
            this.selectionMode = View.SELECTION_MODE_SKEWER;
        }


        // current reference image survey displayed
        this.imageLayers = new Map();

        this.overlayLayers = [];
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
        this.catalogForPopup.view = this;
        this.overlayForPopup = A.graphicOverlay({color: '#ee2345', lineWidth: 3});
        this.overlayForPopup.hide();
        this.overlayForPopup.view = this;

        this.empty = true;

        this.promises = [];
        this.firstHiPS = true;
        this.curNorder = 1;
        this.realNorder = 1;
        this.imageLayersBeingQueried = new Map();

        // some variables for mouse handling
        this.dragging = false;
        this.dragCoo = null;
        this.selectedLayer = undefined;

        this.needRedraw = true;

        // two-fingers rotation
        this.fingersRotationParameters = {
            initialViewAngleFromCenter: undefined,
            initialFingerAngle: undefined,
            rotationInitiated: false
        }
        this.zoom = new Zoom(this);

        this.fadingLatestUpdate = null;
        this.dateRequestRedraw = null;

        let colorPickerElement = document.getElementById('aladin-picker-tooltip');
        if (!colorPickerElement) {
            colorPickerElement = document.createElement('span');
            colorPickerElement.classList.add('aladin-color-picker')
            colorPickerElement.classList.add('aladin-view-label')

            this.aladin.aladinDiv.appendChild(colorPickerElement);
        }

        this.colorPickerTool = {
            domElement: colorPickerElement,
            probedValue: null
        };

        init(this);
        // listen to window resize and reshape canvases
        this.resizeTimer = null;

        this.resizeObserver = new ResizeObserver(() => {
            self.fixLayoutDimensions();
        });

        self.resizeObserver.observe(this.aladinDiv)

        self.fixLayoutDimensions();

        self.redraw()
    };

    // different available modes
    View.PAN = 0;
    View.SELECT = 1;
    View.TOOL_SIMBAD_POINTER = 2;
    View.TOOL_COLOR_PICKER = 3;

    // Selection modes
    View.SELECTION_MODE_EDGE = 0;
    View.SELECTION_MODE_SKEWER = 1;

    // TODO: should be put as an option at layer level
    View.DRAW_SOURCES_WHILE_DRAGGING = true;
    View.DRAW_MOCS_WHILE_DRAGGING = true;

    View.CALLBACKS_THROTTLE_TIME_MS = 100; // minimum time between two consecutive callback calls


    View.prototype._defineProperties = function() {
        Object.defineProperties(this, {
            fov: {
                get() {
                    return this.wasm.getFieldOfView()[0];
                },
                set(newFov) {
                    this.setFoV(newFov);
                }
            },
            zoomFactor: {
                get() {
                    return this.wasm.getZoomFactor();
                },
                set(newZoomFactor) {
                    this.setZoomFactor(newZoomFactor);
                }
            }
        });
    }

    // (re)create needed canvases
    View.prototype.createCanvases = function () {
        let imageCanvas = this.aladinDiv.querySelector('.aladin-imageCanvas');
        if (imageCanvas) {
            imageCanvas.remove();
        }

        /*let gridCanvas = this.aladinDiv.querySelector('.aladin-gridCanvas');
        if (gridCanvas) {
            gridCanvas.remove();
        }*/

        let catalogCanvas = this.aladinDiv.querySelector('.aladin-catalogCanvas')
        if (catalogCanvas) {
            catalogCanvas.remove();
        }

        // canvas to draw the images
        let createCanvas = (name) => {
            // Create a new canvas element
            let canvas = document.createElement('canvas');
            canvas.className = name;

            // Append the canvas to the aladinDiv
            this.aladinDiv.insertBefore(canvas, this.aladinDiv.firstChild);

            return canvas;
        };

        this.catalogCanvas = createCanvas('aladin-catalogCanvas');
        //this.gridCanvas = createCanvas('aladin-gridCanvas');
        this.imageCanvas = createCanvas('aladin-imageCanvas');
    };

    View.prototype.setFoVRange = function(minFoV, maxFoV) {
        this.wasm.setFoVRange(minFoV, maxFoV)
        this.updateZoomState();
    }

    View.prototype.getFoVRange = function() {
        let [minFoV, maxFoV] = this.wasm.getFoVRange();
        if (minFoV == -1.0) {
            minFoV = null;
        }

        if (maxFoV == -1.0) {
            maxFoV = null;
        }

        return [minFoV, maxFoV]
    }

    // called at startup and when window is resized
    // The WebGL backend is resized
    View.prototype.fixLayoutDimensions = function () {
        // make de line height at 0. This prevents the resize observer to infinitely
        // trigger over and over.
        this.aladinDiv.style.setProperty('line-height', 0);
        Utils.cssScale = undefined;

        var computedWidth = Math.floor(parseFloat(this.aladinDiv.getBoundingClientRect().width)) || 1.0;
        var computedHeight = Math.floor(parseFloat(this.aladinDiv.getBoundingClientRect().height)) || 1.0;

        this.width = Math.max(computedWidth, 1);
        this.height = Math.max(computedHeight, 1); // this prevents many problems when div size is equal to 0

        this.cx = this.width / 2;
        this.cy = this.height / 2;

        this.largestDim = Math.max(this.width, this.height);
        this.smallestDim = Math.min(this.width, this.height);
        this.ratio = this.largestDim / this.smallestDim;

        this.mouseMoveIncrement = 160 / this.largestDim;

        // reinitialize 2D context

        this.catalogCtx = this.catalogCanvas.getContext("2d");
        this.catalogCtx.canvas.width = this.width * window.devicePixelRatio;
        this.catalogCtx.canvas.height = this.height * window.devicePixelRatio;
        this.catalogCtx.canvas.style.width = this.width + "px";
        this.catalogCtx.canvas.style.height = this.height + "px";
        this.catalogCtx.scale(window.devicePixelRatio, window.devicePixelRatio);

        this.imageCtx = this.imageCanvas.getContext("webgl2");
        this.imageCtx.canvas.style.width = this.width + "px";
        this.imageCtx.canvas.style.height = this.height + "px";

        this.debounceResize();

        pixelateCanvasContext(this.imageCtx, this.aladin.options.pixelateCanvas);

        // change logo
        if (!this.logoDiv) {
            this.logoDiv = this.aladinDiv.querySelector('.aladin-logo');
        }

        if (this.logoDiv) {
            if (this.width > 800) {
                this.logoDiv.classList.remove('aladin-logo-small');
                this.logoDiv.classList.add('aladin-logo-large');
                this.logoDiv.style.width = '90px';
            }
            else {
                this.logoDiv.classList.add('aladin-logo-small');
                this.logoDiv.classList.remove('aladin-logo-large');
                this.logoDiv.style.width = '32px';
            }
        }

        this.computeNorder();

        this.aladinDiv.style.removeProperty('line-height');

        this.throttledDivResized();
    };

    var pixelateCanvasContext = function (ctx, pixelateFlag) {
        var enableSmoothing = !pixelateFlag;
        ctx.imageSmoothingEnabled = enableSmoothing;
        ctx.webkitImageSmoothingEnabled = enableSmoothing;
        ctx.mozImageSmoothingEnabled = enableSmoothing;
        ctx.msImageSmoothingEnabled = enableSmoothing;
        ctx.oImageSmoothingEnabled = enableSmoothing;
    }

    View.prototype.setMode = function (mode, params) {

        // Undo the specialized cursors, if any.
        const prevMode = this.mode;
        if (prevMode == View.TOOL_SIMBAD_POINTER) {
            this.catalogCanvas.classList.remove('aladin-sp-cursor');
        }

        // hide the picker tooltip
        this.colorPickerTool.domElement.style.display = "none";
        // in case we are in the selection mode
        this.requestRedraw();

        this.aladin.removeStatusBarMessage('selector')

        this.mode = mode;

        if (this.mode == View.TOOL_SIMBAD_POINTER) {
            this.aladin.popup.hide();
            this.catalogCanvas.style.cursor = '';
            this.catalogCanvas.classList.add('aladin-sp-cursor');
        }
        else if (this.mode == View.PAN) {
            this.setCursor('default');
        }
        else if (this.mode == View.SELECT) {
            this.setCursor('crosshair');
            this.aladin.showReticle(false)

            const { mode, callback } = params;
            this.selector.start(mode, callback);
        } else if (this.mode == View.TOOL_COLOR_PICKER) {
            this.colorPickerTool.domElement.style.display = "block";
            this.setCursor('crosshair');
            this.aladin.showReticle(false)
        }

        ALEvent.MODE.dispatchedTo(this.aladin.aladinDiv, {mode});
    };

    View.prototype.setSelectionMode = function (selectionMode) {
        this.selectionMode = selectionMode;
    };

    View.prototype.getSelectionMode = function () {
        return this.selectionMode;
    };

    View.prototype.setCursor = function (cursor) {
        if (this.catalogCanvas.style.cursor == cursor) {
            return;
        }

        this.catalogCanvas.style.cursor = cursor;
    };

    View.prototype.getRawPixelsCanvas = function(width, height) {
        // important: redraw must be called just before getting the pixels
        // because the drawing buffer is not preserved (preserveDrawingBuffer = false) 
        this.redraw()

        const canvas = this.wasm.canvas();

        const c = document.createElement('canvas');
        c.width = width || (this.width * window.devicePixelRatio);
        c.height = height || (this.height * window.devicePixelRatio);

        const ctx = c.getContext('2d');

        ctx.drawImage(canvas, 0, 0, c.width, c.height);
        ctx.drawImage(this.catalogCanvas, 0, 0, c.width, c.height);

        return c;
    };

    View.prototype.getCanvas = async function (width, height, withLogo=true) {
        const loadImage = function (url) {
            return new Promise((resolve, reject) => {
                const image = document.createElement("img")
                image.src = url
                image.onload = () => resolve(image)
                image.onerror = () => reject(new Error('could not load image'))
            })
        }

        const c = this.getRawPixelsCanvas(width, height)
        let ctx = c.getContext("2d");

        // draw the reticle if it is on the view
        let reticle = this.aladin.reticle;
        if (reticle.isVisible()) {
            var svgBlob = new Blob([reticle.el.innerHTML], {type: 'image/svg+xml;charset=utf-8'});
            const reticleImg = await loadImage(URL.createObjectURL(svgBlob));

            const posX = (c.width - reticleImg.width)/2.0;
            const posY = (c.height - reticleImg.height)/2.0;
            ctx.drawImage(reticleImg, posX, posY);
        }

        if(withLogo) {
            const logo = await loadImage("data:image/png;base64,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")

            const offX = c.width - logo.width;
            const offY = c.height - logo.height;
            ctx.drawImage(logo, offX, offY);
        }

        return c;
    };

    /**
     * Return dataURL string corresponding to the current view
     */
    View.prototype.getCanvasDataURL = async function (imgType, width, height, withLogo=true) {
        const c = await this.getCanvas(width, height, withLogo);
        const blob = await exportCanvasBlob(this, c, imgType);
        return blobToDataURL(blob);
    };

    /**
     * Return ArrayBuffer corresponding to the current view
     */
    View.prototype.getCanvasArrayBuffer = async function (imgType, width, height, withLogo=true) {
        const blob = await this.getCanvasBlob(imgType, width, height, withLogo);
        return blobToArrayBuffer(blob);
    }

    /**
     * Return Blob corresponding to the current view
     */
    View.prototype.getCanvasBlob = async function (imgType, width, height, withLogo=true) {
        const c = await this.getCanvas(width, height, withLogo);
        return exportCanvasBlob(this, c, imgType);
    }

    View.prototype.selectLayer = function (layer) {
        let imageLayer = this.imageLayers.get(layer)
        if (!imageLayer) {
            console.warn(layer + ' does not exists. So cannot be selected');
            return;
        }

        if (this.spectraDisplayer)
            this.spectraDisplayer._hide();

        if (imageLayer.dataproductType === "spectral-cube") {
            if (!this.spectraDisplayer) {
                this.spectraDisplayer = new SpectraDisplayer(this, {height: 250});
            }

            this.spectraDisplayer.attachHiPS3D(imageLayer)
        }

        this.selectedLayer = layer;
    };

    var createListeners = function (view) {
        if ('virtualKeyboard' in navigator) {
            // The VirtualKeyboard API is supported!
            navigator.virtualKeyboard.overlaysContent = true;
        }

        // various listeners
        let onDblClick = function (e) {
            const xymouse = Utils.relMouseCoords(e);

            // deselect all the selected sources with Select panel
            view.unselectObjects();

            try {
                const [lon, lat] = view.aladin.pix2world(xymouse.x, xymouse.y, 'icrs');
                view.pointTo(lon, lat);
                // reset the rotation around center view
                view.setRotation(0.0);
            }
            catch (err) {
                return;
            }
        };

        if (!Utils.hasTouchScreen()) {
            Utils.on(view.catalogCanvas, 'dblclick', onDblClick);
        }

        // prevent default context menu from appearing (potential clash with right-click cuts control)
        Utils.on(view.catalogCanvas, "contextmenu", function (e) {
            e.preventDefault();

            if (view.aladin.options.showContextMenu) {
                let ctxMenu = view.aladin.contextMenu;
                if(ctxMenu) {
                    e.stopPropagation();
                    if (!view.rightClick && showContextMenu) {
                        ctxMenu.attach(
                            DefaultActionsForContextMenu.getDefaultActions(view.aladin),
                            null
                        );
                        ctxMenu._show({e});
                    }
                }   
            }
        });


        let cutMinInit = null
        let cutMaxInit = null;

        var onlongtouch = function(e) {
            if (view.aladin.statusBar) {
                view.aladin.statusBar.removeMessage('opening-ctxmenu')
            }

            view.aladin.contextMenu && view.aladin.contextMenu._show({e});
        };
        var longTouchTimer;
        var longTouchDuration = 800;
        var showContextMenu = true;
        var xystart;

        var handleSelect = function(xy, tolerance, withModifierKey=false) {
            tolerance = tolerance || 5;
            var objs = view.closestObjects(xy.x, xy.y, tolerance);

            if (objs) {
                var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                var footprintClickedFunction = view.aladin.callbacksByEventName['footprintClicked'];

                let objsByCats = {};
                let shapes = [];
                for (let o of objs) {
                    // classify the different objects by catalog
                    let cat = o.getCatalog && o.getCatalog();
                    if (cat && cat.name) {
                        if (!objsByCats[cat.name]) {
                            objsByCats[cat.name] = []
                        }

                        objsByCats[cat.name].push(o);
                    }

                    (typeof objClickedFunction === 'function') && objClickedFunction(o, xy);

                    if (o.isFootprint()) {
                        if (typeof footprintClickedFunction === 'function') {
                            footprintClickedFunction(o, xy);
                        }
                    }

                    // If this shape has a catalog then it will be selected from its source
                    // so we will not add it
                    if (!cat) {
                        shapes.push(o);
                    }
                }

                // Rewrite objs
                objs = Array.from(Object.values(objsByCats));
                // Add the external shapes (i.e. which are not associated with catalog sources e.g. those from GraphicOverlay)
                if (shapes.length > 0) {
                    objs.push(shapes)
                }
                view.selectObjects(objs, withModifierKey);

                view.lastClickedObject = objs;

            } else {

                view.unselectObjects();

                // If there is a past clicked object
                if (view.lastClickedObject) {
                    // TODO: do we need to keep that triggering ?
                    var objClickedFunction = view.aladin.callbacksByEventName['objectClicked'];
                    (typeof objClickedFunction === 'function') && objClickedFunction(null, xy);

                    view.lastClickedObject = null;
                }
            }
        }

        /**
         * Perform a skewer-based selection of objects at the specified mouse position.
         *
         * @param {Event|Object} e - Mouse coordinate via mouse event or object with x and y properties
         * @param {boolean} withModifierKey - If true, toggles selection (adds/removes from existing selection);
         *                                    if false, replaces current selection
         */
        var handleSkewerSelect = function(e, withModifierKey) {

            const objList = Selector.getSkewerObjects(e, view);
            view.selectObjects(objList, withModifierKey);
        }

        /**
         * Make the selected objects appear hovered and make all other objects appear unhovered,
         * firing the appropriate callbacks for both cases.
         *
         * The also sets the cursor to a pointer to indicate that some object(s) would be
         * select on click in the current mouse position.
         *
         * @param {Array} objects - Array of objects to apply hover state to
         * @param {Object} xymouse - Mouse coordinates with x and y properties
         */
        var hoverObjects = function(objects, xymouse) {
            var objHoveredFunction = view.aladin.callbacksByEventName['objectHovered'];
            var footprintHoveredFunction = view.aladin.callbacksByEventName['footprintHovered'];

            view.setCursor('pointer');

            for (let o of objects) {

                if (typeof objHoveredFunction === 'function' && (!view.lastHoveredObject || !view.lastHoveredObject.includes(o))) {
                    var ret = objHoveredFunction(o, xymouse);
                }

                if (o.isFootprint()) {
                    if (typeof footprintHoveredFunction === 'function' && (!view.lastHoveredObject || !view.lastHoveredObject.includes(o))) {
                        var ret = footprintHoveredFunction(o, xymouse);
                    }
                }

                if (!view.lastHoveredObject || !view.lastHoveredObject.includes(o)) {
                    o.hover();
                }
            }

            // unhover the objects in lastHoveredObjects that are not in closest anymore
            if (view.lastHoveredObject) {
                var objHoveredStopFunction = view.aladin.callbacksByEventName['objectHoveredStop'];

                for (let lho of view.lastHoveredObject) {
                    if (!objects.includes(lho)) {
                        lho.unhover();

                        if (typeof objHoveredStopFunction === 'function') {
                            objHoveredStopFunction(lho, xymouse);
                        }
                    }
                }
            }
            view.lastHoveredObject = objects;
        }

        /**
         * Removes hover state from all previously hovered objects and fires the
         * apropriate callbacks.
         *
         * The also resets the cursor to the default to indicate that no objects would be
         * selected on click in the current mouse position.
         *
         * @param {Object} xymouse - Mouse coordinates with x and y properties
         */
        var unhoverObjects = function(xymouse) {
            view.setCursor('default');
            if (view.lastHoveredObject) {
                var objHoveredStopFunction = view.aladin.callbacksByEventName['objectHoveredStop'];
                for (let lho of view.lastHoveredObject) {
                    lho.unhover();

                    if (typeof objHoveredStopFunction === 'function') {
                        objHoveredStopFunction(lho, xymouse);
                    }
                }
            }

            view.lastHoveredObject = null;
        }

        var touchStartTime;
        Utils.on(view.catalogCanvas, "mousedown touchstart", function (e) {
            e.stopPropagation();

            const xymouse = Utils.relMouseCoords(e);

            if (view.spectraDisplayer) {
                view.spectraDisplayer.disableInteraction();
            }

            ALEvent.CANVAS_EVENT.dispatchedTo(view.aladinDiv, {
                state: {
                    mode: view.mode,
                    dragging: view.dragging,
                },
                type: e.type,
                xy: xymouse,
            });

            xystart = xymouse;

            if (e.which === 3 || e.button === 2) {
                view.rightClick = true;
                showContextMenu = true;

                if (view.selectedLayer) {
                    const imageLayer = view.imageLayers.get(view.selectedLayer);
                    if (imageLayer) {
                        // Take as start cut values what is inside the properties
                        // If the cuts are not defined in the metadata of the survey
                        // then we take what has been defined by the user
                        [cutMinInit, cutMaxInit] = imageLayer.getCuts();
                    }
                }

                return;
            }

            // detect long touch
            if (e.type === 'touchstart' && e.targetTouches && e.targetTouches.length == 1) {
                if (view.aladin.statusBar) {
                    view.aladin.statusBar.appendMessage({
                        id: 'opening-ctxmenu',
                        message: 'Opening menu...',
                        duration: 'unlimited',
                        offsetTimeDisplay: 200, // in ms
                        type: 'loading'
                    })
                }

                longTouchTimer = setTimeout(() => {onlongtouch(e); view.dragging = false;}, longTouchDuration);
                touchStartTime = Date.now();

            }

            // zoom pinching
            if (e.type === 'touchstart' && e.targetTouches && e.targetTouches.length >= 2) {
                view.dragging = false;

                // Do not start the pinched rotation if the north up is locked
                if (view.aladin.lockNorthUp === true) {
                    return;
                }


                view.pinchZoomParameters.isPinching = true;
                view.pinchZoomParameters.initialZoomFactor = view.zoomFactor;
                view.pinchZoomParameters.initialDistance = Math.sqrt(Math.pow(e.targetTouches[0].clientX - e.targetTouches[1].clientX, 2) + Math.pow(e.targetTouches[0].clientY - e.targetTouches[1].clientY, 2));

                view.fingersRotationParameters.initialViewAngleFromCenter = view.wasm.getRotation();
                view.fingersRotationParameters.initialFingerAngle = Math.atan2(e.targetTouches[1].clientY - e.targetTouches[0].clientY, e.targetTouches[1].clientX - e.targetTouches[0].clientX) * 180.0 / Math.PI;

                return;
            }

            view.dragCoo = xymouse;
            view.dragPastCoo = xymouse;

            view.dragging = true;

            view.aladin.contextMenu && view.aladin.contextMenu._hide()

            if (view.mode === View.PAN) {
                view.setCursor('move');
            }

            view.wasm.pressLeftMouseButton();

            if (view.mode === View.SELECT) {
                view.selector.dispatch('mousedown', {coo: xymouse})
            }

            // false disables default browser behaviour like possibility to touch hold for context menu.
            // To disable text selection use css user-select: none instead of putting this value to false
            return true;
        });

        Utils.on(view.catalogCanvas, "click", function (e) {
            // call listener of 'click' event
            if (view.mode == View.TOOL_SIMBAD_POINTER) {
                // call Simbad pointer or Planetary features
                GenericPointer(view, e);

                return; // when in TOOL_SIMBAD_POINTER mode, we do not call the listeners
            }

            if (view.mode == View.TOOL_COLOR_PICKER) {
                Utils.copy2Clipboard(view.colorPickerTool.probedValue)
                    .then(() => {
                        if (view.aladin.statusBar) {
                            view.aladin.statusBar.appendMessage({
                                message: `<span class="aladin-indicator" style="background-color: ${view.colorPickerTool.probedValue}"></span> [${view.colorPickerTool.probedValue}] copied into your clipboard`,
                                duration: 1500,
                                type: 'info'
                            })
                        }
                    })
                return; // listeners are not called
            }
        });

        Utils.on(document, "mouseup touchend", function(e) {
            var wasDragging = view.realDragging === true;

            if (view.dragging) { // if we were dragging, reset to default cursor
                if(view.mode === View.PAN) {
                    view.setCursor('default');
                }

                view.dragging = false;
                if (wasDragging) {
                    view.realDragging = false;

                    // call the positionChanged once more with a dragging = false
                    view.throttledPositionChanged(false);
                }

                if (view.spectraDisplayer) {
                    view.spectraDisplayer.enableInteraction();
                }
            } // end of "if (view.dragging) ... "
        });

        // reacting on 'click' rather on 'mouseup' is more reliable when panning the view
        Utils.on(view.catalogCanvas, "mouseup mouseout touchend touchcancel", function (e) {
            const xymouse = Utils.relMouseCoords(e);
            const withModifierKey = e.ctrlKey || e.metaKey;

            ALEvent.CANVAS_EVENT.dispatchedTo(view.aladinDiv, {
                state: {
                    mode: view.mode,
                    dragging: view.dragging,
                },
                type: e.type,
                ev: e,
            });

            if (e.type === 'touchend' || e.type === 'touchcancel') {
                if (longTouchTimer) {
                    if (view.aladin.statusBar) {
                        view.aladin.statusBar.removeMessage('opening-ctxmenu')
                    }
                    clearTimeout(longTouchTimer)
                    longTouchTimer = undefined;
                }
            }

            if ((e.type === 'touchend' || e.type === 'touchcancel') && view.pinchZoomParameters.isPinching) {
                view.pinchZoomParameters.isPinching = false;
                view.pinchZoomParameters.initialZoomFactor = view.pinchZoomParameters.initialDistance = undefined;

                return;
            }
            if ((e.type === 'touchend' || e.type === 'touchcancel') && view.fingersRotationParameters.rotationInitiated) {
                view.fingersRotationParameters.initialViewAngleFromCenter = undefined;
                view.fingersRotationParameters.initialFingerAngle = undefined;
                view.fingersRotationParameters.rotationInitiated = false;

                return;
            }

            var wasDragging = view.realDragging === true;

            view.mustClearCatalog = true;
            view.dragCoo = null;
            view.dragPastCoo = null;

            if (e.type === "mouseup") {
                if (view.mode === View.SELECT) {
                    view.selector.dispatch('mouseup', {coo: xymouse})
                }
            }

            if (e.type === "mouseout" || e.type === "touchend" || e.type === "touchcancel") {
                if (e.type === "mouseout" || e.type === "touchcancel") {
                    if (view.mode === View.SELECT) {
                        view.selector.dispatch('mouseout', {coo: xymouse, e})
                    }

                    return;
                }

                if (e.type === "touchend") {
                    if (view.mode === View.SELECT) {
                        view.selector.dispatch('mouseup', {coo: xymouse})

                        return;
                    }
                }
            }

            if (view.rightClick) {
                let ctxMenu = view.aladin.contextMenu;
                if (showContextMenu && ctxMenu && view.aladin.options.showContextMenu) {
                    ctxMenu.attach(
                        DefaultActionsForContextMenu.getDefaultActions(view.aladin),
                        null
                    );
                    ctxMenu._show({e});
                }

                view.rightClick = false;
                showContextMenu = true;
                return;
            }

            // popup to show ?
            if (!wasDragging || e.type === "touchend") {
                if (e.type === "touchend") {
                    if (e.targetTouches && e.targetTouches.length == 0) {
                        // Check if the user moved a lot or not
                        // maybe check the time between the last touch start
                        const elapsedTime = Date.now() - touchStartTime;
                        if (elapsedTime < 100) {
                            view.updateObjectsLookup();
                            if (view.selectionMode === View.SELECTION_MODE_SKEWER) {
                                handleSkewerSelect(e, withModifierKey)
                            } else {
                                handleSelect(xymouse, 15, withModifierKey);
                            }
                        }
                    }
                } else {
                    if (view.selectionMode === View.SELECTION_MODE_EDGE) {
                        handleSelect(xymouse, 5, withModifierKey);
                    } else {
                        handleSkewerSelect(e, withModifierKey);
                    }
                }
            }

            var onClickFunction = view.aladin.callbacksByEventName['click'];
            if (typeof onClickFunction === 'function') {
                var pos = view.aladin.pix2world(xymouse.x, xymouse.y, "icrs");
                if (pos !== undefined) {
                    onClickFunction({ ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y, isDragging: wasDragging });
                }
            }

            if (view.mode === View.SELECT && e.type === "click") {
                view.selector.dispatch('click', {coo: xymouse})
            }

            // TODO : remplacer par mecanisme de listeners
            // on avertit les catalogues progressifs
            view.refreshProgressiveCats();
            if (wasDragging) {
                view.wasm.releaseLeftButtonMouse();
            }
        });

        view.lastHoveredObject = null;
        var lastMouseMovePos = null;
        const pickColor = (xymouse) => {
            const layers = view.aladin.getStackLayers()
            let lastImageLayer = view.aladin.getOverlayImageLayer(layers[layers.length - 1])
            try {
                let probedValue = lastImageLayer.readPixel(xymouse.x, xymouse.y);
                view.colorPickerTool.domElement.style.display = "block"

                if (probedValue !== null && probedValue.length === 3) {
                    // rgb color
                    const r = probedValue[0];
                    const g = probedValue[1];
                    const b = probedValue[2];

                    view.colorPickerTool.probedValue = Color.rgbToHex(r, g, b);
                    view.colorPickerTool.domElement.innerText = view.colorPickerTool.probedValue
                } else if (probedValue !== null && probedValue.length === 4) {
                    // rgba color
                    const r = probedValue[0];
                    const g = probedValue[1];
                    const b = probedValue[2];
                    const a = probedValue[3];

                    view.colorPickerTool.probedValue = Color.rgbaToHex(r, g, b, a);
                    view.colorPickerTool.domElement.innerText = view.colorPickerTool.probedValue
                } else {
                    // 1-channel color
                    view.colorPickerTool.probedValue = probedValue;
                    view.colorPickerTool.domElement.innerText = probedValue
                }
            } catch(e) {
                console.warn("Pixel color reading: " + e)
                // out of the projection, we probe no pixel
                view.colorPickerTool.domElement.style.display = "none"
            }

            view.colorPickerTool.domElement.style.left = `${xymouse.x + view.aladin.aladinDiv.getBoundingClientRect().x}px`;
            view.colorPickerTool.domElement.style.top = `${xymouse.y + view.aladin.aladinDiv.getBoundingClientRect().y}px`;
        }

        Utils.on(view.catalogCanvas, "mousemove touchmove", function (e) {
            e.preventDefault();

            const xymouse = Utils.relMouseCoords(e);

            ALEvent.CANVAS_EVENT.dispatchedTo(view.aladinDiv, {
                state: {
                    mode: view.mode,
                    dragging: view.dragging,
                },
                type: e.type,
                xy: xymouse,
            });

            let dist;
            if (xystart) {
                dist = (xymouse.x - xystart.x)*(xymouse.x - xystart.x) + (xymouse.y - xystart.y)*(xymouse.y - xystart.y);
            }

            if (e.type === 'touchmove' && xystart) {
                if (longTouchTimer && dist > 100) {
                    if (view.aladin.statusBar) {
                        view.aladin.statusBar.removeMessage('opening-ctxmenu')
                    }
                    clearTimeout(longTouchTimer)
                    longTouchTimer = undefined;
                }
            }

            if (view.rightClick) {
                var onRightClickMoveFunction = view.aladin.callbacksByEventName['rightClickMove'];
                if (typeof onRightClickMoveFunction === 'function') {
                    onRightClickMoveFunction(xymouse.x, xymouse.y);

                    // do not process further
                    return;
                }

                if (dist < 100) {
                    return;
                }

                showContextMenu = false;

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

            if (e.type === 'touchmove' && view.pinchZoomParameters.isPinching && e.touches && e.touches.length >= 2) {
                // rotation
                var currentFingerAngle = Math.atan2(e.targetTouches[1].clientY - e.targetTouches[0].clientY, e.targetTouches[1].clientX - e.targetTouches[0].clientX) * 180.0 / Math.PI;
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
                    view.setRotation(rotation);
                }

                // zoom
                const dist = Math.sqrt(Math.pow(e.touches[0].clientX - e.touches[1].clientX, 2) + Math.pow(e.touches[0].clientY - e.touches[1].clientY, 2));
                const zoomFactor = view.pinchZoomParameters.initialZoomFactor * view.pinchZoomParameters.initialDistance / dist;
                view.zoomFactor = zoomFactor;

                return;
            }

            if (!view.dragging && !view.moving) {
                view.updateObjectsLookup();
            }

            if (!view.dragging && !view.moving && view.mode === View.PAN) {
                // call listener of 'mouseMove' event
                var onMouseMoveFunction = view.aladin.callbacksByEventName['mouseMove'];
                if (typeof onMouseMoveFunction === 'function') {
                    var pos = view.aladin.pix2world(xymouse.x, xymouse.y);
                    if (pos !== undefined) {
                        onMouseMoveFunction({ ra: pos[0], dec: pos[1], x: xymouse.x, y: xymouse.y, frame: view.cooFrame.label });
                    }
                    // send null ra and dec when we go out of the "sky"
                    else if (lastMouseMovePos != null) {
                        onMouseMoveFunction({ ra: null, dec: null, x: xymouse.x, y: xymouse.y, frame: view.cooFrame.label });
                    }
                    lastMouseMovePos = pos;
                }

                if (view.selectionMode === View.SELECTION_MODE_EDGE) {
                    // We're in edge selection mode for footprints.  closestObjects() will find those footprints by closeness to a footprint edge.
                    // closestObjects is very costly, we would like to not do it
                    // especially if the objectHovered function is not defined.
                    var closests = view.closestObjects(xymouse.x, xymouse.y, 5);

                    if (closests) {
                        hoverObjects(closests, xymouse);
                    } else {
                        unhoverObjects(view, xymouse);
                    }
                } else if (view.selectionMode === View.SELECTION_MODE_SKEWER) {
                    // We're in skewer mode.  Let's see what would be selected.
                    const skewerTargetsByLayer = Selector.getSkewerObjects(e, view);
                    const skewerObjects = skewerTargetsByLayer.flat();

                    if (skewerObjects.length > 0) {
                        hoverObjects(skewerObjects, xymouse);
                    } else {
                        unhoverObjects(view, xymouse);
                    }

                }


                if (e.type === "mousemove") {
                    return;
                }
            }

            if (view.mode === View.SELECT) {
                view.selector.dispatch('mousemove', {coo: xymouse})
            }

            if (view.mode === View.TOOL_COLOR_PICKER) {
                pickColor(xymouse);
            }

            if (!view.dragging) {
                return;
            }

            view.realDragging = true;

            if (view.mode === View.PAN) {
                view.pan = {
                    s1: view.dragCoo,
                    s2: xymouse
                };
            }

            view.dragCoo = xymouse;
            // update drag coo with the new position

            /*if (view.mode == View.SELECT) {
                view.requestRedraw();
                return;
            }*/
        }); //// endof mousemove ////

        // disable text selection on IE
        //Utils.on(view.aladinDiv, "selectstart", function () { return false; })

        view.prevWheelTime = undefined;

        function normalizeWheel(event) {
            // Safari/Chrome on macOS: deltaMode = 0 (pixels), but trackpad steps are tiny
            let scale = 1;
            if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
                scale = 16; // assume ~16px per line
            } else if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
                scale = window.innerHeight;
            }
            return event.deltaY * scale;
        }
        view.zoomDelta = 0;

        Utils.on(view.catalogCanvas, 'wheel', function (e) {
            e.preventDefault();
            e.stopPropagation();

            const xymouse = Utils.relMouseCoords(e);
            view.xy = xymouse
            ALEvent.CANVAS_EVENT.dispatchedTo(view.aladinDiv, {
                state: {
                    mode: view.mode,
                    dragging: view.dragging,
                },
                type: e.type,
                xy: xymouse,
            });

            var onWheelTriggeredFunction = view.aladin.callbacksByEventName['wheelTriggered'];
            if (typeof onWheelTriggeredFunction === 'function') {
                onWheelTriggeredFunction(e)
            } else {
                // Default Aladin Lite zooming
                const normalizedDelta = e.deltaY && normalizeWheel(e) || e.detail || (-e.wheelDelta);
                // Accumulate the normalized delta
                // We do not zoom because we cannot rely on "wheel" event
                // being triggered at constant time steps
                // The zoom is delayed to the redraw which is animation frame requested!
                view.zoomDelta += normalizedDelta;

                if (view.mode === View.TOOL_COLOR_PICKER) {
                    pickColor(xymouse);
                }
            }

            return false;
        });

        Utils.on(view.catalogCanvas, "mouseover", (_) => {
            view.mouseover = true;
        });

        Utils.on(view.catalogCanvas, "mouseout", (_) => {
            view.mouseover = false;
        });

        Utils.on(window, "keydown", function (e) {
            // check first if the user mouse over the aladin div
            if (!view.mouseover)
                return;

            switch (e.keyCode) {
                // escape
                case 27:
                    // Called when realfullscreen is false. Escaping from real fullscreen does not seem to trigger the keydown event
                    if (view.aladin.isInFullscreen) {
                        view.aladin.toggleFullscreen(view.aladin.options.realFullscreen);
                    }

                    break;
                default:
                    break;

            }
        });
    };

    var init = function (view) {
        var stats = new Stats();
        stats.domElement.style.top = '50px';

        var statsDiv = document.getElementById('aladin-statsDiv');

        if (statsDiv) {
            // Append stats.domElement to statsDiv
            statsDiv.appendChild(stats.domElement);
        }

        view.stats = stats;

        createListeners(view);

        view.displayHpxGrid = false;
        view.displayCatalog = false;
        view.skewerEnabled = false;
    };

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

    /**
     * redraw the whole view
     */
    View.prototype.redraw = function (now) {
        // Elapsed time since last loop
        const elapsedTime = now - this.prevTime;
        this.prevTime = now;

        if (Math.abs(this.zoomDelta) > 1e-3) {
            // Apply a fraction each frame (smoothing)
            let step = this.zoomDelta * 0.2;
            function wheelToZoomFactor(delta) {
                const sensitivity = 0.002; // tune this
                return Math.exp(-delta * sensitivity);
            }

            this.zoomFactor /= wheelToZoomFactor(step);
            this.zoomDelta -= step;
        }

        if (this.pan) {
            let s1 = this.pan.s1;
            let s2 = this.pan.s2;

            if (s1 && s2) {
                this.wasm.moveMouse(s1.x, s1.y, s2.x, s2.y);
                this.wasm.goFromTo(s1.x, s1.y, s2.x, s2.y);

                this.updateCenter();

                ALEvent.POSITION_CHANGED.dispatchedTo(this.aladin.aladinDiv, this.viewCenter);

                // Apply position changed callback after the move
                this.throttledPositionChanged(true);
            }

            this.pan = null;
        }

        this.moving = this.wasm.update(elapsedTime);

        // inertia run throttled position
        if (this.moving && this.aladin.callbacksByEventName && this.aladin.callbacksByEventName['positionChanged'] && this.wasm.isInerting()) {
            // run the trottled position
            this.throttledPositionChanged(false);
        }

        ////// 2. Draw catalogues////////
        const isViewRendering = this.wasm.isRendering();
        if (isViewRendering || this.needRedraw) {
            this.drawAllOverlays();
        }
        this.needRedraw = false;

        // request another frame
        requestAnimFrame(this.redrawClbk);
    };

    View.prototype.drawAllOverlays = function () {
        var ctx = this.catalogCtx;
        this.catalogCanvasCleared = false;

        if (this.mustClearCatalog) {
            ctx.clearRect(0, 0, this.width, this.height);
            this.catalogCanvasCleared = true;
            this.mustClearCatalog = false;
        }

        if (this.catalogs && this.catalogs.length > 0 && this.displayCatalog && (!this.dragging || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            // TODO : do not clear every time
            //// clear canvas ////
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
            }

            for (var i = 0; i < this.catalogs.length; i++) {
                var cat = this.catalogs[i];
                cat.draw(ctx, this.width, this.height);
            }
        }

        // draw popup catalog
        if (this.catalogForPopup.isShowing && this.catalogForPopup.sources.length > 0) {
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
            }

            this.catalogForPopup.draw(ctx, this.width, this.height);

            // draw popup overlay layer
            if (this.overlayForPopup.isShowing) {
                this.overlayForPopup.draw(ctx);
            }
        }

        ////// 3. Draw overlays////////
        if (this.overlays && this.overlays.length > 0 && (!this.dragging || View.DRAW_SOURCES_WHILE_DRAGGING)) {
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
            }

            for (var i = 0; i < this.overlays.length; i++) {
                this.overlays[i].draw(ctx);
            }
        }

        // Draw selected items (catalog sources, overlay, footprints, ...) afterwards
        if (this.selection) {
            this.selection.forEach((objList) => {
                objList.forEach((o) => {
                    if (o instanceof Source) {
                        o.draw(ctx, this.width, this.height)
                    } else {
                        // Circle, Ellipse, Footprints, ...
                        o.draw(ctx, this)
                    }
                })
            });
        }

        // Draw hovered items afterwards
        if (this.lastHoveredObject) {
            this.lastHoveredObject.forEach((o) => {
                if (o instanceof Source) {
                    o.draw(ctx, this.width, this.height)
                } else {
                    // Circle, Ellipse, Footprints, ...
                    o.draw(ctx, this)
                }
            })
        }

        // Redraw HEALPix grid
        if (this.displayHpxGrid) {
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
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
                this.healpixGrid.redraw(ctx, cornersXYViewMapHighres, this.fov, this.curNorder);
            }
            else {
                this.healpixGrid.redraw(ctx, cornersXYViewMapAllsky, this.fov, 3);
            }
        }

        // display grid labels
        if (this.gridCfg.enabled && this.gridCfg.showLabels) {
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
            }

            this.wasm.drawGridLabels();
        }

        if (this.mode === View.SELECT) {
            if (!this.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.width, this.height);
                this.catalogCanvasCleared = true;
            }

            this.selector.dispatch('draw')
        }
    };

    View.prototype.reverseLongitude = function(longitudeReversed) {
        this.wasm.setLongitudeReversed(longitudeReversed);
    }

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
        let [lon, lat] = this.aladin.pix2world(this.cx, this.cy, 'icrs');

        var radius = this.fov * 0.5 * this.ratio;
        this.wasm.queryDisc(norder, lon, lat, radius).forEach(x => pixList.push(Number(x)));

        return pixList;
    };

    View.prototype.readPixel = function(prober) {
        // Hide the coo grid
        this.aladin.hideCooGrid();

        // Ask for the redraw to make the coo grid hiding effective
        this.redraw()

        let c = this.getRawPixelsCanvas(null, null);
        let ctx = c.getContext("2d");

        let imageData;

        if (Utils.isNumber(prober.x) && Utils.isNumber(prober.y)) {
            imageData = ctx.getImageData(prober.x, prober.y, 1, 1);
        } else if (Utils.isNumber(prober.top) && Utils.isNumber(prober.left) && Utils.isNumber(prober.w) && Utils.isNumber(prober.h)) {
            imageData = ctx.getImageData(prober.top, prober.left, prober.w, prober.h);
        }

        // Show the coo grid back again
        this.aladin.showCooGrid();

        return imageData;
    };

    /**
     * Unselects all currently selected objects.
     */
    View.prototype.unselectObjects = function() {
        if (this.manualSelection) {
            return;
        }

        this.aladin.measurementTable.hide();

        if (this.selection) {
            this.selection.forEach((objList) => {
                objList.forEach((o) => o.deselect())
            });

            this.selection = null;
        }

        this.requestRedraw();
    }

    /**
     * Selects the specified objects in the view.
     *
     * If withModifierKey is true, it modifies the existing selection (adds/removes).
     * Otherwise, it replaces the current selection.
     *
     * @param {Array|Object} selection - The objects to select, either an array or a selector object.
     * @param {boolean} [withModifierKey=false] - Whether to modify (versus replace) the existing selections.
     */
    View.prototype.selectObjects = function(selection, withModifierKey=false) {
        if (this.manualSelection) {
            return;
        }

        if (Array.isArray(selection) && withModifierKey) {
            selection = this.computeModifiedSelection(selection)
        }

        // unselect the previous selection
        this.unselectObjects();

        if (Array.isArray(selection)) {
            this.selection = selection;
        } else {
            // select the new
            this.selection = Selector.getObjects(selection, this);
        }

        if (this.selection.length > 0) {
            this.selection.forEach((objListPerCatalog) => {
                objListPerCatalog.forEach((obj) => {
                    obj.select();

                    if (!obj.getCatalog) {
                        return;
                    }
                    // For objects belonging a catalog

                    let cat = obj.getCatalog();

                    // trigger the non action clicked if it does not show the table
                    // table show is handled below
                    if (obj.actionClicked) {
                        if (!cat || !cat.onClick || cat.onClick !== "showTable") {
                            obj.actionClicked()
                        }
                    }
                })
            });

            // show the objects from catalogs having the onClick = "showTable" field
            let tables = this.selection
                .filter(objList => {
                    if (!objList[0].getCatalog) {
                        return false;
                    }

                    let cat = objList[0].getCatalog();
                    return cat && cat.onClick && cat.onClick == 'showTable';
                })
                .map(objList => {
                    // Get the catalog containing that list of objects
                    let catalog = objList[0].getCatalog();

                    let source;
                    let sources = objList.map((o) => {
                        if (o instanceof Footprint) {
                            source = o.source;
                        } else {
                            source = o;
                        }

                        return source;
                    });

                    let tableColor = catalog.color;
                    if (catalog.colorFn) {
                        tableColor = "white"
                    }

                    let table = {
                        'name': catalog.name,
                        'color': tableColor,
                        'rows': sources,
                        'fields': catalog.fields,
                        'showCallback': ObsCore.SHOW_CALLBACKS(this.aladin)
                    };

                    return table;
                })

            this.aladin.measurementTable.showMeasurement(tables);
        }
    }

    View.prototype._getLayerForObj = function(obj) {
        let layer = null;
        if (obj.getCatalog) {
            layer = obj.getCatalog()
        } else {
            layer = obj.overlay
        }
        return layer
    }

    View.prototype._copySelectionsToStage = function(selections, stage, overlays, exclude) {
        for (const group of selections) {
            for (const obj of group) {
                const objExcluded = exclude.includes(obj)
                if (!objExcluded) {
                    const layer = this._getLayerForObj(obj)
                    const idx = overlays.findIndex(item => item.uuid === layer.uuid);
                    if (idx >= 0) {
                        stage[idx].push(obj)
                    } else {
                        console.warn("Layer not found for selected obj: " + obj)
                    }
                }
            }
        }
    }

    /**
     * Computes the full set of selections that should result if the specified (pending) objects were
     * selected with a modifier key pressed.
     *
     * If there are existing selections, it adds pending items that aren't selected,
     * or removes all pending items if they are all already selected.
     *
     * Organizes selections by overlay layers as expected by selectObjects.
     *
     * @param {Array<Array>} pending - Array of array of objects to potentially add/remove from selection.
     * @returns {Array<Array>} The modified selection array.
     */
    View.prototype.computeModifiedSelection = function(pending) {
        const current = this.selection
        let modSelection = pending
        if (current && current.length > 0) {
            // There are some items already selected.
            // We will be adding all the pending selections that are not already selected,
            // UNLESS all of the pending selections are already selected, in which case
            // they will all be unselected.
            const toAdd = []
            let mightRemove = []
            modSelection = []  // We will build a new selection list from the current and pending selections

            // stage will have one row for each existing overlay in which to collect all desired selections.
            const overlays = this.aladin.getOverlays()
            const stage = new Array(overlays.length).fill(null).map(() => []);

            // Put already-selected items in mightRemove and not-yet-selected items in toAdd.
            for (const group of pending) {
                for (const obj of group) {
                    if (obj.isSelected) {
                        mightRemove.push(obj)
                    } else {
                        toAdd.push(obj)
                    }
                }
            }

            // If there is anything in toAdd, then clear mightRemove since they will be left selected.
            if (toAdd.length > 0) {
                mightRemove = []
            }

            // Copy current selections to stage except for anything in mightRemove
            this._copySelectionsToStage(current, stage, overlays, mightRemove)

            // Copy toAdd selections to stage
            this._copySelectionsToStage([toAdd], stage, overlays, [])

            // Build new modified selections list from stage.
            // I can preserve the layer order, but I don't know how to preserve the order within
            // layers without looping through all objects.  Hopefully that order doesn't matter.
            for (let i=0; i<stage.length; i++) {
                if (stage[i].length > 0) {
                    // We have selected objects in this layer so will add the layer (or list of overlays) to modSelection

                    if (overlays[i].type === 'catalog') {
                        // The layer is a catalog so we add one entry for all its selections
                        const catLayer = []
                        modSelection.push(catLayer)
                        for (const obj of stage[i]) {
                            catLayer.push(obj)
                        }

                    } else {
                        // Assume it's a graphicalOverlay and add separate entries for each selected obj
                        // (That is the way objects in graphicalOverlays are currently selected.  If they start
                        // being selected all in one list per overlay, then that can change here.)
                        for (const obj of stage[i]) {
                            modSelection.push([obj])
                        }
                    }
                }
            }

        }
        return modSelection;
    }

    View.prototype.getVisibleCells = function (norder) {
        return this.wasm.getVisibleCells(norder);
    };

    // Called for touchmove events
    View.prototype.setZoomFactor = function(zoomFactor) {
        this.wasm.setZoomFactor(zoomFactor);
        this.updateZoomState();
    }

    View.prototype.setFoV = function(fov) {
        this.wasm.setFieldOfView(fov)
        this.updateZoomState();
    }

    View.prototype.increaseZoom = function () {
        this.zoom.apply({
            stop: this.zoomFactor / 1.4,
            duration: 200,
        });
    }

    View.prototype.decreaseZoom = function () {
        this.zoom.apply({
            stop: this.zoomFactor * 1.4,
            duration: 200,
        });
    }

    View.prototype.setRotation = function(rotation) {
        if (Math.abs(rotation - this.aladin.getRotation()) < 1e-5) {
            return;
        }

        this.wasm.setRotation(rotation);
        var rotationChangedCallback = this.aladin.callbacksByEventName["rotationChanged"];
        typeof rotationChangedCallback === "function" && rotationChangedCallback(rotation);
    }

    View.prototype.setGridOptions = function (options) {
        if (options.color) {
            // 1. the user has maybe given some
            options.color = new Color(options.color);
            // 3. convert from 0-255 to 0-1
            options.color.r /= 255;
            options.color.g /= 255;
            options.color.b /= 255;
        }

        this.gridCfg = {...this.gridCfg, ...options};
        this.wasm.setGridOptions(this.gridCfg);

        this.mustClearCatalog = true;

        ALEvent.COO_GRID_UPDATED.dispatchedTo(this.aladinDiv, this.gridCfg);

        this.requestRedraw();
    };

    View.prototype.getGridOptions = function() {
        return this.gridCfg;
    };

    View.prototype.updateZoomState = function () {
        this.computeNorder();

        let fovX = this.fov;
        let fovY = this.wasm.getFieldOfView()[1];
        fovX = Math.min(fovX, 360);
        fovY = Math.min(fovY, 180);

        this.fovY = fovY;

        this.debounceProgCatOnZoom();

        ALEvent.ZOOM_CHANGED.dispatchedTo(this.aladinDiv, { fovX, fovY });

        this.throttledZoomChanged();
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
        // set the view to the image layer object
        // do the properties query if needed
        imageLayer.layer = layer;
        imageLayer._setView(this);

        // register its promise
        this.imageLayersBeingQueried.set(layer, imageLayer);

        this.addImageLayer(imageLayer, layer);

        return imageLayer;
    };

    // Insert a layer object (Image/HiPS) at a specific index in the stack
    View.prototype._addLayer = function(imageLayer) {
        // Keep the JS frontend in-line with the wasm state
        const layer = imageLayer.layer;
        imageLayer.added = true;

        const idxOverlayLayer = this.overlayLayers.findIndex(overlayLayer => overlayLayer == layer);
        if (idxOverlayLayer === -1) {
            this.overlayLayers.push(layer);
        } else {
            // layer already present
            // it exists
            var alreadyPresentImageLayer = this.imageLayers.get(layer);

            if (alreadyPresentImageLayer) {
                //let idx = this.removeImageLayer(layer)
                //if (idx >= 0) {
                //    this.overlayLayers.splice(idx, 0, layer);
                //}
            }

            // Notify that this image layer has been replaced by the wasm part
            //this.imageLayers.delete(layer);
        }

        this.imageLayers.set(layer, imageLayer);

        // select the layer if he is on top
        this.selectLayer(layer);

        ALEvent.LAYER_ADDED.dispatchedTo(this.aladinDiv, { layer: imageLayer });
    }

    View.prototype._waitsForLayer = function() {
        return this.promises.length !== 0;
    }

    View.prototype.getFirstLayer = function() {
        let layer = this.overlayLayers && this.overlayLayers[0];

        if (!layer) {
            // the overlay has not yet been added and is currently queried.
            layer = this.imageLayersBeingQueried.keys().next().value;
        }

        return layer;
    }

    View.prototype.addImageLayer = function (imageLayer, layer) {
        let self = this;
        // start the query
        const promise = imageLayer.query;
        this.promises.push(promise);

        // All image layer promises must be completed (fullfilled or rejected)
        const task = {
            message: imageLayer.name + ' loading...',
            id: Utils.uuidv4(),
        }
        // Ensure all the properties for HiPSes have been seeked
        ALEvent.FETCH.dispatchedTo(document, {task});
        // All the remaining promises must be terminated and the current one must be resolved
        // so that we can add it to the view (call of _add2View)
        Promise.all([Promise.allSettled(this.promises), promise])
            // Then we add the layer to the view
            .then((_) => imageLayer._addToView(layer))
            // Then we keep a track of the layer in the JS front
            .then((imageLayer) => {
                this._addLayer(imageLayer);

                // If the image layer has successfuly been added
                this.empty = false;

                // Change the view frame in case we have a planetary hips loaded
                if (imageLayer.hipsBody) {
                    if (this.options.showFrame) {
                        this.aladin.setFrame('J2000d');
                    }
                }
            })
            .catch((e) => {
                // remove it from the cache
                this.aladin.hipsCache.delete(imageLayer.id)

                if (imageLayer.errorCallback) {
                    imageLayer.errorCallback(e);
                }

                this.removeImageLayer(imageLayer);

                throw e;
            })
            .finally(() => {
                // Loading state is over
                ALEvent.RESOURCE_FETCHED.dispatchedTo(document, {task});

                self.imageLayersBeingQueried.delete(layer);
                // Remove the settled promise
                this.promises = this.promises.filter(p => p !== promise);

                const waitsForLayer = this._waitsForLayer();

                if (!waitsForLayer && this.delayedBaseLayerCalledParams && this.delayedBaseLayerCalledParams !== layer) {
                    this.aladin.setBaseImageLayer(this.delayedBaseLayerCalledParams)
                    this.delayedBaseLayerCalledParams = null;

                    return;
                }

                if (!waitsForLayer && this.empty) {
                    // no promises to launch and the view has no HiPS.
                    // This situation can occurs if the MOCServer is out
                    // If so we can directly put the url of the DSS hosted in alasky,
                    // it the best I can do if the MOCServer is out
                    self.aladin.setBaseImageLayer(Aladin.DEFAULT_OPTIONS.survey);
                }
            })
    }

    View.prototype.swapLayers = function(layer1, layer2) {
        // Throw an exception if either the first or the second layers are not in the stack
        this.wasm.swapLayers(layer1, layer2);

        // Swap in overlaylayers
        const i = this.overlayLayers.indexOf(layer1);
        const j = this.overlayLayers.indexOf(layer2);

        const tmp = this.overlayLayers[i];
        this.overlayLayers[i] = this.overlayLayers[j];
        this.overlayLayers[j] = tmp;

        // Tell the layer hierarchy has changed
        ALEvent.LAYER_SWAPPED.dispatchedTo(
            this.aladinDiv,
            {
                layer1: this.imageLayers.get(layer1),
                layer2: this.imageLayers.get(layer2)
            }
        );
    }

    View.prototype.removeImageLayer = function (layer) {
        // Get the survey to remove to dissociate it from the view
        let imageLayer = this.imageLayers.get(layer);

        if (imageLayer === undefined) {
            // there is nothing to remove
            return -1;
        }

        // Update the backend
        imageLayer._removeFromView();

        // Get the survey to remove to dissociate it from the view
        imageLayer.added = false;

        const idxOverlaidLayer = this.overlayLayers.findIndex(overlaidLayer => overlaidLayer == layer);
        // Delete it
        this.imageLayers.delete(layer);

        // Remove it from the layer stack
        this.overlayLayers.splice(idxOverlaidLayer, 1);

        if (this.overlayLayers.length > 0 && this.selectedLayer === layer) {
            // If the layer removed was selected then we select the last layer
            this.selectLayer(this.overlayLayers[this.overlayLayers.length - 1]);
        }

        ALEvent.LAYER_REMOVED.dispatchedTo(this.aladinDiv, { layer: imageLayer });

        return idxOverlaidLayer;
    };

    View.prototype.contains = function(survey) {
        if (survey instanceof HiPS || survey instanceof Image) {
            if (survey.added === true) {
                return true;
            }

            // maybe it has not been added yet
            let found = Array.from(this.imageLayersBeingQueried
                .values())
                .some((s) => {
                    return s === survey;
                });

            return found;
        }

        // Case where survey is a string
        if(Array.from(this.imageLayers
            .values())
            .some((s) => {
                return s.id === survey;
            })) {
                return true;
            }

        if(Array.from(this.imageLayersBeingQueried
            .values())
            .some((s) => {
                return s.id === survey;
            })) {
                return true;
            }

        return false;
    }

    View.prototype.setHiPSUrl = function (pastUrl, newUrl) {
        try {
            this.wasm.setHiPSUrl(pastUrl, newUrl);
        } catch(e) {
            console.error(e)
        }
    }

    View.prototype.getImageLayer = function (layer) {
        layer = layer || (this.overlayLayers && this.overlayLayers[0]);
        let imageLayerQueried = this.imageLayersBeingQueried.get(layer);
        let imageLayer = this.imageLayers.get(layer);

        let obj = imageLayer || imageLayerQueried;

        return obj;
    };

    View.prototype.requestRedraw = function () {
        this.needRedraw = true;
    };

    View.prototype.setProjection = function (projName) {
        if (!ProjectionEnum[projName]) {
            console.warn(projName + " is not a valid projection.")
            projName = 'SIN'
        }

        if (this.projection && this.projection.id === ProjectionEnum[projName].id) {
            return;
        }

        this.projection = ProjectionEnum[projName];

        // Change the projection here
        this.wasm.setProjection(projName);
        this.updateZoomState()

        const projFn = this.aladin.callbacksByEventName['projectionChanged'];
        (typeof projFn === 'function') && projFn(projName);

        this.requestRedraw();
    };

    View.prototype.changeFrame = function (cooFrame) {
        this.cooFrame = cooFrame;

        // Set the new frame to the backend
        this.wasm.setCooSystem(this.cooFrame.system);

        // Set the grid label format
        if (this.cooFrame.label == "ICRS") {
            this.setGridOptions({fmt: "sexagesimal"});
        }
        else {
            this.setGridOptions({fmt: "decimal"});
        }

        // Get the new view center position (given in icrs)
        this.updateCenter();

        ALEvent.FRAME_CHANGED.dispatchedTo(this.aladinDiv, {cooFrame: this.cooFrame});

        this.requestRedraw();
    };

    View.prototype.updateCenter = function() {
        // Center position in the frame of the view
        const [lon, lat] = this.wasm.getCenter();

        // ICRS conversion
        let [ra, dec] = this.wasm.viewToICRSCooSys(lon, lat);

        if (ra < 0) {
            ra = ra + 360.0
        }

        this.viewCenter = {ra, dec};
    }

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

    /**
     *
     * @API Point to a specific location
     *
     * @param ra ra expressed in ICRS or ICRS frame
     * @param dec dec expressed in ICRS or ICRS frame
     * @param options
     *
     */
    View.prototype.pointTo = function (ra, dec) {
        ra = parseFloat(ra);
        dec = parseFloat(dec);

        if (!Number.isFinite(ra) || !Number.isFinite(dec)) {
            return;
        }

        this.viewCenter = {ra, dec};

        // Put a javascript code here to do some animation
        this.wasm.setCenter(this.viewCenter.ra, this.viewCenter.dec);

        ALEvent.POSITION_CHANGED.dispatchedTo(this.aladin.aladinDiv, this.viewCenter);

        var self = this;
        setTimeout(function () { self.refreshProgressiveCats(); }, 1000);
        // Apply position changed callback after the move
        self.throttledPositionChanged(false);
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

    View.prototype.removeOverlays = function () {
        this.catalogs = [];
        this.overlays = [];
        this.mocs = [];

        this.allOverlayLayers.forEach((overlay) => {
            ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.dispatchedTo(this.aladinDiv, { overlay });
        })
        this.allOverlayLayers = [];

        this.mustClearCatalog = true;
        this.requestRedraw();
    };

    View.prototype.removeOverlay = function (overlay) {
        let indexToDelete = this.allOverlayLayers.indexOf(overlay);
        if (indexToDelete === -1) {
            // overlay not found
            return;
        }

        this.allOverlayLayers.splice(indexToDelete, 1);

        if (overlay.type == 'catalog' || overlay.type == 'progressivecat') {
            indexToDelete = this.catalogs.indexOf(overlay);

            this.catalogs.splice(indexToDelete, 1);

            this.unselectObjects();
        }
        else if (overlay.type == 'moc') {
            indexToDelete = this.mocs.indexOf(overlay);

            let moc = this.mocs.splice(indexToDelete, 1);
            // remove from aladin lite backend
            moc[0].delete();
        }
        else if (overlay.type == 'overlay') {
            indexToDelete = this.overlays.indexOf(overlay);
            this.overlays.splice(indexToDelete, 1);
        }

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.dispatchedTo(this.aladinDiv, { overlay });

        this.mustClearCatalog = true;
        this.requestRedraw();
    };

    View.prototype.removeOverlayByName = function (overlayName) {
        let overlay = this.allOverlayLayers.find(o => o.name === overlayName);
        if (!overlay) {
            console.error(`Overlay "${overlayName}" not found.`);
            return;
        }
        this.removeOverlay(overlay);
    };

    View.prototype.add = function(overlay) {
        overlay.name = this.makeUniqLayerName(overlay.name);

        let idx = this.allOverlayLayers.length;
        overlay.setView(this, idx);
    };

    View.prototype.addMOC = View.prototype.add;
    View.prototype.addOverlay = View.prototype.add;
    View.prototype.addCatalog = View.prototype.add;

    View.prototype.insertOverlay = function(overlay, idx) {
        this.allOverlayLayers.splice(idx, 0, overlay);
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

                    if (s.isFootprint() && cat.onlyFootprints && !s.tooSmallFootprint) {
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

        let closests = [];

        footprints.forEach((footprint) => {
            const originLineWidth = footprint.getLineWidth();
            let drawingLineWidth = originLineWidth;
            if (footprint.isSelected && footprint.getSelectionLineWidth()) {
                drawingLineWidth = footprint.getSelectionLineWidth();
            }
            let spreadedLineWidth = (drawingLineWidth || 1) + 3;

            footprint.setLineWidth(spreadedLineWidth);
            if (footprint.isShowing && footprint.isInStroke(ctx, this, x * window.devicePixelRatio, y * window.devicePixelRatio)) {
                closests.push(footprint);
            }
            footprint.setLineWidth(originLineWidth);
        })

        return closests;
    };

    // return closest object within a radius of maxRadius pixels. maxRadius is an integer
    View.prototype.closestObjects = function (x, y, maxRadius) {
        // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
        var overlay;
        var canvas = this.catalogCanvas;
        var ctx = canvas.getContext("2d");
        // this makes footprint selection easier as the catch-zone is larger

        let closests = [];
        if (this.overlays) {
            for (var k = 0; k < this.overlays.length; k++) {
                overlay = this.overlays[k];
                closests = closests.concat(this.closestFootprints(overlay.overlayItems, ctx, x, y));
            }
        }

        // Catalogs can also have footprints
        if (this.catalogs) {
            for (var k = 0; k < this.catalogs.length; k++) {
                let catalog = this.catalogs[k];

                for (var s of catalog.getSources()) {
                    if (s.isFootprint() && !s.tooSmallFootprint) {
                        let footprint = s.footprint;
                        const originLineWidth = footprint.getLineWidth();
                        let drawingLineWidth = originLineWidth;
                        if (footprint.isSelected && footprint.getSelectionLineWidth()) {
                            drawingLineWidth = footprint.getSelectionLineWidth();
                        }
                        let spreadedLineWidth = (drawingLineWidth || 1) + 3;

                        footprint.setLineWidth(spreadedLineWidth);
                        if (footprint.isShowing && footprint.isInStroke(ctx, this, x * window.devicePixelRatio, y * window.devicePixelRatio)) {
                            closests.push(s);
                        }
                        footprint.setLineWidth(originLineWidth);
                    }
                }
            }
        }

        if (!this.objLookup) {
            return null;
        }

        var dist = Number.POSITIVE_INFINITY;
        var closest = null;

        for (var dx = -maxRadius; dx <= maxRadius; dx++) {
            if (!this.objLookup[x + dx]) {
                continue;
            }
            for (var dy = -maxRadius; dy <= maxRadius; dy++) {
                if (this.objLookup[x + dx][y + dy]) {
                    var d = dx * dx + dy * dy;
                    if (d < dist) {
                        dist = d;
                        closest = this.objLookup[x + dx][y + dy]
                    } else if (d == dist) {
                        closest.concat(this.objLookup[x + dx][y + dy])
                    }
                }
            }
        }

        if (closest && closest.length > 0) {
            closests = closests.concat(closest)
        }

        if (closests.length === 0)
            return null;

        return closests;
    };

    return View;
})();


