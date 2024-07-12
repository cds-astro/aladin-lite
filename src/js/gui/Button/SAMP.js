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
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { ActionButton } from "../Widgets/ActionButton.js";
import { ALEvent } from "../../events/ALEvent.js";
import waveOnIconUrl from '../../../../assets/icons/wave-on.svg';
import waveOffIconUrl from '../../../../assets/icons/wave-off.svg';

/*
options = {
    action: (connector) => {

    }
    tooltip
}
*/
 export class SAMPActionButton extends ActionButton {
     // Constructor
     constructor(options, aladin) {
        if (!aladin.samp) {
            options = {
                ...options,
                icon: {
                    monochrome: true,
                    url: waveOffIconUrl
                },
                tooltip: {content: 'SAMP disabled in Aladin Lite options', position: {direction: 'top'}},
                disable: true,
            }
        } else {
            //let isHubRunning = aladin.samp.isHubCurrentlyRunning();
            let tooltip = options && options.tooltip || {content: 'Connect to SAMP Hub', position: {direction: 'top'}}
            let action = options && options.action
            if (!action) {
                // default action, just connect and ping
                action = (connector) => {
                    connector.register();
                }
            }

            options = {
                ...options,
                icon: {
                    monochrome: true,
                    url: aladin.samp.isConnected() ? waveOnIconUrl : waveOffIconUrl
                },
                tooltip,
                action(o) {
                    action(aladin.samp)
                }
            }
        }

        super(options)

        this._addListeners(aladin);
    }

    _addListeners(aladin) {
        let self = this;
        ALEvent.SAMP_CONNECTED.listenedBy(aladin.aladinDiv, function (e) {
            const icon = {
                monochrome: true,
                url: waveOnIconUrl
            }
            self.update({icon})
        });

        ALEvent.SAMP_DISCONNECTED.listenedBy(aladin.aladinDiv, function (e) {            
            const icon = {
                monochrome: true,
                url: waveOffIconUrl
            }
            self.update({icon})
        });

        /*ALEvent.SAMP_HUB_RUNNING.listenedBy(aladin.aladinDiv, function (e) {
            const isHubRunning = e.detail.isHubRunning;

            if (hubRunning !== isHubRunning) {
                let newOptions = {
                    disable: !isHubRunning,
                    tooltip: isHubRunning ? {content: 'Connect to SAMP hub'} : {content: 'No hub running found'}
                };

                self.update(newOptions)
                if (isHubRunning === false) {
                    self.update({iconURL: waveOffIconUrl})
                }
                hubRunning = isHubRunning;
            }
        });*/
    }

    static sendSources(aladin) {
        return new SAMPActionButton({
            size: 'small',
            tooltip: {content: 'Send a table through SAMP Hub'},
            action(conn) {
                // hide the menu
                aladin.contextMenu._hide()

                let getSource = (o) => {
                    let s = o;
                    if (o.source) {
                        s = o.source
                    }

                    return s;
                };

                for (const objects of aladin.view.selection) {
                    let s0 = getSource(objects[0]);
                    const cat = s0.catalog;

                    const {url, name} = cat;
                    conn.loadVOTable(url, name, url);

                    let rowList = [];
                    for (const obj of objects) {
                        // select the source
                        let s = getSource(obj)
                        rowList.push('' + s.rowIdx);
                    };
                    conn.tableSelectRowList(name, url, rowList)
                }
            }
        }, aladin)
    }
}
 