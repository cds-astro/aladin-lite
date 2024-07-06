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
import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { ConeSearchActionButton } from "../Button/ConeSearch.js";
import { Coo } from "../../libs/astro/coo.js";
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
    constructor(aladin, options) {
        let self;
        let selectorBtn = new ConeSearchActionButton({
            tooltip: {content: 'Select the area to query the catalogue with', position: {direction: 'left'}},
            onBeforeClick(e) {
                self._hide();
            },
            action(circle) {
                // convert to ra, dec and radius in deg
                try {
                    let [ra, dec] = aladin.pix2world(circle.x, circle.y, options.frame);
                    let radius = aladin.angularDist(circle.x, circle.y, circle.x + circle.r, circle.y, options.frame);
    
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

        let [ra, dec] = aladin.getRaDec();
        let centerCoo = new Coo(ra, dec, 5);
        let [defaultRa, defaultDec] = centerCoo.format('s2');

        let fov = aladin.getFov();
        let fovAngle = new Angle(Math.min(fov[0], fov[1]) / 2, 1).format()

        let form = new Form({
            submit: {
                content: 'Accept',
                action(values) {
                    self._hide();
                    let coo = new Coo();
                    coo.parse(values.ra + ' ' + values.dec)
    
                    let theta = new Angle();
                    theta.parse(values.rad)
                    self.callback && self.callback({
                        ra: coo.lon,
                        dec: coo.lat,
                        rad: theta.degrees(),
                        limit: values.limit,
                    })
                }
            },
            subInputs: [
                {
                    type: 'group',
                    header: Layout.horizontal([selectorBtn, 'Cone search']),
                    subInputs: [
                        {
                            label: "ra:",
                            name: "ra",
                            type: "text",
                            value: defaultRa,
                            placeholder: 'Right ascension',
                            actions: {
                                change(e, input) {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "dec:",
                            name: "dec",
                            type: "text",
                            value: defaultDec,
                            placeholder: 'Declination',
                            actions: {
                                change(e, input) {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "Rad:",
                            name: "rad",
                            type: 'text',
                            value: fovAngle,
                            placeholder: 'Radius',
                            actions: {
                                change(e, input) {
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
                        value: 1000,
                        type: "number",
                        placeholder: 'Limit number of sources',
                        actions: {
                            change(e, input) {
                                input.addEventListener('blur', (event) => {});
                            },
                        }
                    }]
                },
            ]
        });

        super(
            {
                header: {
                    draggable: true,
                    title: 'Cone Search box'
                },
                content: form
            },
            aladin.aladinDiv
        )

        this._hide();

        self = this;
    }

    attach(options) {
        this.callback = options.callback;
        super.update(options)
    }
}
