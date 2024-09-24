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
 * File AladinUtils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
import { Aladin } from "./Aladin";
import { Sesame } from "./Sesame";

/**
 * @namespace AladinUtils
 * @description Aladin Lite utils API namespace for basic functions
 */
export let AladinUtils = {
        /**
         * @namespace HEALPix
         * @memberof AladinUtils
         * @description Namespace for HEALPix-related utilities within the Aladin Lite API.
         */
        HEALPix: {
            /**
             * Represents a geographical point with longitude and latitude coordinates.
             *
             * @typedef {Object} LonLat
             * @property {number} lon - The longitude coordinate.
             * @property {number} lat - The latitude coordinate.
             */

            /**
             * Represents the vertices of a HEALPix cell, where each vertex is a LonLat object.
             *
             * @typedef {Object} HpxCellVertices
             * @property {LonLat} v1 - The first vertex.
             * @property {LonLat} v2 - The second vertex.
             * @property {LonLat} v3 - The third vertex.
             * @property {LonLat} v4 - The fourth vertex.
             */

            /**
             * Computes HEALPix vertices for a given NSIDE and pixel index (ipix).
             *
             * @function
             * @memberof AladinUtils.HEALPix
             * @name vertices
             *
             * @param {number} nside - NSIDE parameter for the HEALPix grid.
             * @param {number | number[]} ipix - Pixel index or an array of pixel indices.
             * @throws {string} Throws an error if A.init is not called first.
             * @returns {HpxCellVertices[]} vertices - An array representing HEALPix cell vertices. Each element has v1, v2, v3, v4 properties. Each vi is an object having a lon and a lat property.
             */
            vertices: function(nside, ipix) {
                let wasm = Aladin.wasmLibs.core;
                if (!wasm) {
                    throw 'A.init must be called first'
                }

                // Cast to 1d array
                if (!Array.isArray(ipix)) {
                    ipix = [ipix];
                }

                const vertices = wasm.HEALPixVertices(nside, ipix)
                return vertices;
            },

           /**
             * Computes HEALPix pixel indices from angular coordinates (longitude and latitude).
             *
             * @function
             * @memberof AladinUtils.HEALPix
             * @name ang2pix
             *
             * @param {number} nside - NSIDE parameter for the HEALPix grid.
             * @param {number | number[]} lon - Longitude or an array of longitudes.
             * @param {number | number[]} lat - Latitude or an array of latitudes.
             * @throws {string} Throws an error if A.init is not called first.
             * @returns {number[]} ipix - Pixel index or an array of pixel indices.
             */
            ang2pix: function(nside, lon, lat) {
                let wasm = Aladin.wasmLibs.core;
                if (!wasm) {
                    throw 'A.init must be called first'
                }

                if (!Array.isArray(lon)) {
                    lon = [lon];
                }

                if (!Array.isArray(lat)) {
                    lat = [lat];
                }

                const ipix = wasm.HEALPixAng2Pix(nside, lon, lat)
                return ipix;
            },

            /**
             * Computes angular coordinates (longitude and latitude) from HEALPix pixel indices.
             *
             * @function
             * @memberof AladinUtils.HEALPix
             * @name pix2ang
             *
             * @param {number} nside - NSIDE parameter for the HEALPix grid.
             * @param {number | number[]} ipix - Pixel index or an array of pixel indices.
             *
             * @throws {string} Throws an error if A.init is not called first.
             * @returns {LonLat[]} lonlat - Longitude and latitude or an array of longitudes and latitudes.
             */
            pix2ang: function(nside, ipix) {
                let wasm = Aladin.wasmLibs.core;
                if (!wasm) {
                    throw 'A.init must be called first'
                }

                // Cast to 1d array
                if (!Array.isArray(ipix)) {
                    ipix = [ipix];
                }

                const lonlat = wasm.HEALPixPix2Ang(nside, ipix)
                return lonlat;
            }
        },

        /**
         * @namespace Sesame
         * @memberof AladinUtils
         * @description Namespace for Sesame related service.
         */
        Sesame: {
            /**
             * Find RA, DEC for any target (object name or position) <br/>
             * if successful, callback is called with an object {ra: {@link number}, dec: {@link number}} <br/>
             * if not successful, errorCallback is called
             *
             * @function
             * @memberof AladinUtils.Sesame
             * @name resolveAstronomicalName
             *
             * @param {string} target - object name or position
             * @param {Function} callback - if successful, callback is called with an object {ra: {@link number}, dec: {@link number}}
             * @param {Function} errorCallback - if not successful, errorCallback is called with the error
             */
            resolveAstronomicalName: Sesame.getTargetRADec
        },

        /**
         * @deprecated
         * 
         * Converts celestial coordinates (ra, dec) to screen coordinates (x, y) in pixels within the view.
         * Use {@link Aladin.world2pix} instead
         * 
         * 
         * @function
         * @memberof AladinUtils
         * @name radecToViewXy
         *
         * @param {number} ra - Right Ascension (RA) coordinate in degrees.
         * @param {number} dec - Declination (Dec) coordinate in degrees.
         * @param {Aladin} aladin - Aladin Lite object containing the WebAssembly API.
         * @returns {number[]} xy - A 2 elements array representing the screen coordinates [X, Y] in pixels.
         */
        radecToViewXy: function(ra, dec, aladin) {
            return aladin.world2pix(ra, dec);
        },

        /**
        * @function
        * @memberof AladinUtils
        * @name degreesToString
        * Convert a number in degrees into a string<br>
        *
        * @param numberDegrees number in degrees (integer or decimal)
        * @return a formattes string
        * 
        * @example <caption> Result in degrees </caption>
        * // returns "1°"
        * Numbers.degreesToString(1)
        * @example <caption> Result in arcminutes </caption>
        * // returns "6 arcmin"
        * Numbers.degreesToString(0.1);
        * @example <caption> Result in arcseconds </caption>
        * // returns "36 arcsec"
        * Numbers.degreesToString(0.01);
        */
        degreesToString: function(numberDegrees) {
            let setPrecision = 3
            let degrees = numberDegrees | 0;
            let notDegrees = Math.abs(numberDegrees - degrees);
            let minutes = notDegrees * 60 | 0;
            let seconds = ((notDegrees* 60 - minutes) *60).toPrecision(setPrecision);
            if (degrees!=0) {
                return numberDegrees.toPrecision(setPrecision) + '°';
            }
            else if (minutes!=0){
                return  (notDegrees * 60).toPrecision(setPrecision) + " arcmin";
            }
            else if (seconds!=0){
                return seconds + " arcsec";
            }
            else {
                return "0°";
            }
        }
 
    };
