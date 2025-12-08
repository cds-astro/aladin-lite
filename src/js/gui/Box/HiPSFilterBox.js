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
import { Form } from "../Widgets/Form.js";
import { Layout } from "../Layout.js";
import { Angle } from "../../libs/astro/angle.js";
import { AladinUtils } from "../../AladinUtils.js";
import { Input } from "../Widgets/Input.js";


/******************************************************************************
 * Aladin Lite project
 *
 * File gui/HiPSBrowserBox.js
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class HiPSFilterBox extends Box {
    constructor(aladin, options) {
        let self;

        let regimeBtn = Input.checkbox({
            name: 'Freq',
            tooltip: {content: 'Observation bandwidth', position: {direction: 'left'}},
            type: 'checkbox',
            checked: false,
            click(e) {
                self._triggerFilteringCallback();
            }
        });
        let resolutionBtn =  Input.checkbox({
            name: 'Resolution',
            tooltip: {content: 'Check for HiPS with a specific pixel resolution.', position: {direction: 'left'}},
            type: 'checkbox',
            checked: false,
            click(e) {
                self._triggerFilteringCallback();
            }
        });

        let regimeOption = Layout.horizontal({
            tooltip: {
                content: "Observation regime",
                position: { direction: "right" },
            },
            label: 'Freq: ',
            layout: [Input.select({
                value: "Optical",
                options: [
                    "Radio",
                    "Infrared",
                    "Millimeter",
                    "Optical",
                    "UV",
                    "EUV",
                    "X-ray",
                    "Gamma-ray",
                ],
                change: (e) => {
                    let regime = e.target.value;
                    self.params["regime"] = regime;

                    self._triggerFilteringCallback();
                },
            }), regimeBtn]
        });

        let resolutionOption = Layout.horizontal({
            label: "Max resolution [°/px]:",
            layout: [
                new Input({
                    name: "res",
                    value: 0.1,
                    type: 'range',
                    cssStyle: {
                        width: '200px'
                    },
                    tooltip: {content: AladinUtils.degreesToString(0.1), position: {direction: 'bottom'}},
                    ticks: [0.1 / 3600, 1 / 3600, 1 / 60, 0.1],
                    stretch: "log",
                    min: 0.1 / 3600,
                    max: 0.1,
                    reversed: true,
                    change: (e, slider, deg) => {
                        slider.update({value: e.target.value, tooltip: {content: AladinUtils.degreesToString(deg), position:{direction:'bottom'}}});

                        let resolution = new Angle(deg);
                        self.params["resolution"] = resolution.degrees();

                        self._triggerFilteringCallback();
                    },
                }),
                resolutionBtn,
            ]
        });
        super(
            {
                header: {
                    title: 'Filter tags',
                    draggable: false,
                },
                close: false,
                classList: ['aladin-HiPS-filter-box'],
                content: Layout.vertical([
                    new Form({
                        subInputs: [
                            {
                                type: "group",
                                subInputs: [
                                    regimeOption,
                                    resolutionOption 
                                ],
                            },
                        ],
                    }),
                ])
            },
            aladin.aladinDiv
        );

        self = this;

        this.callback = options.callback;

        this.regimeBtn = regimeBtn;
        this.resolutionBtn = resolutionBtn;

        this.params = {
            regime: "Optical",
            highlight: true,
            resolution: 1, // 1°/pixel
        };
        this.on = false;
        this.aladin = aladin;
    }

    _triggerFilteringCallback() {
        let filterParams = {};

        if (this.regimeBtn.checked) {
            filterParams['regime'] = this.params['regime']
        }

        if (this.resolutionBtn.checked) {
            filterParams['resolution'] = this.params['resolution']
        }

        if (this.on && this.callback) {
            this.callback(filterParams);
        }
    }

    /*signalBrowserStatus(closed) {
        this.browserClosed = closed;

        // open
        if (!closed) {
            this._requestMOCServer()
        }
    }*/

    enable(enable) {
        this.on = enable;

        this._triggerFilteringCallback();
    }
}
