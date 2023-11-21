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
import { HiPSDefinition } from "./HiPSDefinition.js";
import { ColorCfg } from './ColorCfg.js';
import { ObsCore } from "./vo/ObsCore.js";
import { Aladin } from "./Aladin.js";
// Wasm top level import
import init, * as module from './../core/pkg';

import $ from 'jquery';

// Import aladin css inside the project
import './../css/aladin.css';

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
A.catalog = function (options) {
    return new Catalog(options);
};

// @API
A.catalogHiPS = function (rootURL, options) {
    return new ProgressiveCat(rootURL, null, null, options);
};

// API
A.footprintsFromSTCS = function (stcs, options) {
    var footprints = Overlay.parseSTCS(stcs, options);

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


A.catalogFromURL = function (url, options, successCallback, errorCallback, useProxy) {
    var catalog = A.catalog(options);

    const processVOTable = function (sources, footprints, fields) {
        catalog.setFields(fields);

        if (catalog.isObsCore()) { 
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
        ObsCore.handleActions(catalog);
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