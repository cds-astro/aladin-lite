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

import { DataproductType } from "../core/pkg/core";

export let HiPSList = (function () {
    function HiPSList() {}

    HiPSList.DEFAULT = [
        {
            creatorDid: "ivo://CDS/P/DSS2/color",
            name: "DSS colored",
            id: "P/DSS2/color",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
            startUrl: "https://alasky.cds.unistra.fr/DSS/DSSColor",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://erosita/dr1/rate/rgb",
            id: "erosita/dr1/rate/rgb",
            name: "eROSITA-DE DR1 RGB (0.2-0.5, 0.5-1.0, 1.0-2.0 keV) Rate Image",
            maxOrder: 6,
            tileSize: 512,
            imgFormat: "png",
            cooFrame: "equatorial",
            startUrl: "https://erosita.mpe.mpg.de/dr1/erodat/static/hips/eRASS1_RGB_Rate_c010/",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/2MASS/color",
            name: "2MASS colored",
            id: "P/2MASS/color",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
            startUrl: "https://alaskybis.cds.unistra.fr/2MASS/Color",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/DSS2/red",
            name: "DSS2 Red (F+R)",
            id: "P/DSS2/red",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "fits",
            cooFrame: "equatorial",
            numBitsPerPixel: 16,
            // options
            minCut: {fits: 1000.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 10000.0, jpeg: 255.0, png: 255.0},
            colormap: "magma",
            stretch: "Linear",
            imgFormat: "fits",
            startUrl: "https://alaskybis.cds.unistra.fr/DSS/DSS2Merged",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/DM/I/350/gaiaedr3",
            name: "Density map for Gaia EDR3 (I/350/gaiaedr3)",
            id: "P/DM/I/350/gaiaedr3",
            maxOrder: 7,
            tileSize: 512,
            numBitsPerPixel: -32,
            cooFrame: "equatorial",
            minCut: {fits: 0.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 12000.0, jpeg: 255.0, png: 255.0},
            stretch: "asinh",
            colormap: "rdylbu",
            imgFormat: "fits",
            startUrl: "https://alaskybis.cds.unistra.fr/ancillary/GaiaEDR3/density-map",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/PanSTARRS/DR1/g",
            name: "PanSTARRS DR1 g",
            id: "P/PanSTARRS/DR1/g",
            maxOrder: 11,
            tileSize: 512,
            imgFormat: "fits",
            cooFrame: "equatorial",
            numBitsPerPixel: -32,
            // options
            minCut: {fits: -34.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 7000.0, jpeg: 255.0, png: 255.0},
            stretch: "asinh",
            colormap: "redtemperature",
            startUrl: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/g",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/PanSTARRS/DR1/color-z-zg-g",
            name: "PanSTARRS DR1 color",
            id: "P/PanSTARRS/DR1/color-z-zg-g",
            maxOrder: 11,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
            startUrl: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/color-z-zg-g",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/DECaPS/DR2/color",
            name: "DECaPS DR2 color",
            id: "P/DECaPS/DR2/color",
            maxOrder: 11,
            cooFrame: "equatorial",
            tileSize: 512,
            imgFormat: "png",
            startUrl: "https://alasky.cds.unistra.fr/DECaPS/DR2/CDS_P_DECaPS_DR2_color",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/Fermi/color",
            name: "Fermi color",
            id: "P/Fermi/color",
            maxOrder: 3,
            imgFormat: "jpeg",
            tileSize: 512,
            cooFrame: "equatorial",
            startUrl: "https://alasky.cds.unistra.fr/Fermi/Color",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/GALEXGR6_7/NUV",
            id: "P/GALEXGR6_7/NUV",
            name: "GALEXGR6_7 NUV",
            maxOrder: 8,
            imgFormat: "png",
            tileSize: 512,
            cooFrame: "equatorial",
            startUrl: "https://alasky.cds.unistra.fr/GALEX/GALEXGR6_7_NUV",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/IRIS/color",
            id: "P/IRIS/color",
            name: "IRIS colored",
            maxOrder: 3,
            tileSize: 256,
            imgFormat: "jpeg",
            cooFrame: "galactic",
            startUrl: "https://alasky.cds.unistra.fr/IRISColor",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/Mellinger/color",
            id: "P/Mellinger/color",
            name: "Mellinger colored",
            maxOrder: 4,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "galactic",
            startUrl: "https://alasky.cds.unistra.fr/MellingerRGB",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/SDSS9/color",
            id: "P/SDSS9/color",
            name: "SDSS9 colored",
            maxOrder: 10,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
            startUrl: "https://alasky.cds.unistra.fr/SDSS/DR9/color",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/SPITZER/color",
            id: "P/SPITZER/color",
            name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            maxOrder: 9,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "galactic",
            startUrl: "https://alasky.cds.unistra.fr/Spitzer/SpitzerI1I2I4color",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/allWISE/color",
            id: "P/allWISE/color",
            name: "AllWISE color Red (W4) , Green (W2) , Blue (W1) from raw Atlas Images",
            maxOrder: 8,
            tileSize: 512,
            imgFormat: "jpeg",
            cooFrame: "equatorial",
            startUrl: "https://alaskybis.cds.unistra.fr/AllWISE/RGB-W4-W2-W1",
            dataproductType: "image",
        },
        {
            creatorDid: "ivo://CDS/P/SDSS9/g",
            id: "P/SDSS9/g",
            name: "SDSS9 band-g",
            maxOrder: 10,
            tileSize: 512,
            numBitsPerPixel: 16,
            imgFormat: "fits",
            cooFrame: "equatorial",
            minCut: {fits: 0.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 1.8, jpeg: 255.0, png: 255.0},
            stretch: "linear",
            colormap: "redtemperature",
            startUrl: "https://alasky.cds.unistra.fr/SDSS/DR9/band-g",
            dataproductType: "image",
        },
        {
            id: "P/Finkbeiner",
            name: "Halpha",
            maxOrder: 3,
            minCut: {fits: -10.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 800.0, jpeg: 255.0, png: 255.0},
            colormap: "rdbu",
            imgFormat: "fits",
            startUrl: "https://alasky.cds.unistra.fr/FinkbeinerHalpha",
            dataproductType: "image",
        },
        {
            id: "P/VTSS/Ha",
            name: "VTSS-Ha",
            maxOrder: 3,
            minCut: {fits: -10.0, jpeg: 0.0, png: 0.0},
            maxCut: {fits: 100.0, jpeg: 255.0, png: 255.0},
            colormap: "grayscale",
            imgFormat: "fits",
            startUrl: "https://alasky.cds.unistra.fr/VTSS/Ha",
            dataproductType: "image",
        },
        {
            id: "P/GLIMPSE360",
            name: "GLIMPSE360",
            maxOrder: 9,
            imgFormat: "jpeg",
            minOrder: 3,
            startUrl: "https://alasky.cds.unistra.fr/IPAC/IPAC_P_GLIMPSE360",
        },
    ];

    return HiPSList;
})();