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

import { Layout } from "../../Layout.js";
import { ContextMenu } from "../../Widgets/ContextMenu.js";
import { Input } from "../../Widgets/Input.js";
import { Color } from "../../../Color.js";
import { ALEvent } from "../../../events/ALEvent.js";
import { SAMPActionButton } from "../../Button/SAMP.js";

export class Settings extends ContextMenu {
    // Constructor
    constructor(aladin, menu) {
        super(aladin);
        let self = this;
        self.backgroundColorInput = Input.color({
            name: 'color',
            value: (() => {
                let {r, g, b} = aladin.getBackgroundColor();
                return Color.rgbToHex(r, g, b);
            })(),
            change(e) {
                let hex = e.target.value;
                aladin.setBackgroundColor(hex)
            }
        });
        self.reticleColorInput = Input.color({
            name: 'reticleColor',
            change(e) {
                let hex = e.target.value;
                let reticle = aladin.getReticle();
                reticle.update({color: hex})
            }
        });

        // Event received from aladin
        ALEvent.BACKGROUND_COLOR_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            const {r, g, b} = e.detail.color;

            let hex = Color.rgbToHex(r, g, b);
            self.backgroundColorInput.set(hex)
        });

        self.windowsVisible = {
            StackBox: aladin.options && aladin.options.showLayersControl,
            GridBox: aladin.options && aladin.options.showCooGridControl,
            SimbadPointer: aladin.options && aladin.options.showSimbadPointerControl,
            GotoBox: aladin.options && aladin.options.showGotoControl,
            FullScreen: aladin.options && aladin.options.showFullscreenControl,
        };

        this.toggleCheckbox = (checkbox) => {
            const pastVal = checkbox.get();
            console.log(pastVal, this.aladin.healpixGrid())
            const curVal = !pastVal;

            checkbox.set(curVal)

            return curVal;
        };

        self.hpxGridCheckbox = Input.checkbox({
            name: 'hpxgrid', checked: this.aladin.healpixGrid(),
            click(e) {
                let newVal = self.toggleCheckbox(self.hpxGridCheckbox);
                self.aladin.showHealpixGrid(newVal)
            }
        })
        self.reticleCheckbox = Input.checkbox({name: 'reticle', checked: this.aladin.isReticleDisplayed()})

        this.menu = menu;
        
        let sampBtn = new SAMPActionButton({
            action(conn) {
                if (conn.isConnected()) {
                    conn.unregister();
                    sampBtn.update({tooltip: {content: 'Connect to SAMP Hub'}})
                } else {
                    conn.register();
                    sampBtn.update({tooltip: {content: 'Disconnect'}})
                }

                self._hide()
            }
        }, aladin);
        this.sampBtn = sampBtn;

        this._attach();

    }

    _attach() {
        const toggleWindow = (window) => {
            self.windowsVisible[window] = !self.windowsVisible[window];
            if(!self.windowsVisible[window]) {
                self.menu.removeControl(window)
            } else {
                self.menu.appendControl(window)
            }
        }

        let self = this;

        let reticleSizeSubMenu = [];
        const fontSize = 5; // 10px
        for (let em = 1; em <= 10; em++) {
            let pxSize = fontSize * em;
            reticleSizeSubMenu.push({
                label: em,
                action(o) {
                    let reticle = self.aladin.getReticle();
                    reticle.update({size: pxSize})
                }
            })
        }

        this.attach([
            {
                label: Layout.horizontal({
                    layout: [
                        'Background color',
                        self.backgroundColorInput,
                    ]
                }),
                action(o) {}
            },
            {
                label: 'Reticle',
                subMenu: [
                    {
                        label: Layout.horizontal({layout: [self.reticleCheckbox, 'Reticle']}),
                        action(o) {
                            let newVal = self.toggleCheckbox(self.reticleCheckbox);
                            self.aladin.showReticle(newVal)
        
                            self._attach();
                        }
                    },
                    {
                        label: Layout.horizontal({
                            layout: [
                                'Color',
                                self.reticleColorInput,
                            ]
                        }),
                        action(o) {}
                    },
                    {
                        label: 'Size',
                        subMenu: reticleSizeSubMenu
                    }
                ]
            },
            {
                label: Layout.horizontal({layout: [self.hpxGridCheckbox, 'HEALPix grid']}),
                action(o) {
                    let newVal = self.toggleCheckbox(self.hpxGridCheckbox);
                    self.aladin.showHealpixGrid(newVal)

                    self._attach();
                }
            },
            {
                label: Layout.horizontal({layout: [self.sampBtn, 'SAMP']}),
            },
            {
                label: 'Features',
                subMenu: [
                    {
                        label: 'Stack',
                        selected: self.windowsVisible['StackBox'],
                        action(o) {
                            toggleWindow('StackBox')

                            self._attach();
                        }
                    },
                    {
                        label: 'Simbad',
                        selected: self.windowsVisible['SimbadPointer'],
                        action(o) {
                            toggleWindow('SimbadPointer');

                            self._attach();
                        }
                    },
                    {
                        label: 'Go to',
                        selected: self.windowsVisible['GotoBox'],
                        action(o) {                            
                            toggleWindow('GotoBox');

                            self._attach();
                        }
                    },
                    {
                        label: 'Grid',
                        selected: self.windowsVisible['GridBox'],
                        action(o) {
                            toggleWindow('GridBox');

                            self._attach();
                        }
                    },
                    {
                        label: 'FullScreen',
                        selected: self.windowsVisible['FullScreen'],
                        action(o) {
                            toggleWindow('FullScreen');

                            self._attach();
                        }
                    }
                ]
            }
        ]);
    }

    _show() {
        super.show({
            position: {
                anchor: this.menu.controls['Settings'],
                direction: 'bottom',
            }
        })
    }

    static singleton;

    static getInstance(aladin, parent) {
        if (!Settings.singleton) {
            Settings.singleton = new Settings(aladin, parent);
        }

        return Settings.singleton;
    }
}
 