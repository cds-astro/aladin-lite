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
import { Coo } from "../../libs/astro/coo.js";
import { MocServer } from "../../MocServer.js";
import  autocomplete from 'autocompleter';

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Input } from "../Widgets/Input.js";
import A from "../../A.js";
import { FileLoaderActionButton } from "../Button/FileLoader.js";
import { ConeSearchActionButton } from "../Button/ConeSearch.js";
import { HiPSDefinition } from "../../HiPSDefinition.js";
import { Form } from "../Widgets/Form.js";
import { Angle } from "../../libs/astro/angle.js";
/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

 export class ConeSearchBox extends Box {
    constructor(aladin, callback) {
        let self;
        let selectorBtn = new ConeSearchActionButton({
            tooltip: {content: 'Select the area to query the catalogue with', position: {direction: 'left'}},
            onBeforeClick(e) {
                self._hide();
            },
            action(circle) {
                // convert to ra, dec and radius in deg
                try {
                    let [ra, dec] = aladin.pix2world(circle.x, circle.y);
                    let radius = aladin.angularDist(circle.x, circle.y, circle.x + circle.r, circle.y);
    
                    //var hlon = this.lon/15.0;
                    //var strlon = Numbers.toSexagesimal(hlon, this.prec+1, false);
                    let coo = new Coo(ra, dec, 7);
                    let [lon, lat] = coo.format('s2');

                    let fov = new Angle(radius, 1).format();
                    //selectorBtn.update({tooltip: {content: 'center: ' + ra.toFixed(2) + ', ' + dec.toFixed(2) + '<br\>radius: ' + radius.toFixed(2), position: {direction: 'left'}}})    
                    form.set('ra', lon)
                    form.set('dec', lat)
                    form.set('rad', fov)
                } catch (e) {
                    alert(e, 'Cone search out of projection')
                }

                self._show()
            }
        }, aladin)

        let form = new Form({
            name: 'header',
            type: 'group',
            submit(values) {
                self._hide();
                let coo = new Coo();
                coo.parse(values.ra + ' ' + values.dec)

                let theta = new Angle();
                theta.parse(values.rad)
                callback({
                    ra: coo.lon,
                    dec: coo.lat,
                    rad: theta.degrees(),
                    limit: values.limit,
                })
            },
            subInputs: [
                {
                    type: 'group',
                    header: Layout.horizontal([selectorBtn, 'Cone search']),
                    subInputs: [{
                            label: "ra [deg]:",
                            name: "ra",
                            type: "text",
                            placeholder: 'Right ascension',
                            actions: {
                                'change': (e, input) => {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "dec [deg]:",
                            name: "dec",
                            type: "text",
                            placeholder: 'Declination',
                            actions: {
                                'change': (e, input) => {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "Rad [deg]:",
                            name: "rad",
                            type: 'text',
                            placeholder: 'Radius',
                            actions: {
                                'change': (e, input) => {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        }
                    ]
                },
                {
                    type: 'group',
                    header: 'Max number of sources',
                    subInputs: [{
                        label: "Limit:",
                        name: "limit",
                        step: '1',
                        type: "number",
                        placeholder: 'Limit number of sources',
                        actions: {
                            'change': (e, input) => {
                                input.addEventListener('blur', (event) => {});
                            },
                        }
                    }]
                },
            ]
        });

        super({
            header: {
                draggable: true,
                title: 'Cone Search box'
            },
            content: Layout.horizontal({
                layout: [form]
            })
        }, aladin.aladinDiv)

        // hide by default
        console.log("hide cone search")
        this._hide();

        self = this;
        this.addClass('aladin-anchor-center');
    }

    static box = undefined;

    static getInstance(aladin, callback) {
        if (!ConeSearchBox.box) {
            ConeSearchBox.box = new ConeSearchBox(aladin, callback);
        }

        return ConeSearchBox.box;
    }
}
