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

import filterOnUrl from "../../../../assets/icons/filter-on.svg";
import { Box } from "../Widgets/Box.js";
import { Angle } from "../../libs/astro/angle.js";
import { AladinUtils } from "../../AladinUtils.js";
import { Input } from "../Widgets/Input.js";
import { Icon } from "../Widgets/Icon.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Box/HiPSFilterBox.js
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class HiPSFilterBox extends Box {
    constructor(aladin, options) {
        let self;

        let regimeBtn = Input.checkbox({
            name: 'Freq',
            tooltip: {content: 'enable/disable', position: {direction: 'left'}},
            type: 'checkbox',
            checked: false,
            click(e) {
                self._triggerFilteringCallback();
            }
        });
        let resolutionBtn =  Input.checkbox({
            name: 'Resolution',
            tooltip: {content: 'enable/disable', position: {direction: 'left'}},
            type: 'checkbox',
            checked: false,
            click(e) {
                self._triggerFilteringCallback();
            }
        });

        super(
            {
                header: {
                    title: [
                        new Icon({
                            size: 'medium',
                            url: filterOnUrl,
                            monochrome: true,
                        }),
                        'Filter'
                    ],
                    draggable: false,
                },
                close: false,
                classList: ['aladin-HiPS-filter-box'],
                content: [
                    {
                        start: [
                            "Freq:",
                            Input.select({
                                tooltip: {
                                    content: "Observation regime",
                                    position: { direction: "left" },
                                },
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
                            }),
                        ],
                        end: [regimeBtn]
                    },
                    {
                        start: [
                            "Max resolution [°/px]:",
                            new Input({
                                name: "res",
                                value: 0.1,
                                type: 'range',
                                cssStyle: {
                                    width: '200px'
                                },
                                tooltip: {content: AladinUtils.degreesToString(0.1), position: {direction: 'bottom'}},
                                ticks: [0.001 / 3600, 0.01 / 3600, 0.1 / 3600, 1 / 3600, 1 / 60, 0.1],
                                stretch: "log",
                                min: 0.001 / 3600,
                                max: 0.1,
                                reversed: true,
                                change: (e, slider, deg) => {
                                    slider.update({value: e.target.value, tooltip: {content: AladinUtils.degreesToString(deg), position:{direction:'bottom'}}});

                                    let resolution = new Angle(deg);
                                    self.params["resolution"] = resolution.degrees();

                                    self._triggerFilteringCallback();
                                },
                            })
                        ],
                        end: [resolutionBtn]
                    }
                ]
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

    enable(enable) {
        this.on = enable;

        this._triggerFilteringCallback();
    }
}
