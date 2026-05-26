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
import hipsIconUrl from "../../../../assets/icons/hips.svg";
import addIconUrl from "../../../../assets/icons/plus.svg";
import settingsIconUrl from "../../../../assets/icons/settings.svg";
import { Icon } from "../Widgets/Icon.js";
import { LayerSelector } from "../Input/LayerSelector.js";
import { HiPSBrowserBox } from "./HiPSBrowserBox.js";
import { Input } from "../Widgets/Input.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Form } from "../Widgets/Form.js";
import { TogglerActionButton } from "../Button/Toggler.js";
import { Utils } from "../../Utils";
import { HiPSComposite } from "../../HiPSComposite.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Box/HiPSCompositeBox.js
 *
 * The code source of the interface for creating a new composite HiPS survey from multiple surveys
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class HiPSCompositeBox extends Box {
    static HiPSList = {};

    constructor(aladin, options) {
        let self;

        let nameInput = Input.text({
            tooltip: {
                global: true,
                aladin,
                content: 'What name for your composite survey?'
            },
            placeholder: "What name?...",
            autocomplete: 'off',
            autofocus: true,
            actions: {
                dblclick: (_) => {
                    nameInput.set('')
                },
                keydown: (e) => {
                    e.stopPropagation();
                    //
                }
            },
        });

        let content = [[
            new ActionButton({
                icon: {
                    url: addIconUrl,
                    size: "small",
                    monochrome: true,
                },
                tooltip: {
                    content: "Add a new layer",
                    position: { direction: "top" },
                },
                toggled: false,
                action(_) {
                    self.content.push(self._addNewHiPS());
                    self.update({content: self.content})
                }
            }),
            nameInput
        ]];

        super(
            {
                close: true,
                header: {
                    title: [
                        new Icon({
                            size: 'medium',
                            url: hipsIconUrl,
                            monochrome: true,
                        }),
                        "HiPS Compositor"
                    ],
                    draggable: true,
                },
                content,
                ...options,
            },
            aladin.aladinDiv
        );

        this.aladin = aladin;

        this.hipsOptions = [];
        self = this;

        this.layer = Utils.uuidv4();
        this.hipsComposite = new HiPSComposite(this.hipsOptions)

        this.numHiPSLayers = 0;
        this.content = content.concat([this._addNewHiPS()])
        this.update({content: this.content})

        this.openSettings = null;
    }

    _addNewHiPS() {
        const getIdHiPS = (node) => {
            let parent = node.parentElement;
            return [...parent.parentElement.children].indexOf(parent) - 1;
        };
        this.hipsOptions.push({});
        let self = this;
        let newLayerLayout = [
            new LayerSelector({
                change(e) {
                    let name = e.target.value;
                    let idLayer = getIdHiPS(e.target);

                    if (name === "More...") {
                        if (!aladin.hipsBrowser) {
                            aladin.hipsBrowser = new HiPSBrowserBox(aladin);
                        }

                        aladin.hipsBrowser._show({
                            selected: (hips) => {
                                self.hipsOptions[idLayer].id = hips.id || hips.url
                            },
                            position: { anchor: "center center" }
                        });
                    } else {
                        // it is an hips
                        let HiPSOptions = LayerSelector.cachedLayers[name];
                        self.hipsOptions[idLayer].id = HiPSOptions.id || HiPSOptions.url
                    }

                    self.hipsComposite.setOptions(self.hipsOptions);
                    self.aladin.setOverlayImageLayer(self.hipsComposite, self.layer);
                }
            }),
            this._createLayerSettingsBox(),
            ActionButton.BUTTONS(aladin)
                .remove((e) => {
                    let node = e.target.parentElement.parentElement.parentElement;
                    let idLayer = [...node.parentElement.children].indexOf(node);

                    this.content.splice(idLayer, 1)
                    this.hipsOptions.splice(idLayer, 1);

                    this.update({content: this.content})
                    this.numHiPSLayers = this.content.length - 1;
                })
        ];
        this.numHiPSLayers += 1;

        return newLayerLayout;
    }

    _createLayerSettingsBox() {
        let self = this;
        let layerSettingsBox = new Box({
            close: false,
            content: new Form({
                subInputs: [
                    {
                        type: 'color',
                        label: "Color",
                        value: 'red',
                        name: 'color',
                        change(e) {
                            let idLayer = getIdHiPS(e.target);

                            let hex = e.target.value;

                        }
                    },
                    {
                        label: 'Stretch',
                        type: "select",
                        name: 'stretch',
                        value: 'linear',
                        options: ['sqrt', 'linear', 'asinh', 'pow2', 'log'],
                        change(e) {},
                    },
                    {
                        type: 'number',
                        label: "Min cut",
                        name: 'mincut',
                        value: 0.0,
                        change: (e) => {
                            let minCut = +e.target.value
                        }
                    },
                    {
                        label: 'Max cut',
                        type: "number",
                        name: 'maxcut',
                        value: 0.0,
                        change: (e) => {
                            let maxCut = +e.target.value
                        }
                    },
                ]
            }),
        }, this.aladin.aladinDiv);
        layerSettingsBox._hide()

        /*let layerSettingsBtn = new TogglerActionButton({
            icon: { url: settingsIconUrl, monochrome: true },
            size: "small",
            tooltip: {
                content: "Settings",
                position: { direction: "top" },
            },
            toggled: false,
            on: (_) => {
                layerSettingsBox._show({
                    position: {
                        nextTo: layerSettingsBtn,
                        direction: "right",
                        aladin: self.aladin,
                    },
                });

                if (self.openSettings) {
                    self.openSettings.close();
                }

                self.openSettings = layerSettingsBtn;
            },
            off: (_) => {
                layerSettingsBox._hide();
                if (self.openSettings === layerSettingsBtn) {
                    self.openSettings = null;
                }
            },
        });*/

        return layerSettingsBtn
    }

    _hide() {
        if (this.openSettings)
            this.openSettings.close();

        super._hide()
    }
}
