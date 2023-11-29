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
 * File vo/samp.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { ALEvent } from "../events/ALEvent";
import { samp } from '../libs/samp';
import A from "../A";

export class SAMPConnector {
    constructor(aladin) {
        // Define listeners
        let cc = new samp.ClientTracker();
        let callHandler = cc.callHandler;

        callHandler["script.aladin.send"] = function(senderId, message, isCall) {
            var params = message["samp.params"];
            aladin.setBaseImageLayer(params["url"])
        };

        callHandler["coord.pointAt.sky"] = function(senderId, message, isCall) {
            var params = message["samp.params"];

            const {ra, dec} = params;
            aladin.gotoRaDec(+ra, +dec)
        };

        callHandler["coverage.load.moc.fits"] = function(senderId, message, isCall) {
            var params = message["samp.params"];

            const {url, name} = params;
            let moc = A.MOCFromURL(url, {name, lineWidth: 3});
            aladin.addMOC(moc);
        };

        callHandler["image.load.fits"] = function(senderId, message, isCall) {
            let params = message["samp.params"];

            const {url, name} = params;

            const image = aladin.createImageFITS(url, name, options, (e) => window.alert(e));

            aladin.setOverlayImageLayer(image, name);
        };

        callHandler["table.load.votable"] = function(senderId, message, isCall) {
            let params = message["samp.params"];

            let {url, name} = params;

            A.catalogFromURL(
                url,
                {name, onClick: 'showTable'},
                // Add the catalog if the query has succeded
                (catalog) => {
                    aladin.addCatalog(catalog)
                },
                (e) => window.alert(e)
            );
        };

        let subs = cc.calculateSubscriptions();
        let meta = {
            "samp.name": "Aladin Lite",
            "samp.description": "Aladin Lite web visualizer SAMP connector"
        };
        // Arrange for document to be adjusted for presence of hub every 2 sec.
        this.connector = new samp.Connector("Aladin Lite", meta, cc, subs);
        /*window.addEventListener('load', (e) => {
            this.connector.onHubAvailability((isHubRunning) => {
                // Communicate to Aladin Lite
                ALEvent.SAMP_AVAILABILITY.dispatchedTo(aladin.aladinDiv, { isHubRunning: isHubRunning } );
            }, 2000);
        });*/

        window.addEventListener('unload', (e) => {
            this.connector.unregister();
        });

        // TODO put that in a button
        //this.connector.register();
    }

    // Broadcasts a message given a hub connection.
    _send(mtype, params) {
        // Provides execution of a SAMP operation with register-on-demand.
        this.connector.runWithConnection(
            (connection) => {
                let msg = new samp.Message(mtype, params);
                connection.notifyAll([msg]);
            },
            (e) => {
                window.alert(e)
            }
        )
    }

    /**
     * Load a VOTable by url
     * @param {String} url - URL of the VOTable document to load
     * @param {String} [tableId] - Identifier which may be used to refer to the loaded table in subsequent messages
     * @param {String} [name] - Name which may be used to label the loaded table in the application GUI
     */
    loadVOTable(url, tableId, name) {
        this._send("table.load.votable", {
            url: url,
            "table-id": tableId,
            name: name
        })
    }

    /**
     * Load a fits image by url
     * @param {String} url - URL of the FITS image to load
     * @param {String} [imageId] - Identifier which may be used to refer to the loaded image in subsequent messages
     * @param {String} [name] - Name which may be used to label the loaded image in the application GUI
     */
    loadImageFITS(url, imageId, name) {
        this._send("image.load.fits", {
            "url": url,
            "image-id": imageId,
            "name": name
        })
    }

     /**
     * Load a Multi-Order-Coverage FITS file
     * @param {String} url - URL of a FITS file containing the MOC to load
     * @param {String} [coverageId] - Identifier which may be used to refer to the loaded coverage specification in subsequent messages
     * @param {String} [name] - Name which may be used to label the loaded image in the application GUI
     */
    loadMocFITS(url, coverageId, name) {
        this._send("coverage.load.moc.fits", {
            "url": url,
            "coverage-id": coverageId,
            "name": name
        })
    }

     /**
     * Load a HiPS by an url
     * @param {String} url - base URL for the HiPS to load
     */
    loadHiPS(url) {
        const cmd = 'load ' + url;
        this._send("script.aladin.send", { "script": cmd })
    }

    /**
     * Send a ra, dec position to the hub
     * @param {Float} ra - right ascension in degrees
     * @param {Float} dec - declination in degrees
     */
    centerAtRaDec(ra, dec) {
        this._send("coord.pointAt.sky", { "ra": ra.toString(), "dec": dec.toString() })
    }
}

