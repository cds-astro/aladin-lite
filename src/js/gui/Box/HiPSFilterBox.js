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
import { Input } from "../Widgets/Input.js";
import { Form } from "../Widgets/Form.js";
import { MocServer } from "../../MocServer.js";
import { TogglerActionButton } from "../Button/Toggler.js";
import { Layout } from "../Layout.js";
import { Angle } from "../../libs/astro/angle.js";
import { ALEvent } from "../../events/ALEvent.js";
import { Utils } from "../../Utils.ts";
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
            content: 'Optical',
            toggled: true,
            actionOn: () => {
                self._triggerFilteringCallback();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });
        let spatialBtn = new TogglerActionButton({
            content: 'Spatial',
            toggled: false,
            actionOn: () => {
                self._triggerFilteringCallback();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });
        let resolutionBtn = new TogglerActionButton({
            content: '<=1°',
            toggled: false,
            actionOn: () => {
                self._triggerFilteringCallback();
            },
            actionOff: () => {
                self._triggerFilteringCallback();
            }
        });

        super(
            {
                header: {
                    title: 'HiPS Filter'
                },
                close: false,
                content: new Layout([
                    'Filters:',
                    new Layout([regimeBtn, spatialBtn, resolutionBtn]),
                    'Parameters:',
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
                                            "Optical",
                                            "UV",
                                            "Radio",
                                            "Infrared",
                                            "X-ray",
                                            "Gamma-ray",
                                        ],
                                        change: (e) => {
                                            let regime = e.target.value;
                                            self.params["regime"] = regime;

                                            regimeBtn.update({content: regime});

                                            self._triggerFilteringCallback();
                                        },
                                        tooltip: {
                                            content: "Observation regime",
                                            position: { direction: "right" },
                                        },
                                    },
                                    {
                                        label: "Angular res/px:",
                                        name: "res",
                                        value: "1°",
                                        type: 'text',
                                        classList: ['aladin-valid'],
                                        autocomplete: 'off',
                                        actions: {
                                            input: (e) => {
                                                e.target.classList.remove('aladin-not-valid');
                                                e.target.classList.remove('aladin-valid');

                                                let value = e.target.value;
                                                let resolution = new Angle();
                                                if (resolution.parse(value)) {
                                                    // The angle has been parsed
                                                    console.log(resolution.degrees())
                                                    self.params["resolution"] = resolution.degrees();

                                                    e.target.classList.add('aladin-valid');
                                                    resolutionBtn.update({content: '<=' + value});

                                                    self._triggerFilteringCallback();
                                                } else {
                                                    e.target.classList.add('aladin-not-valid');
                                                }
                                            },
                                            change: (e) => {
                                                e.preventDefault();
                                                e.stopPropagation();
                                            }
                                        }
                                    },
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

        this._triggerFilteringCallback();
    }
}
