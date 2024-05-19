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
import { MocServer } from "../../MocServer.js";
import { TogglerActionButton } from "../Button/Toggler.js";
import { Layout } from "../Layout.js";
import { Angle } from "../../libs/astro/angle.js";
import { ALEvent } from "../../events/ALEvent.js";
import { Utils } from "../../Utils.ts";
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

        let regimeBtn = new TogglerActionButton({
            content: 'Regime',
            tooltip: {content: 'Observation regime', position: {direction: 'bottom'}},
            toggled: true,
            actionOn: () => {
                self._triggerFilteringCallback();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });
        let spatialBtn = new TogglerActionButton({
            content: 'Inside view',
            tooltip: {content: 'Check for HiPS having observation in the view!', position: {direction: 'bottom'}},
            toggled: false,
            actionOn: () => {
                self._requestMOCServer();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });
        let resolutionBtn = new TogglerActionButton({
            content: 'Pixel res',
            tooltip: {content: 'Check for HiPS with a specific pixel resolution.', position: {direction: 'bottom'}},
            toggled: false,
            actionOn: () => {
                self._triggerFilteringCallback();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });

        let logSlider = new Input({
            label: "Max res [°/px]:",
            name: "res",
            value: 0.1,
            type: 'range',
            cssStyle: {
                width: '100%'
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
        });
        super(
            {
                classList: ['aladin-HiPS-filter-box'],
                close: false,
                content: Layout.vertical([
                    '<b>Filter by:</b>',
                    Layout.horizontal([regimeBtn, spatialBtn, resolutionBtn]),
                    '<b>Details:</b>',
                    new Form({
                        subInputs: [
                            {
                                type: "group",
                                subInputs: [
                                    {
                                        label: "Regime:",
                                        name: "regime",
                                        value: "Optical",
                                        type: 'select',
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

                                            //regimeBtn.update({content: regime});

                                            self._triggerFilteringCallback();
                                        },
                                        tooltip: {
                                            content: "Observation regime",
                                            position: { direction: "right" },
                                        },
                                    },
                                    logSlider
                                ],
                            },
                        ],
                    }),
                ])
            },
            aladin.aladinDiv
        );

        self = this;

        this.browserClosed = false;

        this.callback = options.callback;

        this.regimeBtn = regimeBtn;
        this.spatialBtn = spatialBtn;
        this.resolutionBtn = resolutionBtn;

        this.params = {
            regime: "Optical",
            spatial: true,
            resolution: 1, // 1°/pixel
        };
        this.on = false;
        this.aladin = aladin;
        this._addListeners();
    }

    _addListeners() {
        const requestMOCServerDebounced = Utils.debounce(() => {
            this._requestMOCServer()
        }, 500);

        ALEvent.POSITION_CHANGED.listenedBy(this.aladin.aladinDiv, requestMOCServerDebounced);
        ALEvent.ZOOM_CHANGED.listenedBy(this.aladin.aladinDiv, requestMOCServerDebounced);
    }

    _requestMOCServer() {
        if (!this.spatialBtn.toggled || !this.on || this.browserClosed) {
            return;
        }

        let self = this;
        MocServer.getAllHiPSesInsideView(this.aladin)
            .then((HiPSes) => {
                let HiPSIDs = HiPSes.map((x) => x.ID);
                self.params["spatial"] = HiPSIDs;

                self._triggerFilteringCallback();
            })
    }

    _triggerFilteringCallback() {
        let filterParams = {};

        if (this.regimeBtn.toggled) {
            filterParams['regime'] = this.params['regime']
        }

        if (this.spatialBtn.toggled) {
            filterParams['spatial'] = this.params['spatial']
        }

        if (this.resolutionBtn.toggled) {
            filterParams['resolution'] = this.params['resolution']
        }

        if (this.on && this.callback) {
            this.callback(filterParams);
        }
    }

    signalBrowserStatus(closed) {
        this.browserClosed = closed;

        // open
        if (!closed) {
            this._requestMOCServer()
        }
    }

    enable(enable) {
        this.on = enable;

        if (this.on)
            this._requestMOCServer();

        this._triggerFilteringCallback();
    }
}
