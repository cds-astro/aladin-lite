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
import { samp } from "../libs/samp";
import A from "../A";

export class SAMPConnector {

    static _createTag = (function() {
        var count = 0;
        return function() {
            return "t-" + ++count;
        };
    })();

    constructor(aladin) {
        // Define listeners
        let cc = new samp.ClientTracker();
        let callHandler = cc.callHandler;
        this.cc = cc;

        let self = this;
        // listen for hub deconnexion/shutdown and unregister if so
        callHandler["samp.hub.event.shutdown"] = function(senderId, message, isCall) {
            self.unregister();
        };

        callHandler["samp.hub.disconnect"] = function(senderId, message, isCall) {
            self.unregister();
        };

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
            const image = aladin.createImageFITS(url, {name}, (e) => window.alert(e));

            aladin.setOverlayImageLayer(image, name);
        };

        callHandler["table.load.votable"] = function(senderId, message, isCall) {
            let params = message["samp.params"];

            let id = params['table-id'];
            let url = params['url'];
            let name = params['name'] || id;

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

        let selectCatalog = (id, url) => {
            for (const cat of aladin.getOverlays()) {
                if (cat.name === id || cat.url === url) {
                    return cat;
                }
            }

            return null;
        }
        callHandler["table.select.rowList"] = function(senderId, message, isCall) {
            let params = message["samp.params"];

            let id = params['table-id'];
            let url = params['url'];
            let rowList = params['row-list'];

            // search for the catalog
            let catalog = selectCatalog(id, url)

            if (catalog) {
                let objects = [];
                for (const idx of rowList) {
                    objects.push(catalog.sources[idx]);
                }

                aladin.selectObjects(objects);
            }
        };

        callHandler["table.highlight.row"] = function(senderId, message, isCall) {
            let params = message["samp.params"];

            let id = params['table-id'];
            let url = params['url'];
            let row = params['row'];

            // search for the catalog
            let catalog = selectCatalog(id, url)

            if (catalog) {
                const source = catalog.sources[row];
                aladin.selectObjects([[source]]);
            }
        };

        let subs = cc.calculateSubscriptions();
        let meta = {
            "samp.name": "Aladin Lite",
            "samp.description": "Aladin Lite web visualizer SAMP connector",
            "samp.icon.url": "https://raw.githubusercontent.com/cds-astro/aladin-lite/master/assets/aladin-logo.gif"
        };
        // Arrange for document to be adjusted for presence of hub every 2 sec.
        this.connector = new samp.Connector("Aladin Lite", meta, cc, subs);
        //window.addEventListener('load', (e) => {
        /*let onHubAvailability = this.connector.onHubAvailability((isHubRunning) => {
            console.log(isHubRunning, this.isHubRunning)
            if (this.isHubRunning !== isHubRunning) {
                if (isHubRunning === false) {
                    // Reset the connector when the hub disconnects
                    this.unregister();
                }
            }
            this.isHubRunning = isHubRunning;
            ALEvent.SAMP_HUB_RUNNING.dispatchedTo(aladin.aladinDiv, { isHubRunning } );
        }, 2000);     */
        //});
        this.connected = false;

        // This is triggered when closing the web app
        window.addEventListener('unload', (e) => {
            this.unregister();
        });

        // This is triggered when refreshing the page
        window.addEventListener("beforeunload", (e) => {
            this.unregister();
        });

        this.aladin = aladin;
        this.connectionAsked = false;
        this.msg = {}
    }

    // Broadcasts a message given a hub connection.
    _send(mtype, params) {
        this._pushMsgToAllClients(mtype, params);

        if (!this.connected) {
            let warnMsg = 'Please connect the client. Go to Settings (gear icon) -> SAMP';
            alert(warnMsg);
            throw warnMsg;
        }

        if (this.sending) {
            // We are waiting for acks to be received from the clients 
            return;
        }

        this.sending = true;

        let conn = this.connector.connection;

        let sendMsgToAClient = (id) => {
            // Check if the id is still valid
            let validClient = this.cc.ids[id];
            if (!validClient) {
                // Remove all the messages to that client
                delete this.msg[id];
                // Do not call any messages
                return;
            }

            const msg = this.msg[id].shift();
            console.log('Send msg:', msg, ' to client ', id)

            let tag = SAMPConnector._createTag();

            let doneFunc = (responderId, msgTag, response) => {
                console.log('done func', responderId, msgTag, response)
                delete this.cc.replyHandler[tag];

                if (this._noMoreMsgToSend()) {
                    // We finished sending messages
                    this.sending = false;
                    return;
                }

                // No more messages to send
                if (this.msg[id].length === 0) {
                    // No more message to send for this client
                    return;
                }

                // There are still messages to send for that client
                sendMsgToAClient(id);
            };

            this.cc.replyHandler[tag] = doneFunc;
            let errFunc = doneFunc;

            conn.call([id, tag, msg], null, errFunc);
        }

        // Send the first message to all clients
        for (const id in this.msg) {
            sendMsgToAClient(id)
        }
    }

    _pushMsgToAllClients(mtype, params) {
        // Create the message
        const msg = new samp.Message(mtype, params);

        for (const id in this.cc.ids) {
            if (id === 'hub') {
                continue;
            }

            // New client found ? create a new entry
            if (!this.msg[id]) {
                this.msg[id] = [];
            }

            // Push the message to all clients
            this.msg[id].push(msg); 
        }
    }

    _noMoreMsgToSend() {
        for (const id in this.msg) {
            let validClient = this.cc.ids[id];
            if (!validClient) {
                delete this.msg[id];
            } else {
                if (this.msg[id] && this.msg[id].length > 0) {
                    return false;
                }
            }
        }

        return true;
    }

    unregister() {
        this.connector.unregister();
        this.connected = false;
        ALEvent.SAMP_DISCONNECTED.dispatchedTo(this.aladin.aladinDiv);
    }

    register() {
        new Promise((resolve, reject) => {
            if (this.connected) {
                if (this.connector.connection) {
                    return resolve(this.connector.connection)
                } else {
                    return reject();
                }
            }

            // It is not connected
            if (this.connectionAsked === true) {
                return reject('Connection is being asked');
            }

            // It is not connected and the connection has not
            // been asked, thus we ask the user

            var regErrHandler = (err) => {
                this.connected = false;
                this.connectionAsked = false;

                window.alert('Could not connect to a SAMP Hub. Maybe the hub is missing')

                reject(err);
            };
            var regSuccessHandler = (conn) => {
                this.connected = true;
                this.connectionAsked = false;

                ALEvent.SAMP_CONNECTED.dispatchedTo(this.aladin.aladinDiv);

                resolve(conn)
            };
            this.connectionAsked = true;
            this.connector.runWithConnection(
                regSuccessHandler,
                regErrHandler
            );
        })
    }

    isConnected() {
        return this.connected;
    }

    isHubCurrentlyRunning() {
        return this.isHubRunning;
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
     * Highlight a row
     * @param {String} url - URL of the VOTable document to load
     * @param {String} [tableId] - Identifier which may be used to refer to the loaded table in subsequent messages
     * @param {Integer} [row] - Row index (zero-based) of the row to highlight.
     */
    tableSelectRowList(tableId, url, rowList) {
        this._send("table.select.rowList", {
            "table-id": tableId,
            url,
            "row-list": rowList
        })
    }

    /**
     * Highlight a row
     * @param {String} url - URL of the VOTable document to load
     * @param {String} [tableId] - Identifier which may be used to refer to the loaded table in subsequent messages
     * @param {Integer} [row] - Row index (zero-based) of the row to highlight.
     */
     highlightRowTable(tableId, url, row) {
        this._send("table.highlight.row", {
            "table-id": tableId,
            url,
            row
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

    ping() {
        this._send("samp.app.ping", {})
    }
}

