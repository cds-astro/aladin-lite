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
 * Author: Thomas Boch[CDS], Matthieu Baumann[CDS]
 *
 *****************************************************************************/

import { version } from "./../../package.json";
import { View } from "./View.js";
import { Utils } from "./Utils";
import { GraphicOverlay } from "./Overlay.js";
import { Logger } from "./Logger.js";
import { ProgressiveCat } from "./ProgressiveCat.js";
import { Sesame } from "./Sesame.js";
import { PlanetaryFeaturesNameResolver } from "./PlanetaryFeaturesNameResolver.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { MeasurementTable } from "./MeasurementTable.js";
import { HiPS } from "./HiPS.js";
import { Coo } from "./libs/astro/coo.js";
import { CooConversion } from "./CooConversion.js";
import { HiPSCache } from "./HiPSCache.js";
import { HiPSList } from "./DefaultHiPSList.js";

import { ProjectionEnum } from "./ProjectionEnum.js";

import { ALEvent } from "./events/ALEvent.js";
import { Color } from "./Color.js";
import { Image } from "./Image.js";
import { DefaultActionsForContextMenu } from "./DefaultActionsForContextMenu.js";
import { SAMPConnector } from "./vo/samp.js";
import { Reticle } from "./Reticle.js";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";

// GUI
import { AladinLogo } from "./gui/AladinLogo.js";
import { Location } from "./gui/Location.js";
import { FoV } from "./gui/FoV.js";
import { ShareActionButton } from "./gui/Button/ShareView.js";
import { ContextMenu } from "./gui/Widgets/ContextMenu.js";
import { Popup } from "./Popup.js";
import A from "./A.js";
import { StatusBarBox } from "./gui/Box/StatusBarBox.js";
import { FullScreenActionButton } from "./gui/Button/FullScreen.js";
import { ProjectionActionButton } from "./gui/Button/Projection.js";

// features
import { SettingsButton } from "./gui/Button/Settings";
import { SimbadPointer } from "./gui/Button/SimbadPointer";
import { OverlayStackButton } from "./gui/Button/OverlayStack";
import { GridEnabler } from "./gui/Button/GridEnabler";
import { CooFrame } from "./gui/Input/CooFrame";
import { Circle } from "./shapes/Circle";
import { Ellipse } from "./shapes/Ellipse";
import { Polyline } from "./shapes/Polyline";

/**
 * @typedef {Object} AladinOptions
 * @description Options for configuring the Aladin Lite instance.
 *
 * @property {string} [survey="P/DSS2/color"] URL or ID of the survey to use
 * @property {string[]} [surveyUrl]
 *   Array of URLs for the survey images. This replaces the survey parameter.
 * @property {Object[]|string[]} [hipsList] A list of predefined HiPS for the Aladin instance.
 *   This option is used for searching for a HiPS in a list of surveys
 *   This list can have string item (either a CDS ID or an HiPS url) or an object that describes the HiPS
 *   more exhaustively. See the example below to see the different form that this item can have to describe a HiPS.
 * @property {string} [target="0 +0"] - Target coordinates for the initial view.
 * @property {CooFrame} [cooFrame="J2000"] - Coordinate frame.
 * @property {number} [fov=60] - Field of view in degrees.
 * @property {number} [northPoleOrientation=0] - North pole orientation in degrees. By default it is set to 0 deg i.e. the north pole will be found vertically north to the view.
 *  Positive orientation goes towards east i.e. in counter clockwise order as the east lies in the left direction of the view.
 * @property {string} [backgroundColor="rgb(60, 60, 60)"] - Background color in RGB format.
 *
 * @property {boolean} [showZoomControl=true] - Whether to show the zoom control toolbar.
 * This element belongs to the FoV UI thus its CSS class is `aladin-fov` 
 * @property {boolean} [showLayersControl=true] - Whether to show the layers control toolbar.
 * CSS class for that button is `aladin-stack-control` 
 * @property {boolean} [expandLayersControl=false] - Whether to show the stack box opened at starting
 * CSS class for the stack box is `aladin-stack-box`
 * @property {boolean} [showFullscreenControl=true] - Whether to show the fullscreen control toolbar.
 * CSS class for that button is `aladin-fullScreen-control` 
 * @property {boolean} [showSimbadPointerControl=false] - Whether to show the Simbad pointer control toolbar.
 * CSS class for that button is `aladin-simbadPointer-control` 
 * @property {boolean} [showCooGridControl=false] - Whether to show the coordinate grid control toolbar.
 * CSS class for that button is `aladin-grid-control` 
 * @property {boolean} [showSettingsControl=false] - Whether to show the settings control toolbar.
 * CSS class for that button is `aladin-settings-control` 
 * @property {boolean} [showShareControl=false] - Whether to show the share control toolbar.
 * CSS class for that button is `aladin-share-control` 
 * @property {boolean} [showStatusBar=true] - Whether to show the status bar. Enabled by default.
 * CSS class for that button is `aladin-status-bar` 
 * @property {boolean} [showFrame=true] - Whether to show the viewport frame.
 * CSS class for that button is `aladin-cooFrame` 
 * @property {boolean} [showFov=true] - Whether to show the field of view indicator.
 * CSS class for that button is `aladin-fov` 
 * @property {boolean} [showCooLocation=true] - Whether to show the coordinate location indicator.
 * CSS class for that button is `aladin-location` 
 * @property {boolean} [showProjectionControl=true] - Whether to show the projection control toolbar.
 * CSS class for that button is `aladin-projection-control` 
 * @property {boolean} [showContextMenu=false] - Whether to show the context menu.
 * @property {boolean} [showReticle=true] - Whether to show the reticle.
 * @property {boolean} [showCatalog=true] - Whether to show the catalog.
 * @property {boolean} [showCooGrid=true] - Whether the coordinates grid should be shown at startup.
 *
 * @property {boolean} [fullScreen=false] - Whether to start in full-screen mode.
 * @property {string} [reticleColor="rgb(178, 50, 178)"] - Color of the reticle in RGB format.
 * @property {number} [reticleSize=22] - Size of the reticle.
 * 
 * @property {string} [gridColor="rgb(178, 50, 178)"] - Color of the grid in RGB format. 
 *                                                      Is overshadowed by gridOptions.color if defined.
 * @property {number} [gridOpacity=0.8] - Opacity of the grid (0 to 1). 
 *                                        Is overshadowed by gridOptions.opacity if defined.
 * @property {Object} [gridOptions] - More options for the grid.
 * @property {string} [gridOptions.color="rgb(178, 50, 178)"] - Color of the grid. Can be specified as a named color 
 *                    (see {@link https://developer.mozilla.org/en-US/docs/Web/CSS/named-color| named colors}),
 *                    as rgb (ex: "rgb(178, 50, 178)"), or as a hex color (ex: "#86D6AE").              
 * @property {number} [gridOptions.thickness=2] - The thickness of the grid, in pixels.
 * @property {number} [gridOptions.opacity=0.8] - Opacity of the grid and labels. It is comprised between 0 and 1.
 * @property {boolean} [gridOptions.showLabels=true] - Whether the grid has labels.
 * @property {number} [gridOptions.labelSize=15] - The font size of the labels.
 * 
 * @property {string} [projection="SIN"] - Projection type. Can be 'SIN' for orthographic, 'MOL' for mollweide, 'AIT' for hammer-aitoff, 'ZEA' for zenital equal-area or 'MER' for mercator
 * @property {boolean} [log=true] - Whether to log events.
 * @property {boolean} [samp=false] - Whether to enable SAMP (Simple Application Messaging Protocol).
 * @property {boolean} [realFullscreen=false] - Whether to use real fullscreen mode.
 * @property {boolean} [pixelateCanvas=true] - Whether to pixelate the canvas.
 * @property {boolean} [manualSelection=false] - When set to true, no selection will be performed, only events will be generated.
 * @property {Object} [selector] - More options for the the selector.
 * @property {string} [selector.color] - Color of the selector, defaults to the color of the reticle. Can be a hex color or a function returning a hex color.
 * @property {number} [selector.lineWidth=2] - Width of the selector line.
 * 
 * @example
 * let aladin = A.aladin({
    target: 'galactic center',
    fov: 10,
    hipsList: [
        // url
        "https://alaskybis.unistra.fr/DSS/DSSColor",
        // ID from HiPS list
        "CDS/P/2MASS/color",
        // Not full HiPS described
        {
            name: 'DESI Legacy Surveys color (g, r, i, z)',
            id: 'CDS/P/DESI-Legacy-Surveys/DR10/color',
        },
        // HiPS with options. Fields accepted are those described in {@link A.hiPSOptions}.
        {
            name: "SDSS9 band-g",
            id: "P/SDSS9/g",
            creatorDid: "ivo://CDS/P/SDSS9/g",
            maxOrder: 10,
            tileSize: 512,
            numBitsPerPixel: 16,
            imgFormat: 'fits',
            cooFrame: 'equatorial',
            minCut: 0,
            maxCut: 1.8,
            stretch: 'linear',
            colormap: "redtemperature",
        }
    ]
})*/

/**
 * @typedef {Object} CircleSelection
 * @description Options for configuring the Aladin Lite instance.
 *
 * @property {number} x - x coordinate of the center's circle in pixels
 * @property {number} y - y coordinate of the center's circle in pixels
 * @property {number} r - radius of the circle in pixels
 * @property {function} contains - function taking a {x, y} object telling if the vertex is contained or not
 * @property {function} bbox - returns the bbox of the selection in pixels
 */

/**
 * @typedef {Object} RectSelection
 * @description Options for configuring the Aladin Lite instance.
 *
 * @property {number} x - top left x coordinate of the rectangle in pixels
 * @property {number} y - top left y coordinate of the rectangle in pixels
 * @property {number} w - width of the selection in pixels
 * @property {number} h - height of the selection in pixels
 * @property {function} contains - function taking a {x, y} object telling if the vertex is contained in the selection or not
 * @property {function} bbox - returns the bbox of the selection in pixels
 */

/**
 * @typedef {Object} PolygonSelection
 * @description Options for configuring the Aladin Lite instance.
 *
 * @property {Object[]} vertices - vertices of the polygon selection in pixels. Each vertex has a x and y key in pixels.
 * @property {function} contains - function taking a {x, y} object telling if the vertex is contained in the selection or not
 * @property {function} bbox - returns the bbox of the selection in pixels
 */

/**
 * @typedef {string} CooFrame
 * String with possible values: 'equatorial', 'ICRS', 'ICRSd', 'j2000', 'gal, 'galactic'
 */

/**
 * @typedef {string} ListenerCallback
 * String with possible values:
 *      'select' (deprecated, use objectsSelected instead),
 *      'objectsSelected',
        'objectClicked',
        'objectHovered',
        'objectHoveredStop',

        'footprintClicked',
        'footprintHovered',

        'positionChanged',
        'zoomChanged',

        'click',
        'rightClickMove',
        'mouseMove',

        'fullScreenToggled',
        'cooFrameChanged',
        'resizeChanged',
        'projectionChanged',
        'layerChanged'
 */

export let Aladin = (function () {
    /**
     * Creates an instance of the Aladin interactive sky atlas.
     *
     * @class
     * @constructs Aladin
     * @param {string|HTMLElement} aladinDiv - The ID of the HTML element or the HTML element itself
     *                                         where the Aladin sky atlas will be rendered.
     * @param {AladinOptions} requestedOptions - Options to customize the behavior and appearance of the Aladin atlas.
     * @throws {Error} Throws an error if aladinDiv is not provided or is invalid.
     *
     * @example
     * // Usage example:
     * import { A } from 'aladin-lite';
     *
     * let aladin = A.Aladin('#aladin-lite-div', { option1: 'value1', option2: 'value2' });
     */
    var Aladin = function (aladinDiv, requestedOptions) {
        this.callbacksByEventName = {}; // we store the callback functions (on 'zoomChanged', 'positionChanged', ...) here
        this.hipsCache = new HiPSCache();

        // check that aladinDiv exists, stop immediately otherwise
        if (!aladinDiv) {
            console.error(
                "Aladin div has not been found. Please check its name!"
            );
            return;
        }
        this.wasm = null;
        this.aladinDiv = aladinDiv;

        const self = this;

        ALEvent.HIPS_LAYER_ADDED.listenedBy(aladinDiv, (imageLayer) => {
            this.callbacksByEventName["layerChanged"] &&
            this.callbacksByEventName["layerChanged"](imageLayer.detail.layer, imageLayer.detail.layer.layer, "ADDED");
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(aladinDiv, (imageLayer) => {
            this.callbacksByEventName["layerChanged"] &&
            this.callbacksByEventName["layerChanged"](imageLayer.detail.layer, imageLayer.detail.layer.layer, "REMOVED");
        });

        // if not options was set, try to retrieve them from the query string
        if (requestedOptions === undefined) {
            requestedOptions = this.getOptionsFromQueryString();
        }
        requestedOptions = requestedOptions || {};

        // 'fov' option was previsouly called 'zoom'
        if ("zoom" in requestedOptions) {
            var fovValue = requestedOptions.zoom;
            delete requestedOptions.zoom;
            requestedOptions.fov = fovValue;
        }

        // merge with default options
        var options = {};

        for (var key in Aladin.DEFAULT_OPTIONS) {
            if (requestedOptions[key] !== undefined) {
                options[key] = requestedOptions[key];
            } else {
                options[key] = Aladin.DEFAULT_OPTIONS[key];
            }
        }

        // 'gridOptions' is an object, so it need it own loop
        if ("gridOptions" in requestedOptions) {
            for (var key in Aladin.DEFAULT_OPTIONS.gridOptions) {
                if (requestedOptions.gridOptions[key] === undefined) {
                    options.gridOptions[key] =
                        Aladin.DEFAULT_OPTIONS.gridOptions[key];
                }
            }
        }

        for (var key in requestedOptions) {
            if (Aladin.DEFAULT_OPTIONS[key] === undefined) {
                options[key] = requestedOptions[key];
            }
        }

        this.options = options;

        this.reduceDeformations = true;

        // Init the measurement table
        this.measurementTable = new MeasurementTable(this);

        // parent div
        aladinDiv.classList.add("aladin-container");
        // set different options
        // Reticle
        this.view = new View(this);

        // Aladin logo
        new AladinLogo(this.aladinDiv);

        this.reticle = new Reticle(this.options, this);
        this.popup = new Popup(this.aladinDiv, this.view);

        this.ui = [];

        // Background color
        if (options.backgroundColor) {
            this.backgroundColor = options.backgroundColor;
            this.setBackgroundColor(this.backgroundColor);
        }

        // Grid
        let gridOptions = options.gridOptions;

        // color and opacity can be defined by two variables. The item in gridOptions
        // should take precedence.
        gridOptions["color"] = options.gridOptions.color || options.gridColor;
        gridOptions["opacity"] =
            options.gridOptions.opacity || options.gridOpacity;
        if (options && options.showCooGrid) {
            gridOptions.enabled = true;
        }

        this.setCooGrid(gridOptions);

        this.gotoObject(options.target, undefined);

        if (options.log) {
            var params = options;
            params["version"] = Aladin.VERSION;
            Logger.log("startup", params);
        }

        if (options.catalogUrls) {
            for (var k = 0, len = options.catalogUrls.length; k < len; k++) {
                this.createCatalogFromVOTable(options.catalogUrls[k]);
            }
        }

        let hipsList = [].concat(options.hipsList);

        const fillHiPSCache = () => {
            for (var survey of hipsList) {
                let id, url, name;
                let cachedSurvey = {};

                if (typeof survey === "string") {
                    try {
                        url = new URL(survey).href;
                    } catch (e) {
                        id = survey;
                    }

                    name = url || id;
                } else if (survey instanceof Object) {
                    if (survey.id) {
                        id = survey.id;
                    }
                    if (survey.url) {
                        url = survey.url;
                    }

                    name = survey.name || survey.id || survey.url;

                    cachedSurvey = { ...cachedSurvey, ...survey };
                } else {
                    console.warn(
                        "unable to parse the survey list item: ",
                        survey
                    );
                    continue;
                }

                if (id) {
                    cachedSurvey["id"] = id;
                }
                if (url) {
                    cachedSurvey["url"] = url;
                }
                if (name) {
                    cachedSurvey["name"] = name;
                }

                // at least id or url is defined
                let key = id || url;

                // Merge what is already in the cache for that HiPS with new properties
                // coming from the MOCServer
                let hips = new HiPS(key, key, cachedSurvey)
                self.hipsCache.append(key, hips);
            }
        };
        this._setupUI(options);

        fillHiPSCache();

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
            } else if (options.survey === HiPS.DEFAULT_SURVEY_ID) {
                // DSS is cached inside HiPS class, no need to provide any further information
                const survey = this.createImageSurvey(
                    HiPS.DEFAULT_SURVEY_ID
                );

                this.setBaseImageLayer(survey);
            } else {
                this.setBaseImageLayer(options.survey);
            }
        } else {
            // Add the image layers
            // For that we check the survey key of options
            // It can be given as a single string or an array of strings
            // for multiple blending surveys
            // take in priority the surveyUrl parameter
            let url;
            if (Array.isArray(options.surveyUrl)) {
                // mirrors given, take randomly one
                let numMirrors = options.surveyUrl.length;
                let id = Math.floor(Math.random() * numMirrors);
                url = options.surveyUrl[id];
            } else {
                url = options.surveyUrl;
            }

            this.setBaseImageLayer(url);
        }

        this.view.showCatalog(options.showCatalog);

        // FullScreen toolbar icon
        this.isInFullscreen = false;
        // go to full screen ?
        if (options.fullScreen === true) {
            // strange behaviour to wait for a sec
            self.toggleFullscreen(self.options.realFullscreen);
        }

        // maximize control
        if (options.showFullscreenControl) {
            // react to fullscreenchange event to restore initial width/height (if user pressed ESC to go back from full screen)
            Utils.on(
                document,
                "fullscreenchange webkitfullscreenchange mozfullscreenchange MSFullscreenChange",
                function (e) {
                    var fullscreenElt =
                        document.fullscreenElement ||
                        document.webkitFullscreenElement ||
                        document.mozFullScreenElement ||
                        document.msFullscreenElement;
                    if (fullscreenElt === null || fullscreenElt === undefined) {
                        //self.aladinDiv.classList.remove('aladin-fullscreen');

                        var fullScreenToggledFn =
                            self.callbacksByEventName["fullScreenToggled"];
                        typeof fullScreenToggledFn === "function" &&
                            fullScreenToggledFn(self.isInFullscreen);
                    }
                }
            );
        }

        // set right click context menu
        if (options.showContextMenu) {
            this.contextMenu = new ContextMenu(this);
            this.contextMenu.attach(
                DefaultActionsForContextMenu.getDefaultActions(this)
            );
        }

        if (options.samp) {
            this.samp = new SAMPConnector(this);
        }

        if (options.inertia !== undefined) {
            this.wasm.setInertia(options.inertia);
        }

        if (options.northPoleOrientation) {
            this.setViewCenter2NorthPoleAngle(options.northPoleOrientation);
        }
    };

    Aladin.prototype._setupUI = function (options) {
        let self = this;

        // Status bar
        if (options.showStatusBar) {
            this.statusBar = new StatusBarBox(this);
            this.addUI(this.statusBar);
        }

        // Add the frame control
        if (options.showFrame) {
            this.addUI(new CooFrame(this));
        }

        // Add the location info
        if (options.showCooLocation) {
            this.addUI(new Location(this));
        }

        // Add the FoV info
        if (options.showFov || options.showZoomControl) {
            this.addUI(new FoV(this, options));
        }

        ////////////////////////////////////////////////////
        let stack = new OverlayStackButton(this);
        let simbad = new SimbadPointer(this);
        let grid = new GridEnabler(this);
        this.addUI(stack);
        this.addUI(simbad);
        this.addUI(grid);

        // Add the layers control
        if (!options.showLayersControl) {
            stack._hide();
        }

        // Add the simbad pointer control
        if (!options.showSimbadPointerControl) {
            simbad._hide();
        }

        // Add the projection control
        // Add the coo grid control
        if (!options.showCooGridControl) {
            grid._hide();
        }

        // Settings control
        if (options.showSettingsControl) {
            let settings = new SettingsButton(this, {
                features: { stack, simbad, grid },
            });
            this.addUI(settings);
        }

        // share control panel
        if (options.showShareControl) {
            this.addUI(new ShareActionButton(self));
        }

        if (options.showProjectionControl) {
            this.projBtn = new ProjectionActionButton(this);
            this.addUI(this.projBtn);
        }

        if (options.showFullscreenControl) {
            this.addUI(new FullScreenActionButton(self));
        }

        if (options.expandLayersControl) {
            stack.click();
        }

        this._applyMediaQueriesUI();
    };

    Aladin.prototype._applyMediaQueriesUI = function () {
        const applyMediaQuery = function (
            maxWidth,
            matchingCallback,
            unmatchingCallback
        ) {
            function mqFunction(x) {
                if (x.matches) {
                    // If media query matches
                    matchingCallback();
                } else {
                    unmatchingCallback();
                }
            }

            // Create a MediaQueryList object
            var mq = window.matchMedia("(max-width: " + maxWidth + ")");

            // Attach listener function on state changes
            mq.addEventListener("change", function () {
                mqFunction(mq);
            });

            mqFunction(mq);
        };

        let self = this;

        applyMediaQuery(
            "48rem",
            () => {
                if (self.projBtn) {
                    self.projBtn.update({ verbosity: "reduced" });
                }
            },
            () => {
                if (self.projBtn) {
                    self.projBtn.update({ verbosity: "full" });
                }
            }
        );
    };

    /**** CONSTANTS ****/
    Aladin.VERSION = version;

    Aladin.JSONP_PROXY = "https://alaskybis.cds.unistra.fr/cgi/JSONProxy";
    //Aladin.JSONP_PROXY = "https://alaskybis.unistra.fr/cgi/JSONProxy";

    Aladin.URL_PREVIEWER = "https://aladin.cds.unistra.fr/AladinLite/";

    // access to WASM libraries
    Aladin.wasmLibs = {};
    Aladin.DEFAULT_OPTIONS = {
        survey: HiPS.DEFAULT_SURVEY_ID,
        // surveys suggestion list
        hipsList: HiPSList.DEFAULT,
        //surveyUrl: ["https://alaskybis.unistra.fr/DSS/DSSColor", "https://alasky.unistra.fr/DSS/DSSColor"],
        target: "0 +0",
        cooFrame: "J2000",
        fov: 60,
        northPoleOrientation: 0,
        inertia: true,
        backgroundColor: "rgb(60, 60, 60)",
        // Zoom toolbar
        showZoomControl: true,
        // Menu toolbar
        showLayersControl: true,
        expandLayersControl: false,
        showFullscreenControl: true,
        showSimbadPointerControl: false,
        showCooGridControl: false,
        showSettingsControl: false,
        // Share toolbar
        showShareControl: false,

        // Viewport toolbar
        showFrame: true,
        showFov: true,
        showCooLocation: true,
        showProjectionControl: true,

        // Other UI elements
        showContextMenu: false,
        showStatusBar: true,
        // Internal
        showReticle: true,
        showCatalog: true, // TODO: still used ??

        fullScreen: false,
        reticleColor: "rgb(178, 50, 178)",
        reticleSize: 22,
        gridColor: "rgb(178, 50, 178)",
        gridOpacity: 0.8,
        gridOptions: {
            enabled: false,
            showLabels: true,
            thickness: 2,
            labelSize: 15,
        },
        projection: "SIN",
        log: true,
        samp: false,
        realFullscreen: false,
        pixelateCanvas: true,
        manualSelection: false
    };

    // realFullscreen: AL div expands not only to the size of its parent, but takes the whole available screen estate
    Aladin.prototype.toggleFullscreen = function (realFullscreen) {
        let self = this;

        realFullscreen = Boolean(realFullscreen);
        self.isInFullscreen = !self.isInFullscreen;

        ContextMenu.hideAll();

        this.ui.forEach(ui => {
            if (ui.toggle) {
                ui.toggle();
                ui.toggle();
            }
        })

        //this.fullScreenBtn.attr('title', isInFullscreen ? 'Restore original size' : 'Full screen');

        if (this.aladinDiv.classList.contains("aladin-fullscreen")) {
            this.aladinDiv.classList.remove("aladin-fullscreen");
        } else {
            this.aladinDiv.classList.add("aladin-fullscreen");
        }

        if (realFullscreen) {
            // go to "real" full screen mode
            if (self.isInFullscreen) {
                var d = this.aladinDiv;

                if (d.requestFullscreen) {
                    d.requestFullscreen();
                } else if (d.webkitRequestFullscreen) {
                    d.webkitRequestFullscreen();
                } else if (d.mozRequestFullScreen) {
                    // notice the difference in capitalization for Mozilla functions ...
                    d.mozRequestFullScreen();
                } else if (d.msRequestFullscreen) {
                    d.msRequestFullscreen();
                }
            }
            // exit from "real" full screen mode
            else {
                if (document.exitFullscreen) {
                    document.exitFullscreen();
                } else if (document.webkitExitFullscreen) {
                    document.webkitExitFullscreen();
                } else if (document.mozCancelFullScreen) {
                    document.mozCancelFullScreen();
                } else if (document.webkitExitFullscreen) {
                    document.webkitExitFullscreen();
                }
            }
        }

        // Delay the fixLayoutDimensions layout for firefox
        /*setTimeout(function () {
            self.view.fixLayoutDimensions();
        }, 1000);*/

        // force call to zoomChanged callback
        var fovChangedFn = self.callbacksByEventName["zoomChanged"];
        typeof fovChangedFn === "function" && fovChangedFn(self.view.fov);

        var fullScreenToggledFn =
            self.callbacksByEventName["fullScreenToggled"];
        typeof fullScreenToggledFn === "function" &&
            fullScreenToggledFn(self.isInFullscreen);
    };

    Aladin.prototype.getOptionsFromQueryString = function () {
        var options = {};
        var requestedTarget = Utils.urlParam("target");
        if (requestedTarget) {
            options.target = requestedTarget;
        }
        var requestedFrame = Utils.urlParam("frame");
        if (requestedFrame && CooFrameEnum[requestedFrame]) {
            options.frame = requestedFrame;
        }
        var requestedSurveyId = Utils.urlParam("survey");
        if (
            requestedSurveyId &&
            HiPS.getSurveyInfoFromId(requestedSurveyId)
        ) {
            options.survey = requestedSurveyId;
        }
        var requestedZoom = Utils.urlParam("zoom");
        if (requestedZoom && requestedZoom > 0 && requestedZoom < 180) {
            options.zoom = requestedZoom;
        }

        var requestedShowreticle = Utils.urlParam("showReticle");
        if (requestedShowreticle) {
            options.showReticle = requestedShowreticle.toLowerCase() == "true";
        }

        var requestedCooFrame = Utils.urlParam("cooFrame");
        if (requestedCooFrame) {
            options.cooFrame = requestedCooFrame;
        }

        var requestedFullscreen = Utils.urlParam("fullScreen");
        if (requestedFullscreen !== undefined) {
            options.fullScreen = requestedFullscreen;
        }

        return options;
    };

    /**
     * Sets the field of view (FoV) of the Aladin instance to the specified angle in degrees.
     *
     * @memberof Aladin
     * @param {number} FoV - The angle of the field of view in degrees.
     *
     * @example
     * let aladin = A.aladin('#aladin-lite-div');
     * aladin.setFoV(60);
     */
    Aladin.prototype.setFoV = function (FoV) {
        this.view.setZoom(FoV);
    };

    Aladin.prototype.setFov = Aladin.prototype.setFoV;

    // @API
    // (experimental) try to adjust the FoV to the given object name. Does nothing if object is not known from Simbad
    Aladin.prototype.adjustFovForObject = function (objectName) {
        var self = this;
        this.getFovForObject(objectName, function (fovDegrees) {
            self.setFoV(fovDegrees);
        });
    };

    Aladin.prototype.getFovForObject = Aladin.prototype.getFoVForObject =
        function (objectName, callback) {
            var query =
                "SELECT galdim_majaxis, V FROM basic JOIN ident ON oid=ident.oidref JOIN allfluxes ON oid=allfluxes.oidref WHERE id='" +
                objectName +
                "'";
            var url =
                "//simbad.u-strasbg.fr/simbad/sim-tap/sync?query=" +
                encodeURIComponent(query) +
                "&request=doQuery&lang=adql&format=json&phase=run";

            Utils.fetch({
                url,
                method: "GET",
                dataType: "json",
                success: (result) => {
                    var defaultFov = 4 / 60; // 4 arcmin
                    var fov = defaultFov;

                    if ("data" in result && result.data.length > 0) {
                        var galdimMajAxis = Utils.isNumber(result.data[0][0])
                            ? result.data[0][0] / 60.0
                            : null; // result gives galdim in arcmin
                        var magV = Utils.isNumber(result.data[0][1])
                            ? result.data[0][1]
                            : null;

                        if (galdimMajAxis !== null) {
                            fov = 2 * galdimMajAxis;
                        } else if (magV !== null) {
                            if (magV < 10) {
                                fov = (2 * Math.pow(2.0, 6 - magV / 2.0)) / 60;
                            }
                        }
                    }

                    typeof callback === "function" && callback(fov);
                },
            });
        };

    /**
     * Sets the coordinate frame of the Aladin instance to the specified frame.
     *
     * @memberof Aladin
     * @param {string} frame - The name of the coordinate frame. Possible values: 'j2000d', 'j2000', 'gal', 'icrs'. The given string is case insensitive.
     *
     * @example
     * // Set the coordinate frame to 'J2000'
     * const aladin = A.aladin('#aladin-lite-div');
     * aladin.setFrame('J2000');
     */
    Aladin.prototype.setFrame = function (frame) {
        if (!frame) {
            return;
        }
        var newFrame = CooFrameEnum.fromString(frame, CooFrameEnum.J2000);
        if (newFrame == this.view.cooFrame) {
            return;
        }

        this.view.changeFrame(newFrame);

        var frameChangedFunction = this.callbacksByEventName["cooFrameChanged"];
        if (typeof frameChangedFunction === "function") {
            frameChangedFunction(newFrame.label);
        }
    };

    /**
     * Sets the projection of the Aladin instance to the specified type.
     *
     * @memberof Aladin
     * @param {string} projection The type of projection to set. Possible values
     * <br>"TAN" (Gnomonic projection)
     * <br>"STG" (Stereographic projection)
     * <br>"SIN" (Orthographic projection)
     * <br>"ZEA" (Zenital equal-area projection)
     * <br>"MER" (Mercator projection)
     * <br>"AIT" (Hammer-Aitoff projection)
     * <br>"MOL" (Mollweide projection)
     *
     * @example
     * // Set the projection to 'orthographic'
     * let aladin = A.aladin('#aladin-lite-div');
     * aladin.setProjection('SIN');
     */
    Aladin.prototype.setProjection = function (projection) {
        if (!projection) {
            return;
        }
        this.view.setProjection(projection);

        ALEvent.PROJECTION_CHANGED.dispatchedTo(this.aladinDiv, {
            projection,
        });
    };

    /**
     * Append a message to the status bar
     *
     * @memberof Aladin
     * @param {Object} options - The message to display
     * @param {string} options.id - The id of the message, is useful for removing unlimited time messages
     * @param {string} options.message - The message to display
     * @param {string|number} options.duration - The duration of the message. Accepts a time in milliseconds or 'unlimited'
     * @param {string} options.type - The type of the message. Can be 'loading', 'tooltip', 'info'
     *
     * @example
     *
     * aladin.addStatusBarMessage({
     *       duration: 10000,
     *       type: 'info',
     *       message: 'Aladin Lite v3.3 is out. New features available:<ul><li>New Button, Box objects</li><li>Polygonal, circular selection</li></ul>'
     * })
     */
    Aladin.prototype.addStatusBarMessage = function (options) {
        if (this.statusBar) {
            this.statusBar.appendMessage(options);
        }
    };

/**
     * Remove a message from the status bar
     *
     * @memberof Aladin
     * @param {string} id - The id of the message to remove
     */
    Aladin.prototype.removeStatusBarMessage = function (id) {
        if (this.statusBar) {
            this.statusBar.removeMessage(id);
        }
    };

    Aladin.prototype.getProjectionName = function () {
        const self = this;

        let projName = undefined;
        for (let key in ProjectionEnum) {
            if (ProjectionEnum[key] == self.view.projection) {
                projName = key;
                break;
            }
        }

        return projName;
    };
    ``;

    /**
     * Returns the current coordinate system: possible values are 'J2000', 'J2000d', and 'Galactic' .
     *
     * @memberof Aladin
     * @returns {string} The current coordinate system: possible values are 'J2000', 'J2000d', and 'Galactic' .
     *
     * @example
     * const aladin = A.aladin('#aladin-lite-div', {cooFrame: 'galactic'});
     * let cooFrame = aladin.getFrame();
     * assert(cooFrame, 'galactic')
     */
    Aladin.prototype.getFrame = function () {
        return this.view.cooFrame.label;
    };

    /**
     * Moves the Aladin instance to the specified astronomical object.
     *
     * @memberof Aladin
     * @param {string} targetName - The name or identifier of the astronomical object to move to.
     * @param {Object} [callbackOptions] - Optional callback options.
     * @param {function} [callbackOptions.success] - The callback function to execute on successful navigation.
     * @param {function} [callbackOptions.error] - The callback function to execute on error during navigation.
     *
     * @example
     * // Move to the astronomical object named 'M42' with callbacks
     * const aladinInstance = A.aladin('#aladin-lite-div');
     * aladinInstance.gotoObject('M42', {
     *   success: () => {
     *     console.log('Successfully moved to M42.');
     *   },
     *   error: (err) => {
     *     console.error('Error moving to M42:', err);
     *   }
     * });
     */
    Aladin.prototype.gotoObject = function (targetName, callbackOptions) {
        let successCallback = undefined;
        let errorCallback = undefined;
        if (typeof callbackOptions === "object") {
            if (callbackOptions.hasOwnProperty("success")) {
                successCallback = callbackOptions.success;
            }
            if (callbackOptions.hasOwnProperty("error")) {
                errorCallback = callbackOptions.error;
            }
        }
        // this is for compatibility reason with the previous method signature which was function(targetName, errorCallback)
        else if (typeof callbackOptions === "function") {
            errorCallback = callbackOptions;
        }

        var isObjectName = /[a-zA-Z]/.test(targetName);

        // try to parse as a position
        if (!isObjectName) {
            var coo = new Coo();
            coo.parse(targetName);
            // Convert from view coo sys to icrs
            const [ra, dec] = this.wasm.viewToICRSCooSys(coo.lon, coo.lat);

            this.view.pointTo(ra, dec);

            typeof successCallback === "function" &&
                successCallback(this.getRaDec());
        }
        // ask resolution by Sesame
        else {
            var self = this;
            // sky case
            (async () => {
                let baseImageLayer;
                if (this.getBaseImageLayer()) {
                    baseImageLayer = await this.getBaseImageLayer().query;
                }
                if (
                    this.getBaseImageLayer() === undefined ||
                    !baseImageLayer.isPlanetaryBody()
                ) {
                    Sesame.resolve(
                        targetName,
                        function (data) {
                            // success callback
                            // Location given in icrs at J2000
                            const coo = data.coo;
                            self.view.pointTo(coo.jradeg, coo.jdedeg);

                            typeof successCallback === "function" &&
                                successCallback(self.getRaDec());
                        },
                        function (data) {
                            // errror callback
                            if (console) {
                                console.log(
                                    "Could not resolve object name " +
                                        targetName
                                );
                                console.log(data);
                            }
                            typeof errorCallback === "function" &&
                                errorCallback();
                        }
                    );
                }
                // planetary case
                else {
                    const body = baseImageLayer.hipsBody;
                    PlanetaryFeaturesNameResolver.resolve(
                        targetName,
                        body,
                        function (data) {
                            // success callback
                            self.view.pointTo(data.lon, data.lat);

                            typeof successCallback === "function" &&
                                successCallback(self.getRaDec());
                        },
                        function (data) {
                            // error callback
                            if (console) {
                                console.log(
                                    "Could not resolve object name " +
                                        targetName
                                );
                                console.log(data);
                            }
                            typeof errorCallback === "function" &&
                                errorCallback();
                        }
                    );
                }
            })();
        }
    };

    /**
     * Moves the Aladin instance to the specified position.
     *
     * @memberof Aladin
     * @param {number} lon - longitude in degrees
     * @param {number} lat - latitude in degrees
     * @param {string} [frame] - The name of the coordinate frame. Possible values: 'j2000d', 'j2000', 'gal', 'icrs'. The given string is case insensitive.
     *
     * @example
     * // Move to position
     * const aladin = A.aladin('#aladin-lite-div');
     * aladin.gotoPosition(20, 10, "galactic");
     */
    Aladin.prototype.gotoPosition = function (lon, lat, frame) {
        var radec;
        // convert the frame from string to CooFrameEnum
        if (frame) {
            frame = CooFrameEnum.fromString(
                this.options.cooFrame,
                CooFrameEnum.J2000
            );
        }
        // both are CooFrameEnum
        let positionGivenFrame = frame || this.view.cooFrame;
        // First, convert to J2000 if needed
        if (positionGivenFrame === CooFrameEnum.GAL) {
            radec = CooConversion.GalacticToJ2000([lon, lat]);
        } else {
            radec = [lon, lat];
        }

        this.gotoRaDec(radec[0], radec[1]);
    };

    var idTimeoutAnim;
    var doAnimation = function (aladin) {
        /*if (idTimeoutAnim) {
            clearTimeout(idTimeoutAnim)
        }*/

        var params = aladin.animationParams;
        if (params == null || !params["running"]) {
            return;
        }
        var now = new Date().getTime();
        // this is the animation end: set the view to the end position, and call complete callback
        if (now > params["end"]) {
            aladin.gotoRaDec(params["raEnd"], params["decEnd"]);

            if (params["complete"]) {
                params["complete"]();
            }

            return;
        }

        // compute current position
        var fraction =
            (now - params["start"]) / (params["end"] - params["start"]);
        var curPos = intermediatePoint(
            params["raStart"],
            params["decStart"],
            params["raEnd"],
            params["decEnd"],
            fraction
        );
        var curRa = curPos[0];
        var curDec = curPos[1];
        //var curRa =  params['raStart'] + (params['raEnd'] - params['raStart']) * (now-params['start']) / (params['end'] - params['start']);
        //var curDec = params['decStart'] + (params['decEnd'] - params['decStart']) * (now-params['start']) / (params['end'] - params['start']);

        aladin.gotoRaDec(curRa, curDec);

        //idTimeoutAnim = setTimeout(function () { doAnimation(aladin); }, 10);
        requestAnimFrame(() => {
            doAnimation(aladin);
        });
    };

    /*
     * Stop all animations that have been initiated  by animateToRaDec or by zoomToFoV
     * @API
     *
     */
    Aladin.prototype.stopAnimation = function () {
        if (this.zoomAnimationParams) {
            this.zoomAnimationParams["running"] = false;
        }
        if (this.animationParams) {
            this.animationParams["running"] = false;
        }
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
    Aladin.prototype.animateToRaDec = function (ra, dec, duration, complete) {
        duration = duration || 5;

        this.animationParams = null;

        var animationParams = {};
        animationParams["start"] = new Date().getTime();
        animationParams["end"] = new Date().getTime() + 1000 * duration;
        var raDec = this.getRaDec();
        animationParams["raStart"] = raDec[0];
        animationParams["decStart"] = raDec[1];
        animationParams["raEnd"] = ra;
        animationParams["decEnd"] = dec;
        animationParams["complete"] = complete;
        animationParams["running"] = true;

        this.animationParams = animationParams;

        doAnimation(this);
    };

    var doZoomAnimation = function (aladin) {
        var params = aladin.zoomAnimationParams;
        if (params == null || !params["running"]) {
            return;
        }
        var now = new Date().getTime();
        // this is the zoom animation end: set the view to the end fov, and call complete callback
        if (now > params["end"]) {
            aladin.setFoV(params["fovEnd"]);

            if (params["complete"]) {
                params["complete"]();
            }

            return;
        }

        // compute current position
        var fraction =
            (now - params["start"]) / (params["end"] - params["start"]);
        var curFov =
            params["fovStart"] +
            (params["fovEnd"] - params["fovStart"]) * Math.sqrt(fraction);

        aladin.setFoV(curFov);

        setTimeout(function () {
            doZoomAnimation(aladin);
        }, 50);
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
        zoomAnimationParams["start"] = new Date().getTime();
        zoomAnimationParams["end"] = new Date().getTime() + 1000 * duration;
        var fovArray = this.getFov();
        zoomAnimationParams["fovStart"] = Math.max(fovArray[0], fovArray[1]);
        zoomAnimationParams["fovEnd"] = fov;
        zoomAnimationParams["complete"] = complete;
        zoomAnimationParams["running"] = true;

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
            return (d * Math.PI) / 180;
        }
        function radToDeg(r) {
            return (r * 180) / Math.PI;
        }
        var lat1 = degToRad(lat1);
        var lng1 = degToRad(lng1);
        var lat2 = degToRad(lat2);
        var lng2 = degToRad(lng2);
        var d =
            2 *
            Math.asin(
                Math.sqrt(
                    Math.pow(Math.sin((lat1 - lat2) / 2), 2) +
                        Math.cos(lat1) *
                            Math.cos(lat2) *
                            Math.pow(Math.sin((lng1 - lng2) / 2), 2)
                )
            );
        var A = Math.sin((1 - fraction) * d) / Math.sin(d);
        var B = Math.sin(fraction * d) / Math.sin(d);
        var x =
            A * Math.cos(lat1) * Math.cos(lng1) +
            B * Math.cos(lat2) * Math.cos(lng2);
        var y =
            A * Math.cos(lat1) * Math.sin(lng1) +
            B * Math.cos(lat2) * Math.sin(lng2);
        var z = A * Math.sin(lat1) + B * Math.sin(lat2);
        var lon = Math.atan2(y, x);
        var lat = Math.atan2(z, Math.sqrt(Math.pow(x, 2) + Math.pow(y, 2)));

        return [radToDeg(lon), radToDeg(lat)];
    }

    /**
     * Gets the current [Right Ascension, Declination] position of the center of the Aladin view.
     *
     * This method returns the celestial coordinates of the center of the Aladin view in the International
     * Celestial Reference System (ICRS) or J2000 equatorial coordinates.
     *
     * @memberof Aladin
     * @returns {number[]} - An array representing the [Right Ascension, Declination] coordinates in degrees.
     *                       The first element is the Right Ascension (RA), and the second element is the Declination (Dec).
     */
    Aladin.prototype.getRaDec = function () {
        let radec = this.wasm.getCenter(); // This is given in the frame of the view
        // We must convert it to ICRS
        const radec_j2000 = this.wasm.viewToICRSCooSys(radec[0], radec[1]);

        if (radec_j2000[0] < 0) {
            return [radec_j2000[0] + 360.0, radec_j2000[1]];
        }

        return radec_j2000;
    };

    /**
     * Moves the Aladin instance to the specified position given in ICRS frame
     *
     * @memberof Aladin
     * @param {number} ra - Right-ascension in degrees
     * @param {number} dec - Declination in degrees
     *
     * @example
     * const aladin = A.aladin('#aladin-lite-div');
     * aladin.gotoRaDec(20, 10);
     */
    Aladin.prototype.gotoRaDec = function (ra, dec) {
        this.view.pointTo(ra, dec);
    };

    Aladin.prototype.showHealpixGrid = function (show) {
        this.view.showHealpixGrid(show);
    };

    Aladin.prototype.healpixGrid = function () {
        return this.view.displayHpxGrid;
    };

    Aladin.prototype.showSurvey = function (show) {
        this.view.showSurvey(show);
    };

    Aladin.prototype.showCatalog = function (show) {
        this.view.showCatalog(show);
    };

    Aladin.prototype.showReticle = function (show) {
        this.reticle.update({ show });
    };

    Aladin.prototype.getReticle = function () {
        return this.reticle;
    };

    // these 4 methods should be merged into a unique "add" method
    Aladin.prototype.addCatalog = function (catalog) {
        this.view.addCatalog(catalog);

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(this.aladinDiv, {
            layer: catalog,
        });
    };

    Aladin.prototype.addOverlay = function (overlay) {
        this.view.addOverlay(overlay);

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(this.aladinDiv, {
            layer: overlay,
        });
    };

    Aladin.prototype.addMOC = function (moc) {
        this.view.addMOC(moc);

        // see MOC.setView for sending it to outside the UI
    };

    Aladin.prototype.addUI = function (ui) {
        ui = [].concat(ui);

        for (var ui of ui) {
            this.ui.push(ui);
            ui.attachTo(this.aladinDiv);
    
            // as the ui is pushed to the dom, setting position may need the aladin instance to work
            // so we recompute it
            if (ui.options) {
                ui.update({ position: { ...ui.options.position, aladin: this } });
            }
        }
    };

    // @API
    Aladin.prototype.findLayerByUUID = function (uuid) {
        const result = this.view.allOverlayLayers.filter(
            (layer) => layer.uuid === uuid
        );
        if (result.length == 0) {
            return null;
        }

        return result[0];
    };

    /**
     * Remove all the overlays (MOC, Overlay, ProgressiveCat, Catalog) from the view
     * @memberof Aladin
     */
    Aladin.prototype.removeOverlays = function () {
        this.view.removeOverlays();
    };

    /**
     * @deprecated
     * Old method name, use {@link Aladin.prototype.removeOverlays} instead.
     * @memberof Aladin
     */
    Aladin.prototype.removeLayers = Aladin.prototype.removeOverlays;
    /**
    * @typedef {MOC|Catalog|ProgressiveCat|GraphicOverlay} Overlay
    * @description Possible overlays
    */
    /**
     * Remove an overlay by its layer name
     *
     * @memberof Aladin
     * @param {string|Overlay} overlay - The name of the overlay to remove or the overlay object itself
     */
    Aladin.prototype.removeOverlay = function (overlay) {
        if(typeof overlay === 'string' || overlay instanceof String) {
            this.view.removeOverlayByName(overlay);
        } else {
            this.view.removeOverlay(overlay);
        }
    };

    /**
     * @deprecated
     * Old method name, use {@link Aladin.prototype.removeOverlay} instead.
     * @memberof Aladin
     */
    Aladin.prototype.removeLayer = Aladin.prototype.removeOverlay;

    /**
     * @memberof Aladin
     * @param {string} id - Mandatory unique identifier for the survey.
     * @param {string} [name] - A convinient name for the survey, optional
     * @param {string|FileList|HiPSLocalFiles} url - Can be:
     * <ul>
     * <li>An http url towards a HiPS.</li>
     * <li>A relative path to your HiPS</li>
     * <li>A special ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>A dict storing a local HiPS files. This object contains a tile file: hips[order][ipix] = File and refers to the properties file like so: hips["properties"] = File. </li>
     *     A javascript {@link FileList} pointing to the opened webkit directory is also accepted.
     * </ul>
     * @param {string} [cooFrame] - Values accepted: 'equatorial', 'icrs', 'icrsd', 'j2000', 'gal', 'galactic'
     * @param {number} [maxOrder] - The maximum HEALPix order of the HiPS, i.e the HEALPix order of the most refined tile images of the HiPS.
     * @param {HiPSOptions} [options] - Options describing the survey
     * @returns {HiPS} A HiPS image object.
     */
    Aladin.prototype.createImageSurvey = function (
        id,
        name,
        url,
        cooFrame,
        maxOrder,
        options
    ) {
        let hips = new HiPS(id, url || id, { name, maxOrder, url, cooFrame, ...options })

        if (this instanceof Aladin && !this.hipsCache.contains(id)) {
            // Add it to the cache as soon as possible if we have a reference to the aladin object
            this.hipsCache.append(hips.id, hips)
        }

        return hips;
    };

    /**
     * @function createImageSurvey
     * @memberof Aladin
     * @static
     * @param {string} id - Mandatory unique identifier for the survey.
     * @param {string} [name] - A convinient name for the survey, optional
     * @param {string|FileList|HiPSLocalFiles} url - Can be:
     * <ul>
     * <li>An http url towards a HiPS.</li>
     * <li>A relative path to your HiPS</li>
     * <li>A special ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>A dict storing a local HiPS files. This object contains a tile file: hips[order][ipix] = File and refers to the properties file like so: hips["properties"] = File. </li>
     *     A javascript {@link FileList} pointing to the opened webkit directory is also accepted.
     * </ul>
     * @param {string} [cooFrame] - Values accepted: 'equatorial', 'icrs', 'icrsd', 'j2000', 'gal', 'galactic'
     * @param {number} [maxOrder] - The maximum HEALPix order of the HiPS, i.e the HEALPix order of the most refined tile images of the HiPS.
     * @param {HiPSOptions} [options] - Options describing the survey
     * @returns {HiPS} A HiPS image object.
     */
    Aladin.createImageSurvey = Aladin.prototype.createImageSurvey;

    /**
     * Remove a HiPS/FITS image from the list of favorites.
     * 
     * @throws A warning when the asset is currently present in the view
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS</li>
     * <li>2. Or it can be a CDS identifier that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object created from {@link A.HiPS}</li>
     * <li>4. A {@link Image} FITS image object</li>
     * </ul>
     */
    Aladin.prototype.removeHiPSFromFavorites = function (survey) {
        if (this.contains(survey)) {
            // TODO: handle this case
            console.warn(survey + ' is among the list of HiPS currently in the view.');
        }

        if (typeof survey !== "string") {
            survey = survey.id;
        }
        
        this.hipsCache.delete(survey);
    }

    /**
     * Check whether a survey is currently in the view
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS</li>
     * <li>2. Or it can be a CDS identifier that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object</li>
     * <li>4. A {@link Image} Image object</li>
     * </ul>
     */
    Aladin.prototype.contains = function(survey) {
        this.view.contains(survey)
    }

    /**
     * Creates a FITS image object
     * @deprecated prefer use {@link A.image}
     *
     * @function createImageFITS
     * @memberof Aladin
     * @static
     * @param {string} url - The url of the fits.
     * @param {ImageOptions} [options] - Options for rendering the image
     * @param {function} [success] - A success callback
     * @param {function} [error] - A success callback
     * @returns {Image} A FITS image object.
     */
    Aladin.prototype.createImageFITS = function (
        url,
        options,
        successCallback,
        errorCallback
    ) {
        try {
            url = new URL(url);
        } catch (e) {
            // The url could be created
            url = Utils.getAbsoluteURL(url);
            url = new URL(url);
        }

        url = url.toString();

        // Do not use proxy with CORS headers until we solve that: https://github.com/MattiasBuelens/wasm-streams/issues/20
        //url = Utils.handleCORSNotSameOrigin(url).href;

        let image = new Image(url, {...options, successCallback, errorCallback});

        return image;
    };

    /**
     * @deprecated prefer use {@link A.imageFITS} instead
     * Creates a FITS image object
     *
     * @function createImageFITS
     * @memberof Aladin
     * @static
     * @param {string} url - The url of the fits.
     * @param {ImageOptions} [options] - Options for rendering the image
     * @param {function} [success] - A success callback
     * @param {function} [error] - A success callback
     * @returns {Image} A FITS image object.
     */
    Aladin.createImageFITS = Aladin.prototype.createImageFITS;

    /**
     * @deprecated
     * Create a new layer from an url or CDS ID.
     * Please use {@link A.hiPS} instead for creating a new survey image
     *
     * @memberof Aladin
     * @param {string} id - Can be:
     * <ul>
     * <li>An http url towards a HiPS.</li>
     * <li>A relative path to your HiPS</li>
     * <li>A special ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * </ul>
     * @param {HiPSOptions} [options] - Options for rendering the image
     * @param {function} [success] - A success callback
     * @param {function} [error] - A success callback
     * @returns {HiPS} A FITS image object.
     */
    Aladin.prototype.newImageSurvey = function (id, options) {
        // a wrapper on createImageSurvey that aggregates all params in an options object
        return this.createImageSurvey(
            id, 
            options && options.name,
            id,
            options && options.cooFrame,
            options && options.maxOrder,
            options
        );
    };

    /**
     * Add a new HiPS layer to the view on top of the others
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} [survey="P/DSS2/color"] - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS.</li>
     * <li>2. Or it can be a CDS ID that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}.</li>
     * <li>3. It can also be an {@link A.HiPS} HiPS object created from {@link A.HiPS}</li>
     * </ul>
     * By default, the {@link https://alasky.cds.unistra.fr/DSS/DSSColor/|Digital Sky Survey 2} survey will be displayed
     */
    Aladin.prototype.addNewImageLayer = function (survey = "P/DSS2/color") {
        let layerName = Utils.uuidv4();
        return this.setOverlayImageLayer(survey, layerName);
    };

    /**
     * Change the base layer of the view
     *
     * It internally calls {@link Aladin#setBaseImageLayer|Aladin.setBaseImageLayer} with the url/{@link HiPS}/{@link Image} given
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS.</li>
     * <li>2. Or it can be a CDS identifier that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object created from {@link A.HiPS}</li>
     * <li>4. A {@link Image} FITS image object</li>
     * </ul>
     */
    Aladin.prototype.setImageLayer = function (imageLayer) {
        this.setBaseImageLayer(imageLayer);
    };

    /**
     * Change the base layer of the view
     *
     * It internally calls {@link Aladin#setBaseImageLayer|Aladin.setBaseImageLayer} with the url/{@link HiPS}/{@link Image} given
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS.</li>
     * <li>2. Or it can be a CDS ID that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object created from {@link A.HiPS}</li>
     * <li>4. A {@link Image} FITS image object</li>
     * </ul>
     */
    Aladin.prototype.setImageSurvey = Aladin.prototype.setImageLayer;

    // @param imageSurvey : ImageSurvey object or image survey identifier
    // @api
    // @old
    Aladin.prototype.setBackgroundColor = function (rgb) {
        this.backgroundColor = new Color(rgb);
        // Once the wasm is ready, send the color to change it

        ALEvent.AL_USE_WASM.dispatchedTo(this.aladinDiv, {
            callback: (wasm) => {
                wasm.setBackgroundColor(this.backgroundColor);
                ALEvent.BACKGROUND_COLOR_CHANGED.dispatchedTo(this.aladinDiv, {
                    color: this.backgroundColor,
                });
            },
        });
    };

    Aladin.prototype.getBackgroundColor = function () {
        return this.backgroundColor;
    };

    /**
     * Remove an image layer/overlay from the instance
     *
     * @memberof Aladin
     * @param {string|Overlay} item - the overlay object or image layer name to remove
     */
     Aladin.prototype.remove = function (item) {
        const layers = this.getStackLayers()
        let idxToDelete = layers.findIndex(l => l === item);
        if (idxToDelete >= 0) {
            this.view.removeImageLayer(item);
            return;
        }

        // must be an overlay
        this.view.removeOverlay(item)
    };

    /**
     * Remove a specific layer
     *
     * @memberof Aladin
     * @param {string} layer - The name of the layer to remove or the HiPS/Image object
     */
    Aladin.prototype.removeImageLayer = function (layer) {
        this.view.removeImageLayer(layer);
    };

    /**
     * Change the base layer of the view
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS.</li>
     * <li>2. Or it can be a CDS ID that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object created from {@link A.HiPS}</li>
     * <li>4. A {@link Image} FITS image object</li>
     * </ul>
     */
    Aladin.prototype.setBaseImageLayer = function (urlOrHiPSOrFITS) {
        return this.setOverlayImageLayer(urlOrHiPSOrFITS, "base");
    };

    /**
     * Get the base image layer object
     *
     * @memberof Aladin
     * @returns {HiPS|Image} - Returns the image layer corresponding to the base layer
     */
    Aladin.prototype.getBaseImageLayer = function () {
        return this.view.getImageLayer("base");
    };

    /**
     * Add a new HiPS/FITS image layer in the view
     *
     * @memberof Aladin
     * @param {string|HiPS|Image} urlOrHiPSOrFITS - Can be:
     * <ul>
     * <li>1. An url that refers to a HiPS.</li>
     * <li>2. Or it can be a CDS ID that refers to a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>3. A {@link HiPS} HiPS object created from {@link A.HiPS}</li>
     * <li>4. A {@link Image} FITS/jpeg/png image</li>
     * </ul>
     * @param {string} [layer="overlay"] - A layer name. By default 'overlay' is chosen and it is destined to be plot
     * on top the 'base' layer. If the layer is already present in the view, it will be replaced by the new HiPS/FITS image given here.
     */
    Aladin.prototype.setOverlayImageLayer = function (
        urlOrHiPSOrFITS,
        layer = "overlay"
    ) {
        let imageLayer;

        let hipsCache = this.hipsCache;
        // 1. User gives an ID
        if (typeof urlOrHiPSOrFITS === "string") {
            const idOrUrl = urlOrHiPSOrFITS;
            // many cases here
            // 1/ It has been already added to the cache
            let cachedLayer = hipsCache.get(idOrUrl)
            if (cachedLayer) {
                imageLayer = cachedLayer
            } else {
                // 2/ Not in the cache, then we create the hips from this url/id and 
                // go to the case 3
                imageLayer = A.HiPS(idOrUrl);
                return this.setOverlayImageLayer(imageLayer, layer);
            }
        } else {
            // 3/ It is an image survey.
            imageLayer = urlOrHiPSOrFITS;

            let cachedLayer = hipsCache.get(imageLayer.id)
            if (!cachedLayer) {
                hipsCache.append(imageLayer.id, imageLayer)
            } else {
                // first set the options of the cached layer to the one of the user
                cachedLayer.setOptions(imageLayer.options)
                // if it is in the cache we get it from the cache
                imageLayer = cachedLayer
            }
        }

        return this.view.setOverlayImageLayer(imageLayer, layer);
    };

    /**
     * Get an image layer from a layer name
     *
     * @memberof Aladin
     * @param {string} [layer="overlay"] - The name of the layer

     * @returns {HiPS|Image} - The requested image layer.
     */
    Aladin.prototype.getOverlayImageLayer = function (layer = "overlay") {
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

     /**
     * Set the view center rotation in degrees
     *
     * @memberof Aladin
     * @param {number} rotation - The center rotation in degrees. Positive angles rotates the
     * view in the counter clockwise order (or towards the east)
     */
    Aladin.prototype.setViewCenter2NorthPoleAngle = function (rotation) {
        this.view.setViewCenter2NorthPoleAngle(rotation);
    };

     /**
     * Get the view center to north pole angle in degrees. This is equivalent to getting the 3rd Euler angle
     *
     * @memberof Aladin
     * 
     * @returns {number} - Angle between the position center and the north pole
     */
    Aladin.prototype.getViewCenter2NorthPoleAngle = function () {
        return this.view.wasm.getViewCenter2NorthPoleAngle();
    };

    // @api
    // Set the current layer that is targeted
    // Rightclicking for changing the cuts is done the targeted layer
    Aladin.prototype.selectLayer = function (layer) {
        this.view.selectLayer(layer);
    };

    Aladin.prototype.getSelectedLayer = function () {
        return this.view.selectedLayer;
    };

    /**
     * Get list of overlays layers
     *
     * @memberof Aladin
     * @returns {MOC[]|Catalog[]|ProgressiveCat[]|GraphicOverlay[]} - Returns the ordered list of image layers. Items can be {@link HiPS} or {@link Image} objects.
     */
    Aladin.prototype.getOverlays = function () {
        return this.view.allOverlayLayers;
    };

    /**
     * Get list of layers
     *
     * @memberof Aladin
     * @returns {HiPS[]|Image[]} - Returns the ordered list of image layers. Items can be {@link HiPS} or {@link Image} objects.
     */
    Aladin.prototype.getStackLayers = function () {
        return this.view.overlayLayers;
    };

    Aladin.prototype.isHpxGridDisplayed = function () {
        return this.view.displayHpxGrid;
    };

    Aladin.prototype.isReticleDisplayed = function () {
        return this.reticle.isVisible();
    };

    /**
     * @deprecated
     * Please use {@link A.catalogHiPS} instead
     */
    Aladin.prototype.createProgressiveCatalog = function (
        url,
        frame,
        maxOrder,
        options
    ) {
        return new ProgressiveCat(url, frame, maxOrder, options);
    };

    /**
     * @deprecated
     * Please use {@link A.graphicOverlay} instead
     */
    Aladin.prototype.createOverlay = function (options) {
        return new GraphicOverlay(options);
    };

    // Select corresponds to rectangular selection
    Aladin.AVAILABLE_CALLBACKS = [
        "select", // deprecated, use objectsSelected instead
        "objectsSelected",

        "objectClicked",
        "objectHovered",
        "objectHoveredStop",

        "footprintClicked",
        "footprintHovered",

        "positionChanged",
        "zoomChanged",

        "click",
        "rightClickMove",
        "mouseMove",

        "fullScreenToggled",
        "cooFrameChanged",
        "resizeChanged",
        "projectionChanged",
        "layerChanged"
    ];

    /**
     * Listen aladin for specific events
     *
     * @memberof Aladin
     * @param {ListenerCallback} what - e.g. objectHovered, select, zoomChanged, positionChanged
     * @param {function} myFunction - a callback function.
     * Note: <ul>
     * <li>positionChanged and zoomChanged are throttled every 100ms.</li>
     * <li>positionChanged's callback gives an object having ra and dec keywords of the current position in ICRS frame. See the below example.</li>
     * </ul>
     * @example
// define function triggered when  a source is hovered
aladin.on('objectHovered', function(object, xyMouseCoords) {
    if (object) {
        msg = 'You hovered object ' + object.data.name + ' located at ' + object.ra + ', ' + object.dec + '; mouse coords - x: '
            + xyMouseCoords.x + ', y: ' + xyMouseCoords.y;
    }
    else {
        msg = 'No object hovered';
    }
    $('#infoDiv').html(msg);
});

aladin.on('objectHoveredStop', function(object, xyMouseCoords) {
    if (object) {
        msg = 'You stopped hove object ' + object.data.name + ' located at ' + object.ra + ', ' + object.dec + '; mouse coords - x: '
            + xyMouseCoords.x + ', y: ' + xyMouseCoords.y;
    }
    $('#infoDiv').html(msg);
});

// define function triggered when an object is clicked
var objClicked;
aladin.on('objectClicked', function(object, xyMouseCoords) {
    if (object) {
        objClicked = object;
        object.select();
        msg = 'You clicked object ' + object.data.name + ' located at ' + object.ra + ', ' + object.dec + '; mouse coords - x: '
            + xyMouseCoords.x + ', y: ' + xyMouseCoords.y;
    }
    else {
        objClicked.deselect();
        msg = 'You clicked in void';
    }
    $('#infoDiv').html(msg);
});

aladin.on("objectsSelected", (objs) => {
    console.log("objs", objs)
})

aladin.on("positionChanged", ({ra, dec}) => {
    console.log("positionChanged", ra, dec)
})

aladin.on("layerChanged", (layer, layerName, state) => {
    console.log("layerChanged", layer, layerName, state)
})
     */
    Aladin.prototype.on = function (what, myFunction) {
        if (Aladin.AVAILABLE_CALLBACKS.indexOf(what) < 0) {
            return;
        }

        this.callbacksByEventName[what] = myFunction;

        /*if (what === "positionChanged") {
            // tell the backend about that callback
            // because it needs to be called when the inertia is done
            ALEvent.AL_USE_WASM.dispatchedTo(this.aladinDiv, {
                callback: (wasm) => {
                    let myFunctionThrottled = Utils.throttle(
                        myFunction,
                        View.CALLBACKS_THROTTLE_TIME_MS
                    );

                wasm.setCallbackPositionChanged(myFunctionThrottled);
            }})
        }*/
    };

    Aladin.prototype.addListener = function (alEventName, customFn) {
        new ALEvent(alEventName).listenedBy(this.aladinDiv, customFn);
    };

    /**
     * Select specific objects in the view
     * 
     * @memberof Aladin
     * @param {?Array.<Source, Footprint, Circle, Ellipse, Polyline, Vector>} objects - If null is passed then nothing will be selected and sources already selected will be deselected
     */
    Aladin.prototype.selectObjects = function (objects) {
        if (!objects) {
            this.view.unselectObjects();
            return;
        }

        let objListPerCatalog = {};

        for (let o of objects) {
            let cat = o.getCatalog();
            if (cat) {
                let objList = objListPerCatalog[cat.name];
                if (!objList) {
                    objList = [];
                } else {
                    objList.push(o);
                }
            }
        }
        objects = Object.values(objListPerCatalog);

        this.view.selectObjects(objects);
    };

    /**
     * Enters selection mode
     *
     * @memberof Aladin
     * @param {string} [mode='rect'] - The mode of selection, can be either, 'rect', 'poly', or 'circle'
     * @param {function} [callback] - A function called once the selection has been done
     * The callback accepts one parameter depending of the mode used: <br/>
     * - If mode='circle' that parameter is of type {@link CircleSelection} <br/>
     * - If mode='rect' that parameter is of type {@link RectSelection} <br/>
     * - If mode='poly' that parameter is of type {@link PolygonSelection}
     *
     * @example
     * // Creates and add a MOC from the user polygonal selection
     * aladin.select('poly', p => {
     *    try {
     *        let ra = []
     *        let dec = []
     *        for (const v of p.vertices) {
     *            let [lon, lat] = aladin.pix2world(v.x, v.y);
     *            ra.push(lon)
     *            dec.push(lat)
     *        }
     *
     *        let moc = A.MOCFromPolygon(
     *            {ra, dec},
     *            {name: 'poly', lineWidth: 3.0, color: 'pink'},
     *        );
     *        aladin.addMOC(moc)
     *    } catch(_) {
     *        alert('Selection covers a region out of the projection definition domain.');
     *    }
     *})
     */
    Aladin.prototype.select = async function (mode = "rect", callback) {
        await this.reticle.loaded;

        this.fire("selectstart", { mode, callback });
    };

    Aladin.prototype.fire = function (what, params) {
        if (what === "selectstart") {
            const { mode, callback } = params;
            this.view.startSelection(mode, callback);
        } else if (what === "simbad") {
            this.view.setMode(View.TOOL_SIMBAD_POINTER);
        } else if (what === "default") {
            this.view.setMode(View.PAN);
        }
    };

    Aladin.prototype.hideBoxes = function () {
        if (this.boxes) {
            for (var k = 0; k < this.boxes.length; k++) {
                if (typeof this.boxes[k].hide === "function") {
                    this.boxes[k].hide();
                }
            }
        }
    };

    // TODO : LayerBox (or Stack?) must be extracted as a separate object
    Aladin.prototype.showLayerBox = function () {
        this.stack.showImageLayerBox();
    };

    /**
     * Sets the coordinate grid options for the Aladin Lite view.
     *
     * This method allows you to customize the appearance of the coordinate grid in the Aladin Lite view.
     *
     * @memberof Aladin
     * @param {Object} options - Options to customize the coordinate grid.
     * @param {string} [options.color] - The color of the coordinate grid.
     * @param {number} [options.opacity] - The opacity of the coordinate grid (value between 0 and 1).
     * @param {number} [options.labelSize] - The size of the coordinate grid labels in pixels.
     * @param {number} [options.thickness] - The thickness of the coordinate grid lines.
     * @param {boolean} [options.enabled] - If true, the coordinate grid is enabled; otherwise, it is disabled.
     *
     * @example
     * // Set the coordinate grid color to red
     * aladin.setCooGrid({ color: 'red' });
     *
     * // Enable the coordinate grid
     * aladin.setCooGrid({ enabled: true });
     */
    Aladin.prototype.setCooGrid = function (options) {
        if (options.color) {
            // 1. the user has maybe given some
            options.color = new Color(options.color);
            // 3. convert from 0-255 to 0-1
            options.color.r /= 255;
            options.color.g /= 255;
            options.color.b /= 255;
        }

        this.view.setGridOptions(options);
    };

    Aladin.prototype.getGridOptions = function () {
        return this.view.getGridOptions();
    };

    Aladin.prototype.showCooGrid = function () {
        this.setCooGrid({ enabled: true });
    };

    Aladin.prototype.hideCooGrid = function () {
        this.setCooGrid({ enabled: false });
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
    Aladin.prototype.exportAsPNG = function (downloadFile = false) {
        (async () => {

            const url = await this.getViewDataURL();

            if (downloadFile) {
                Utils.download(url, "screenshot");
            } else {
                // open a new window
                var w = window.open();
                w.document.write(
                    '<img src="' + url + '" width="' + this.view.width + 'px">'
                );
                w.document.title = "Aladin Lite snapshot";
            }
        })();
    };

    /**
     * Return the current view as a png data URL (base64-formatted string)
     *
     * @memberof Aladin
     *
     * @param {Object} [options] Object with attributs, options are:
     * @param {Object} [options.format] 'image/png' or 'image/jpeg'
     * @param {Object} options.width Width in pixels of the image to output
     * @param {Object} options.height Height in pixels of the image to output
     * @param {Object} [options.logo=true] Boolean to display the Aladin Lite logo
     * @returns {Promise<string>} The image as a png data URL
     */
    Aladin.prototype.getViewDataURL = async function (options) {
        var options = options || {};
        // support for old API signature
        if (typeof options !== "object") {
            var imgFormat = options;
            options = { format: imgFormat };
        }

        const canvasDataURL = await this.view.getCanvasDataURL(
            options.format,
            options.width,
            options.height,
            options.logo
        );

        return canvasDataURL;
    };

    /**
     * Return the current view as a png ArrayBuffer
     *
     * @memberof Aladin
     *
     * @param {boolean} withLogo Display or not the Aladin Lite logo
     * @returns {Promise<ArrayBuffer>} The image as a png ArrayBuffer
     */
    Aladin.prototype.getViewArrayBuffer = async function (withLogo) {
        return await this.view.getCanvasArrayBuffer("image/png", null, null, withLogo);
    }

    /**
     * Return the current view as a png Blob
     *
     * @memberof Aladin
     *
     * @param {string} dataType The type of data to return. Can be 'url', 'arraybuffer' or 'blob'
     * @param {string} [imgType='image/png'] The type of image to return. Can be 'image/png', 'image/jpeg' or 'image/webp'
     * @param {boolean} [withLogo=true] Display or not the Aladin Lite logo
     * @returns {Promise<any>}
     */
    Aladin.prototype.getViewData = async function (dataType, imgType="image/png", withLogo=true){
        switch (dataType) {
            case "url":
                return await this.view.getCanvasDataURL(imgType, null, null, withLogo);
            case "arraybuffer":
                return await this.view.getCanvasArrayBuffer(imgType, null, null, withLogo);
            case "blob":
                return await this.view.getCanvasBlob(imgType, null, null, withLogo);
            default:
                throw new Error("Unknown data type: " + dataType);
        }
    }

    /**
     * Return the current view WCS as a key-value dictionary
     * Can be useful in coordination with getViewDataURL
     *
     * @memberof Aladin
     * @returns {Object} - A JS object describing the WCS of the view.
     */
    Aladin.prototype.getViewWCS = function () {
        // get general view properties
        const center = this.wasm.getCenter();
        const fov = this.getFov();
        const width = this.view.width;
        const height = this.view.height;

        // get values common for all
        let cdelt1 = -fov[0] / width;
        const cdelt2 = fov[1] / height;
        const projName = this.getProjectionName();

        if (projName == "FEYE")
            return "Fish eye projection is not supported by WCS standards.";

        // reversed longitude case
        if (this.getBaseImageLayer().longitudeReversed) {
            cdelt1 = -cdelt1;
        }

        // solar system object dict from planetary fits standard
        // https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2018EA000388
        const solarSystemObjects = {
            earth: "EA",
            moon: "SE",
            mercury: "ME",
            venus: "VE",
            mars: "MA",
            jupiter: "JU",
            saturn: "SA",
            uranus: "UR",
            neptune: "NE",
            // satellites other than the Moon
            satellite: "ST", // not findable in the hips properties?
        };

        // we define a generic LON LAT keyword for unknown body types
        let cooType1 = "LON--";
        let cooType2 = "LAT--";

        // just in case it would be equatorial
        let radesys;

        if (this.getBaseImageLayer().isPlanetaryBody()) {
            const body = this.getBaseImageLayer().hipsBody;
            if (body in solarSystemObjects) {
                cooType1 = `${solarSystemObjects[body]}LN-`;
                cooType2 = `${solarSystemObjects[body]}LT-`;
            }
        } else {
            switch (this.getFrame()) {
                case "ICRS":
                case "ICRSd":
                    cooType1 = "RA---";
                    cooType2 = "DEC--";
                    radesys = "ICRS    ";
                    break;
                case "GAL":
                    cooType1 = "GLON-";
                    cooType2 = "GLAT-";
            }
        }

        const WCS = {
            NAXIS: 2,
            NAXIS1: width,
            NAXIS2: height,
            CRPIX1: width / 2 + 0.5,
            CRPIX2: height / 2 + 0.5,
            CRVAL1: center[0],
            CRVAL2: center[1],
            CTYPE1: cooType1 + projName,
            CTYPE2: cooType2 + projName,
            CUNIT1: "deg     ",
            CUNIT2: "deg     ",
            CDELT1: cdelt1,
            CDELT2: cdelt2,
        };

        // handle the case of equatorial coordinates that need
        // the radecsys keyword
        if (radesys == "ICRS    ") WCS.RADESYS = radesys;

        const isProjZenithal = ['TAN', 'SIN', 'STG', 'ZEA'].some((p) => p === projName)
        if (isProjZenithal) {
            // zenithal projections
            // express the 3rd euler angle for zenithal projection
            let thirdEulerAngle = this.getViewCenter2NorthPoleAngle();
            WCS.LONPOLE = 180 - thirdEulerAngle
        } else {
            // cylindrical or pseudo-cylindrical projections
            if (WCS.CRVAL2 === 0) {
                // ref point on the equator not handled (yet)
                console.warn('TODO: 3rd euler rotation is not handled for ref point located at delta_0 = 0')
            } else {
                // ref point not on the equator
                const npLonlat = this.view.wasm.getNorthPoleCelestialPosition();
                let dLon = WCS.CRVAL1 - npLonlat[0];

                // dlon angle must lie between -PI and PI
                // For dlon angle between -PI;-PI/2 or PI/2;PI one must invert LATPOLE
                if (this.getViewCenter2NorthPoleAngle() < -90 || this.getViewCenter2NorthPoleAngle() > 90) {
                    // so that the south pole becomes upward to the ref point
                    WCS.LATPOLE = -90
                }

                const toRad = Math.PI / 180
                const toDeg = 1.0 / toRad;

                // Reverse the Eq 9 from the WCS II paper from Mark Calabretta to obtain LONPOLE
                // function of CRVAL2 and native coordinates of the fiducial ref point, i.e. (phi_0, theta_0) = (0, 0)
                // for cylindrical projections 
                WCS.LONPOLE = Math.asin(Math.sin(dLon * toRad) * Math.cos(WCS.CRVAL2 * toRad)) * toDeg;

                if (WCS.CRVAL2 < 0) {
                    // ref point is located in the south hemisphere
                    WCS.LONPOLE = -180 - WCS.LONPOLE;
                }
            }
        }

        return WCS;
    };

    /**
     * Restrict the FoV range between a min and a max value
     *
     * @memberof Aladin
     * @param {number} minFoV - in degrees when zoom in at max. If undefined, the zooming in is not limited
     * @param {number} maxFoV - in degrees when zoom out at max. If undefined, the zooming out is not limited
     *
     * @example
     * let aladin = A.aladin('#aladin-lite-div');
     * aladin.setFoVRange(30, 60);
     */
    Aladin.prototype.setFoVRange = function (minFoV, maxFoV) {
        this.view.setFoVRange(minFoV, maxFoV);
    };

    Aladin.prototype.setFOVRange = Aladin.prototype.setFoVRange;

    /**
     * Transform pixel coordinates to world coordinates.
     *
     * The origin (0,0) of pixel coordinates is at the top-left corner of the Aladin Lite view.
     *
     * @memberof Aladin
     * @param {number} x - The x-coordinate in pixel coordinates.
     * @param {number} y - The y-coordinate in pixel coordinates.
     * @param {CooFrame} [frame] - The frame in which we want to retrieve the coordinates.
     * If not given, the frame chosen is the one from the view
     *
     * @returns {number[]} - An array representing the [Right Ascension, Declination] coordinates in degrees in the `frame`.
     * If not specified, returns the coo in the frame of the current view.
     *
     * @throws {Error} Throws an error if an issue occurs during the transformation.
     */
    Aladin.prototype.pix2world = function (x, y, frame) {
        if (frame) {
            frame = CooFrameEnum.fromString(frame, CooFrameEnum.J2000);
            if (frame.label == CooFrameEnum.SYSTEMS.GAL) {
                frame = Aladin.wasmLibs.core.CooSystem.GAL;
            }
            else {
                frame = Aladin.wasmLibs.core.CooSystem.ICRS;
            }
        }

        let lonlat = this.view.wasm.pix2world(x, y, frame);

        let [lon, lat] = lonlat;

        if (lon < 0) {
            return [lon + 360.0, lat];
        }

        return [lon, lat];
    };

    /**
     * Transform world coordinates to pixel coordinates in the view.
     *
     * @memberof Aladin
     * @param {number} lon - Londitude coordinate in degrees.
     * @param {number} lat - Latitude coordinate in degrees.
     * @param {CooFrame} [frame] - If not specified, the frame used is ICRS

     * @returns {number[]} - An array representing the [x, y] coordinates in pixel coordinates in the view.
     *
     * @throws {Error} Throws an error if an issue occurs during the transformation.
     */
    Aladin.prototype.world2pix = function (lon, lat, frame) {
        if (frame) {
            if (frame instanceof string) {
                frame = CooFrameEnum.fromString(frame, CooFrameEnum.J2000);
            }
    
            if (frame.label == CooFrameEnum.SYSTEMS.GAL) {
                frame = Aladin.wasmLibs.core.CooSystem.GAL;
            }
            else {
                frame = Aladin.wasmLibs.core.CooSystem.ICRS;
            }
        }

        return this.view.wasm.world2pix(lon, lat, frame);
    };

    /**
     * Get the angular distance in degrees between two locations
     *
     * @memberof Aladin
     * @param {number} x1 - The x-coordinate of the first pixel coordinates.
     * @param {number} y1 - The y-coordinate of the first pixel coordinates.
     * @param {number} x2 - The x-coordinate of the second pixel coordinates.
     * @param {number} y2 - The y-coordinate of the second pixel coordinates.
     * @param {CooFrame} [frame] - The frame in which we want to retrieve the coordinates.
     * If not given, the frame chosen is the one from the view
     *
     * @returns {number} - The angular distance between the two pixel coordinates in degrees
     *
     * @throws {Error} Throws an error if an issue occurs during the transformation.
     */
    Aladin.prototype.angularDist = function (x1, y1, x2, y2, frame) {
        const [ra1, dec1] = this.pix2world(x1, y1, frame);
        const [ra2, dec2] = this.pix2world(x2, y2, frame);

        return this.wasm.angularDist(ra1, dec1, ra2, dec2);
    };

    /**
     * Gets a set of points along the current Field of View (FoV) corners.
     *
     * @memberof Aladin
     * @param {number} [nbSteps=1] - The number of points to return along each side (the total number of points returned is 4 * nbSteps).
     * @param {CooFrame} [frame] - The frame in which the coo will be given. Default to the view frame.
     *
     * @returns {number[][]} - A set of positions along the current FoV with the following format: [[ra1, dec1], [ra2, dec2], ..., [ra_n, dec_n]].
     *                         The positions will be given in degrees
     *
     * @throws {Error} Throws an error if an issue occurs during the transformation.
     *
     */
    Aladin.prototype.getFoVCorners = function (nbSteps, frame) {
        // default value: 1
        if (!nbSteps || nbSteps < 1) {
            nbSteps = 1;
        }

        var points = [];
        var x1, y1, x2, y2;
        for (var k = 0; k < 4; k++) {
            x1 = k == 0 || k == 3 ? 0 : this.view.width - 1;
            y1 = k < 2 ? 0 : this.view.height - 1;
            x2 = k < 2 ? this.view.width - 1 : 0;
            y2 = k == 1 || k == 2 ? this.view.height - 1 : 0;

            for (var step = 0; step < nbSteps; step++) {
                let radec = this.pix2world(
                    x1 + (step / nbSteps) * (x2 - x1),
                    y1 + (step / nbSteps) * (y2 - y1),
                    frame
                );
                points.push(radec);
            }
        }

        return points;
    };

    /**
     * Gets the current Field of View (FoV) size in degrees as a 2-element array.
     *
     * @memberof Aladin
     * @returns {number[]} - A 2-element array representing the current FoV size in degrees. The first element is the FoV width,
     *                       and the second element is the FoV height.
     */
    Aladin.prototype.getFov = function () {
        // can go up to 1000 deg
        var fovX = this.view.fov;
        var s = this.getSize();

        // constrain to the projection definition domain
        fovX = Math.min(fovX, this.view.projection.fov);
        var fovY = (s[1] / s[0]) * fovX;

        fovY = Math.min(fovY, 180);
        // TODO : take into account AITOFF projection where fov can be larger than 180

        return [fovX, fovY];
    };

    Aladin.prototype.getFoV = Aladin.prototype.getFov;

    /**
     * Returns the size in pixels for the Aladin view
     *
     * @memberof Aladin
     * @returns {number[]} - A 2-element array representing the current Aladin view size in pixels. The first element is the width,
     *                       and the second element is the height.
     */
    Aladin.prototype.getSize = function () {
        return [this.view.width, this.view.height];
    };

    /**
     * Returns the HTML div element
     *
     * @memberof Aladin
     * @return {HTMLElement} - The aladin lite div HTML element
     */
    Aladin.prototype.getParentDiv = function () {
        return this.aladinDiv;
    };

    // @API
    /*
     * return a Box GUI element to insert content
     */
    /*Aladin.prototype.box = function (options) {
        var box = new Box(options);
        box.$parentDiv.appendTo(this.aladinDiv);

        return box;
    };*/

    // @API
    /*
     * show popup at ra, dec position with given title and content
     *
     * If circleRadius, the corresponding circle will also be plotted
     */
    Aladin.prototype.showPopup = function (
        ra,
        dec,
        title,
        content,
        circleRadius
    ) {
        this.view.catalogForPopup.removeAll();
        this.view.overlayForPopup.removeAll();

        let marker;
        if (circleRadius !== undefined) {
            this.view.overlayForPopup.add(
                A.circle(ra, dec, circleRadius, {
                    fillColor: "rgba(255, 0, 0, 0.2)",
                })
            );
            marker = A.marker(ra, dec, {
                popupTitle: title,
                popupDesc: content,
                useMarkerDefaultIcon: true,
            });
        } else {
            marker = A.marker(ra, dec, {
                popupTitle: title,
                popupDesc: content,
                useMarkerDefaultIcon: false,
            });
        }

        this.view.catalogForPopup.addSources(marker);

        this.view.overlayForPopup.show();
        this.view.catalogForPopup.show();

        this.popup.setTitle(title);
        this.popup.setText(content);

        this.popup.setSource(marker);
        this.popup.show();
    };

    // @API
    /*
     * hide popup
     */
    Aladin.prototype.hidePopup = function () {
        this.popup.hide();
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

        return (
            Aladin.URL_PREVIEWER +
            "?target=" +
            encodeURIComponent(coo.format("s")) +
            "&fov=" +
            this.getFov()[0].toFixed(2) +
            "&survey=" +
            encodeURIComponent(
                this.getBaseImageLayer().id || this.getBaseImageLayer().rootUrl
            )
        );
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

        var survey = this.getBaseImageLayer().url;
        var fov = this.getFov()[0];
        let s = "";
        const NL = "\n";
        s +=
            '<div id="aladin-lite-div" style="width:400px;height:400px;"></div>' +
            NL;
        s +=
            '<script src="https://aladin.cds.unistra.fr/AladinLite/api/v3/latest/aladin.js" charset="utf-8"></script>' +
            NL;
        s += "<script>" + NL;
        s +=
            "let aladin;" +
            NL +
            "A.init.then(() => {" +
            NL +
            "   aladin = A.aladin('#aladin-lite-div', {survey: '" +
            survey +
            "', fov: " +
            fov.toFixed(2) +
            ', target: "' +
            coo.format("s") +
            '"});' +
            NL +
            "});" +
            NL;
        s += "</script>";

        return s;
    };

    /**
     * Display a FITS image in the Aladin Lite.
     *
     * @memberof Aladin
     * @param {string} url - The URL of the FITS image.
     * @param {ImageOptions} [options] - Options to customize the display
     * @param {Function} [successCallback=<center the view on the FITS file>] - The callback function to be executed on a successful display.
     *      The callback gives the ra, dec, and fov of the image; By default, it centers the view on the FITS file loaded.
     * @param {Function} [errorCallback] - The callback function to be executed if an error occurs during display.
     * @param {string} [layer="base"] - The name of the layer. If not specified, it will be replace the base layer.
     *
     * @example
aladin.displayFITS(
    'https://fits.gsfc.nasa.gov/samples/FOCx38i0101t_c0f.fits', // url of the fits file
    {
        minCut: 5000,
        maxCut: 17000,
        colormap: 'viridis'
    },
    (ra, dec, fov, image) => {
        // ra, dec and fov are centered around the fits image
        image.setColormap("magma", {stretch: "asinh"});

        aladin.gotoRaDec(ra, dec);
        aladin.setFoV(fov);
    },
);
     */
    Aladin.prototype.displayFITS = function (
        url,
        options,
        successCallback,
        errorCallback,
        layer = "base"
    ) {
        successCallback =
            successCallback ||
            ((ra, dec, fov, _) => {
                this.gotoRaDec(ra, dec);
                this.setFoV(fov);
            });
        const image = this.createImageFITS(
            url,
            options,
            successCallback,
            errorCallback
        );
        return this.setOverlayImageLayer(image, layer);
    };

    /**
     * Display a JPEG image in the Aladin Lite view.
     *
     * @memberof Aladin
     * @param {string} url - The URL of the JPEG image.
     * @param {Object} [options] - Options to customize the display. Can include the following properties:
     * @param {string} [options.label="JPG/PNG image"]  - A label for the displayed image.
     * @param {number} [options.order] - The desired HEALPix order format.
     * @param {boolean} [options.nocache] - True if you want to disable the cache
     * @param {number} [options.transparency=1.0] - Opacity of the image rendered in aladin lite. Between 0 and 1.
     * @param {Function} [successCallback] - The callback function to be executed on a successful display.
     *      The callback gives the ra, dec, and fov of the image;
     * @param {Function} [errorCallback] - The callback function to be executed if an error occurs during display.
     * @param {string} [layer="overlay"] - The name of the layer. If not specified, it will add a new overlay layer on top of the base.
     *
     * @example
     * aladin.displayJPG(
     *  // the JPG to transform to HiPS
     *   'https://noirlab.edu/public/media/archives/images/large/noirlab1912a.jpg',
     *   {
     *       transparency: 0.6,
     *       label: 'NOIRLab image'
     *   },
     *   (ra, dec, fov) => {
     *      // your code here
     *   })
     *);
     */
    Aladin.prototype.displayJPG = function (
        url,
        options,
        successCallback,
        errorCallback,
        layer = "overlay"
    ) {
        options = options || {};
        options.color = true;
        options.label = options.label || "JPG/PNG image";
        options.outputFormat = "png";

        options = options || {};

        var data = { url };
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

        const request = (url, params = {}, method = "GET") => {
            let options = {
                method,
            };
            if ("GET" === method) {
                url += "?" + new URLSearchParams(params).toString();
            } else {
                options.body = JSON.stringify(params);
            }

            return fetch(url, options).then((response) => response.json());
        };
        const get = (url, params) => request(url, params, "GET");

        get("https://alasky.unistra.fr/cgi/fits2HiPS", data).then(
            async (response) => {
                if (response.status != "success") {
                    console.error("An error occured: " + response.message);
                    if (errorCallback) {
                        errorCallback(response.message);
                    }
                    return;
                }
                var label = options.label;
                var meta = response.data.meta;

                const survey = self.createImageSurvey(
                    response.data.url,
                    label,
                    response.data.url
                );
                self.setOverlayImageLayer(survey, layer);

                var transparency = (options && options.transparency) || 1.0;
                survey.setOpacity(transparency);

                var executeDefaultSuccessAction = true;
                if (successCallback) {
                    executeDefaultSuccessAction = successCallback(
                        meta.ra,
                        meta.dec,
                        meta.fov
                    );
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
            }
        );
    };

    Aladin.prototype.displayPNG = Aladin.prototype.displayJPG;

    /**
     * Add a custom colormap from a list of colors
     *
     * @memberof Aladin
     * 
     * @returns - The list of all the colormap labels
     */
    Aladin.prototype.getListOfColormaps = function() {
        return this.view.wasm.getAvailableColormapList();
    };

    /**
     * Add a custom colormap from a list of colors
     *
     * @memberof Aladin
     * @param {string} label - The label of the colormap
     * @param {string[]} colors - A list string colors
     * 
     * @example
     * 
     * aladin.addColormap('mycmap', ["lightblue", "red", "violet", "#ff00aaff"])
     */
    Aladin.prototype.addColormap = function(label, colors) {
        colors = colors.map((label) => {
            return new Color(label).toHex() + 'ff';
        });

        this.view.wasm.createCustomColormap(label, colors);

        ALEvent.UPDATE_CMAP_LIST.dispatchedTo(this.aladinDiv, {
            cmaps: this.getListOfColormaps()
        });
    };

    /*
    Aladin.prototype.setReduceDeformations = function (reduce) {
        this.reduceDeformations = reduce;
        this.view.requestRedraw();
    }
    */

    return Aladin;
})();
