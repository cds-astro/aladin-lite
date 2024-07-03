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
 * File ImageHiPS
 *
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ALEvent } from "./events/ALEvent.js";

export let HiPSCache = (function () {
    function HiPSCache() {}

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.append = function (key, image) {
        HiPSCache.cache[key] = image;

        ALEvent.HIPS_CACHE_UPDATED.dispatchedTo(document.body);
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.delete = function (key) {
        delete HiPSCache.cache[key];

        ALEvent.HIPS_CACHE_UPDATED.dispatchedTo(document.body);
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.get = function (key) {
        return HiPSCache.cache[key];
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.contains = function (key) {
        return HiPSCache.cache[key] !== undefined && HiPSCache.cache[key] !== null;
    };

    // A cache storing directly surveys important information to not query for the properties each time
    HiPSCache.cache = {};

    HiPSCache.DEFAULT_HIPS_LIST = [
        {
            creatorDid: "ivo://CDS/P/DSS2/color",
            name: "DSS colored",
            id: "CDS/P/DSS2/color",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "ICRS",
        },
        {
            creatorDid: "ivo://CDS/P/2MASS/color",
            name: "2MASS colored",
            id: "CDS/P/2MASS/color",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "ICRS",
        },
        {
            creatorDid: "ivo://CDS/P/DSS2/red",
            name: "DSS2 Red (F+R)",
            id: "CDS/P/DSS2/red",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "fits",
            cooFrame: "ICRS",
            numBitsPerPixel: 16,
            // options
            minCut: 1000.0,
            maxCut: 10000.0,
            colormap: "magma",
            stretch: "Linear",
            imgFormat: "fits",
        },
        {
            creatorDid: "ivo://CDS/P/DM/I/350/gaiaedr3",
            name: "Density map for Gaia EDR3 (I/350/gaiaedr3)",
            id: "CDS/P/DM/I/350/gaiaedr3",
            maxOrder: 7,
            tileSize: 512,
            numBitsPerPixel: -32,
            cooFrame: "ICRS",
            minCut: 0,
            maxCut: 12000,
            stretch: "asinh",
            colormap: "rdylbu",
            imgFormat: "fits",
        },
        {
            creatorDid: "ivo://CDS/P/PanSTARRS/DR1/g",
            name: "PanSTARRS DR1 g",
            id: "CDS/P/PanSTARRS/DR1/g",
            maxOrder: 11,
            tileSize: 512,
            imgFormat: "fits",
            cooFrame: "ICRS",
            numBitsPerPixel: -32,
            // options
            minCut: -34,
            maxCut: 7000,
            stretch: "asinh",
            colormap: "redtemperature",
        },
        {
            creatorDid: "ivo://CDS/P/PanSTARRS/DR1/color-z-zg-g",
            name: "PanSTARRS DR1 color",
            id: "CDS/P/PanSTARRS/DR1/color-z-zg-g",
            maxOrder: 11,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "ICRS",
        },
        {
            creatorDid: "ivo://CDS/P/DECaPS/DR2/color",
            name: "DECaPS DR2 color",
            id: "CDS/P/DECaPS/DR2/color",
            maxOrder: 11,
            cooFrame: "equatorial",
            tileSize: 512,
            imgFormat: "png",
        },
        {
            creatorDid: "ivo://CDS/P/Fermi/color",
            name: "Fermi color",
            id: "CDS/P/Fermi/color",
            maxOrder: 3,
            imgFormat: "jpeg",
            tileSize: 512,
            cooFrame: "equatorial",
        },
        {
            creatorDid: "ivo://CDS/P/GALEXGR6_7/NUV",
            id: "P/GALEXGR6_7/NUV",
            name: "GALEXGR6_7 NUV",
            maxOrder: 8,
            imgFormat: "png",
            tileSize: 512,
            cooFrame: "equatorial",
        },
        {
            creatorDid: "ivo://CDS/P/IRIS/color",
            id: "CDS/P/IRIS/color",
            name: "IRIS colored",
            maxOrder: 3,
            tileSize: 256,
            imgFormat: "jpeg",
            cooFrame: "galactic",
        },
        {
            creatorDid: "ivo://CDS/P/Mellinger/color",
            id: "CDS/P/Mellinger/color",
            name: "Mellinger colored",
            maxOrder: 4,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "galactic",
        },
        {
            creatorDid: "ivo://CDS/P/SDSS9/color",
            id: "CDS/P/SDSS9/color",
            name: "SDSS9 colored",
            maxOrder: 10,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
        },
        {
            creatorDid: "ivo://CDS/P/SPITZER/color",
            id: "CDS/P/SPITZER/color",
            name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "galactic",
        },
        {
            creatorDid: "ivo://CDS/P/allWISE/color",
            id: "CDS/P/allWISE/color",
            name: "AllWISE color",
            maxOrder: 8,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
        },
        {
            creatorDid: "ivo://CDS/P/SDSS9/g",
            id: "CDS/P/SDSS9/g",
            name: "SDSS9 band-g",
            maxOrder: 10,
            tileSize: 512,
            numBitsPerPixel: 16,
            imgFormat: "fits",
            cooFrame: "equatorial",
            minCut: 0,
            maxCut: 1.8,
            stretch: "linear",
            colormap: "redtemperature",
        },
        {
            id: "CDS/P/Finkbeiner",
            name: "Halpha",
            maxOrder: 3,
            minCut: -10,
            maxCut: 800,
            colormap: "rdbu",
            imgFormat: "fits",
        },
        {
            id: "CDS/P/VTSS/Ha",
            name: "VTSS-Ha",
            maxOrder: 3,
            minCut: -10.0,
            maxCut: 100.0,
            colormap: "grayscale",
            imgFormat: "fits",
        },
        {
            id: "xcatdb/P/XMM/PN/color",
            name: "XMM PN colored",
            maxOrder: 7,
        },
        {
            id: "CDS/P/allWISE/color",
            name: "AllWISE color",
            maxOrder: 8,
        },
        /*{
                id: "CDS/P/GLIMPSE360",
                name: "GLIMPSE360",
                // This domain is not giving the CORS headers
                // We need to query by with a proxy equipped with CORS header.
                //url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=https://www.spitzer.caltech.edu/glimpse360/aladin/data",
                maxOrder: 9,
                imgFormat: "jpeg",
                minOrder: 3,
            }*/
    ];

    return HiPSCache;
})();
