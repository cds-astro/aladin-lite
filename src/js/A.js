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

import { MOC } from "./MOC.js";
import { Overlay } from "./Overlay.js";
import { Circle } from "./Circle.js";
import { Ellipse } from "./Ellipse.js";
import { Polyline } from "./Polyline.js";
import { Catalog } from "./Catalog.js";
import { ProgressiveCat } from "./ProgressiveCat.js";
import { Source } from "./Source.js";
import { Coo } from "./libs/astro/coo.js";
import { URLBuilder } from "./URLBuilder.js";
import { ColorCfg } from './ColorCfg.js';
import { Footprint } from './Footprint.js';
import { Toolbar } from "./gui/widgets/Toolbar.js";
import { Aladin } from "./Aladin.js";
// Wasm top level import
import init, * as module from './../core/pkg';

// Import aladin css inside the project
import './../css/aladin.css';
import { ActionButton } from "./gui/Widgets/ActionButton.js";

///////////////////////////////
/////// Aladin Lite API ///////
///////////////////////////////

/**
 * @namespace A
 * @description Aladin Lite API namespace for creating celestial objects.
 * @example
 * // Usage example:
 * import { A } from 'aladin-lite';
 *
 * const aladin = new A.aladin("#aladin-lite-div", { survey: 'your survey url', fov: 180, projection: 'SIN' });
 */
let A = {};

//// New API ////
// For developers using Aladin lite: all objects should be created through the API,
// rather than creating directly the corresponding JS objects
// This facade allows for more flexibility as objects can be updated/renamed harmlessly

/**
 * @typedef {Object} AladinOptions
 * @description Options for configuring the Aladin Lite instance.
 *
 * @property {string} [survey="https://alaskybis.unistra.fr/DSS/DSSColor"] URL or ID of the survey to use
 * @property {string[]} [surveyUrl=["https://alaskybis.unistra.fr/DSS/DSSColor", "https://alasky.unistra.fr/DSS/DSSColor"]]
 *   Array of URLs for the survey images. This replaces the survey parameter.
 * @property {string} [target="0 +0"] - Target coordinates for the initial view.
 * @property {string} [cooFrame="J2000"] - Coordinate frame.
 * @property {number} [fov=60] - Field of view in degrees.
 * @property {string} [backgroundColor="rgb(60, 60, 60)"] - Background color in RGB format.
 *
 * @property {boolean} [showZoomControl=true] - Whether to show the zoom control toolbar.
 * @property {boolean} [showLayersControl=true] - Whether to show the layers control toolbar.
 * @property {boolean} [showFullscreenControl=true] - Whether to show the fullscreen control toolbar.
 * @property {boolean} [showGotoControl=true] - Whether to show the goto control toolbar.
 * @property {boolean} [showSimbadPointerControl=false] - Whether to show the Simbad pointer control toolbar.
 * @property {boolean} [showCooGridControl=false] - Whether to show the coordinate grid control toolbar.
 * @property {boolean} [showSettingsControl=true] - Whether to show the settings control toolbar.
 *
 * @property {boolean} [showShareControl=false] - Whether to show the share control toolbar.
 *
 * @property {boolean} [showFrame=true] - Whether to show the viewport frame.
 * @property {boolean} [showFov=true] - Whether to show the field of view indicator.
 * @property {boolean} [showCooLocation=true] - Whether to show the coordinate location indicator.
 * @property {boolean} [showProjectionControl=true] - Whether to show the projection control toolbar.
 *
 * @property {boolean} [showContextMenu=false] - Whether to show the context menu.
 * @property {boolean} [showReticle=true] - Whether to show the reticle.
 * @property {boolean} [showCatalog=true] - Whether to show the catalog.
 *
 * @property {boolean} [fullScreen=false] - Whether to start in full-screen mode.
 * @property {string} [reticleColor="rgb(178, 50, 178)"] - Color of the reticle in RGB format.
 * @property {number} [reticleSize=22] - Size of the reticle.
 * @property {string} [gridColor="rgb(0, 255, 0)"] - Color of the grid in RGB format.
 * @property {number} [gridOpacity=0.5] - Opacity of the grid (0 to 1).
 * @property {string} [projection="SIN"] - Projection type.
 * @property {boolean} [log=true] - Whether to log events.
 * @property {boolean} [samp=false] - Whether to enable SAMP (Simple Application Messaging Protocol).
 * @property {boolean} [realFullscreen=false] - Whether to use real fullscreen mode.
 * @property {boolean} [pixelateCanvas=true] - Whether to pixelate the canvas.
 */

/**
 * Creates an Aladin Lite instance within the specified HTML element.
 *
 * @function
 * @name A.aladin
 * @memberof A
 * @param {string} divSelector - The ID selector for the HTML element.
 * @param {AladinOptions} [options] - Options for configuring the Aladin Lite instance.
 * @returns {Aladin} An instance of the Aladin Lite library.
 * @example
 * const aladinInstance = A.aladin('#aladin-container', options);
 */
A.aladin = function (divSelector, options) {
    return new Aladin(document.querySelector(divSelector), options);
};

/**
 * Creates a celestial source object with the given coordinates.
 *
 * @function
 * @name A.source
 * @memberof A
 * @param {number} ra - Right Ascension (RA) coordinate in degrees.
 * @param {number} dec - Declination (Dec) coordinate in degrees.
 * @param {*} [data] - Additional data associated with the source.
 * @param {SourceOptions} [options] - Options for configuring the source object.
 * @returns {Source} A celestial source object.
 * @example
 * const sourceObj = A.source(180.0, 30.0, data, options);
 */
A.source = function (ra, dec, data, options) {
    return new Source(ra, dec, data, options);
};

/**
 * Creates a marker at the specified celestial coordinates.
 *
 * @function
 * @name A.marker
 * @memberof A
 * @param {number} ra - Right Ascension (RA) coordinate in degrees.
 * @param {number} dec - Declination (Dec) coordinate in degrees.
 * @param {MarkerOptions} [options] - Options for configuring the marker.
 * @param {*} [data] - Additional data associated with the marker.
 * @returns {Source} A marker source object.
 * @example
 * const markerObj = A.marker(180.0, 30.0, data, options);
 */
A.marker = function (ra, dec, options, data) {
    options = options || {};
    options['marker'] = true;
    return A.source(ra, dec, data, options);
};

// @API
A.polygon = function (raDecArray, options) {
    const numVertices = raDecArray.length;

    if (numVertices < 3) {
        // Cannot define a polygon from that
        throw 'Cannot define a polygon from less than 3 vertices';
    }

    const lastVertexIdx = numVertices - 1;

    // User gave a closed polygon, so we remove the last vertex
    if (raDecArray[0][0] == raDecArray[lastVertexIdx][0] && raDecArray[0][1] == raDecArray[lastVertexIdx][1]) {
        raDecArray.pop()
        // but declare the polygon as closed
    }

    options = options || {};
    options.closed = true;

    return new Polyline(raDecArray, options);
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
A.catalogHiPS = function (rootURL, options) {
    return new ProgressiveCat(rootURL, null, null, options);
};

// API
A.coo = function (longitude, latitude, prec) {
    return new Coo(longitude, latitude, prec);
};

// API
A.footprint = function(shapes, source) {
    return new Footprint(shapes, source);
};

// API
A.footprintsFromSTCS = function (stcs, options) {
    var footprints = Overlay.parseSTCS(stcs, options);

    return footprints;
}

// API
A.MOCFromURL = function (url, options, successCallback) {
    var moc = new MOC(options);
    moc.parse(url, successCallback);

    return moc;
};

// API
A.MOCFromJSON = function (jsonMOC, options, successCallback, errorCallback) {
    var moc = new MOC(options);
    moc.parse(jsonMOC, successCallback, errorCallback);

    return moc;
};

// API
A.MOCFromCircle = function (circle, options, successCallback, errorCallback) {
    var moc = new MOC(options);
    moc.parse(circle, successCallback, errorCallback);

    return moc;
};

A.MOCFromPolygon= function (polygon, options, successCallback, errorCallback) {
    var moc = new MOC(options);
    moc.parse(polygon, successCallback, errorCallback);

    return moc;
};

/**
 * Represents options for configuring a catalog.
 *
 * @typedef {Object} CatalogOptions
 * @property {string} url - The URL of the catalog.
 * @property {string} [name="catalog"] - The name of the catalog.
 * @property {string} [color] - The color associated with the catalog.
 * @property {number} [sourceSize=8] - The size of the sources in the catalog.
 * @property {number} [markerSize=12] - The size of the markers associated with sources.
 * @property {string} [shape="square"] - The shape of the sources (e.g., "square", "circle", "rhomb", "triangle", "cross").
 * @property {number} [limit] - The maximum number of sources to display.
 * @property {function} [onClick] - The callback function to execute on a source click.
 * @property {boolean} [readOnly=false] - Whether the catalog is read-only.
 * @property {string} [raField] - The ID or name of the field holding Right Ascension (RA).
 * @property {string} [decField] - The ID or name of the field holding Declination (dec).
 * @property {function} [filter] - The filtering function for sources.
 * @property {boolean} [displayLabel=false] - Whether to display labels for sources.
 * @property {string} [labelColor] - The color of the source labels.
 * @property {string} [labelFont="10px sans-serif"] - The font for the source labels.
 */

/**
 * Represents a catalog with configurable options for display and interaction.
 *
 * @function
 * @name A.catalog
 * @memberof A
 * @param {CatalogOptions} options - Configuration options for the catalog.
 * @returns {Catalog}
 */
A.catalog = function (options) {
    return new Catalog(options);
};

/**
 * Asynchronously creates a new catalog instance from the specified URL with additional options.
 *
 * @function
 * @memberof A
 * @param {string} url - The URL of the catalog.
 * @param {CatalogOptions} [options] - Additional configuration options for the catalog.
 * @param {function} [successCallback] - The callback function to execute on successful catalog creation.
 * @param {function} [errorCallback] - The callback function to execute on error during catalog creation.
 * @param {boolean} [useProxy=false] - Indicates whether to use a proxy for loading the catalog.
 * @returns {Catalog} A new instance of the Catalog class created from the specified URL.
 *
 * @example
 * // Create a catalog from a URL using the A.catalogFromURL method
 * const catalogURL = "https://example.com/catalog";
 * const catalogOptions = {
 *   name: "My Catalog",
 *   color: "#ff0000",
 *   sourceSize: 10,
 *   // ... other options
 * };
 *
 * const myCatalog = A.catalogFromURL(
 *   catalogURL,
 *   catalogOptions,
 *   (catalog) => {
 *     // Catalog successfully loaded
 *     aladin.addCatalog(catalog)
 *   },
 *   (error) => {
 *     // Error loading catalog
 *     console.error("Error loading catalog:", error);
 *   },
 * );
 */
A.catalogFromURL = function (url, options, successCallback, errorCallback, useProxy) {
    options.url = url;
    var catalog = A.catalog(options);

    const processVOTable = function (table) {
        let {sources, footprints, fields, type} = table;
        catalog.setFields(fields);

        if (catalog.type === 'ObsCore') {
            // The fields corresponds to obscore ones
            // Set the name of the catalog to be ObsCore:<catalog name>
            catalog.name = "ObsCore:" + url;
        }

        catalog.addFootprints(footprints)
        catalog.addSources(sources);

        if (successCallback) {
            successCallback(catalog);
        }

        // Even if the votable is not a proper ObsCore one, try to see if specific columns are given
        // e.g. access_format and access_url
        //ObsCore.handleActions(catalog);
    };

    if (useProxy !== undefined) {
        Catalog.parseVOTable(
            url,
            processVOTable,
            errorCallback,
            catalog.maxNbSources,
            useProxy,
            catalog.raField, catalog.decField
        );
    } else {
        Catalog.parseVOTable(
            url,
            processVOTable,
            () => {
                Catalog.parseVOTable(
                    url,
                    processVOTable,
                    errorCallback,
                    catalog.maxNbSources,
                    true,
                    catalog.raField, catalog.decField
                );
            },
            catalog.maxNbSources,
            false,
            catalog.raField, catalog.decField
        );
    }

    return catalog;
};

// API
// @param target: can be either a string representing a position or an object name, or can be an object with keys 'ra' and 'dec' (values being in decimal degrees)
A.catalogFromSimbad = function (target, radius, options, successCallback, errorCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'Simbad';
    }
    var url = URLBuilder.buildSimbadCSURL(target, radius);
    return A.catalogFromURL(url, options, successCallback, errorCallback, false);
};

// API
A.catalogFromNED = function (target, radius, options, successCallback, errorCallback) {
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

    return A.catalogFromURL(url, options, successCallback, errorCallback, true);
};

A.catalogFromSKAORucio = function (target, radiusDegrees, options, successCallback, errorCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'SKAO';
    }
    var url = URLBuilder.buildSKAORucioCSURL(target, radiusDegrees);

    return A.catalogFromURL(url, options, successCallback, errorCallback, true);
};

// API
A.catalogFromVizieR = function (vizCatId, target, radius, options, successCallback, errorCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'VizieR:' + vizCatId;
    }

    var url = URLBuilder.buildVizieRCSURL(vizCatId, target, radius, options);
    return A.catalogFromURL(url, options, successCallback, errorCallback, false);
};

// API
A.catalogFromVizieR = function (vizCatId, target, radius, options, successCallback, errorCallback) {
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'VizieR:' + vizCatId;
    }

    var url = URLBuilder.buildVizieRCSURL(vizCatId, target, radius, options);
    return A.catalogFromURL(url, options, successCallback, errorCallback, false);
};

// API
A.catalogFromSkyBot = function (ra, dec, radius, epoch, queryOptions, options, successCallback, errorCallback) {
    queryOptions = queryOptions || {};
    options = options || {};
    if (!('name' in options)) {
        options['name'] = 'SkyBot';
    }
    var url = URLBuilder.buildSkyBotCSURL(ra, dec, radius, epoch, queryOptions);
    return A.catalogFromURL(url, options, successCallback, errorCallback, false);
};

/// UI API
/*
{
    direction: 'vertical' | 'horizontal',
    cssStyle: {...}
    position: {
            top,
            left
        } \ {
            anchor: 'left top' |
                'left center' |
                'left bottom' |
                'right top' |
                'right center' |
                'right bottom' |
                'center top' |
                'center center' |
                'center bottom'
        }
    }
}
*/
A.toolbar = function(options) {
    return new Toolbar(options);
}

A.button = function(options) {
    return new ActionButton(options);
}

/*A.hipsDefinitionFromURL = function(url, successCallback) {
    HiPSDefinition.fromURL(url, successCallback);
};*/

A.getAvailableListOfColormaps = function() {
    return ColorCfg.COLORMAPS;
};

/**
 * Initializes the Aladin Lite library, checking for WebGL2 support.
 * This method must be called before instancing an Aladin Lite object.
 *
 * @function
 * @name A.init
 * @memberof A
 * @async
 *
 * @throws {string} Throws an error if WebGL2 is not supported by the browser.
 *
 * @returns {Promise<void>} A promise that resolves once the initialization is complete.
 *
 * @example
 * // Usage example:
 * A.init
 *   .then(async () => {
 *     const aladinInstance = A.aladin('div', requestedOptions);
 *     // Perform further actions with the Aladin Lite instance
 *   })
 *   .catch(error => {
 *     console.error('Error initializing Aladin Lite:', error);
 *   });
 */
A.init = (async () => {
    const isWebGL2Supported = document
        .createElement('canvas')
        .getContext('webgl2');

    await init();
    // Check for webgl2 support
    if (isWebGL2Supported) {
        Aladin.wasmLibs.core = module;
    } else {
        // WebGL1 not supported
        // According to caniuse, https://caniuse.com/webgl2, webgl2 is supported by 89% of users
        throw "WebGL2 not supported by your browser";
    }
})();

export default A;
