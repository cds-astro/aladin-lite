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

import { Layout } from "../Layout.js";
import { ContextMenu } from "../Widgets/ContextMenu.js";
import { Input } from "../Widgets/Input.js";
import { Color } from "../../Color.js";
import { ALEvent } from "../../events/ALEvent.js";
import { SAMPActionButton } from "../Button/SAMP.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import helpIconBtn from '../../../../assets/icons/help.svg';
import { Utils } from "../../Utils";

export class SettingsCtxMenu extends ContextMenu {
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
        ALEvent.RETICLE_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            const color = e.detail.color;
            let hex = new Color(color).toHex();

            self.reticleColorInput.set(hex)
        });

        this.toggleCheckbox = (checkbox) => {
            const pastVal = checkbox.get();
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
            let windowEnabled = self.menu.isEnabled(window)
            if(windowEnabled) {
                self.menu.disable(window)
            } else {
                self.menu.enable(window)
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
                label: {
                    content: [self.backgroundColorInput, 'Background color']
                },
                action(o) {}
            },
            {
                label: 'Reticle',
                subMenu: [
                    {
                        label: {
                            content: [self.reticleCheckbox, 'Reticle']
                        },
                        action(o) {
                            let newVal = self.toggleCheckbox(self.reticleCheckbox);
                            self.aladin.showReticle(newVal)
        
                            self._attach();
                        }
                    },
                    {
                        label: {
                            content: [
                                self.reticleColorInput,
                                'Color',
                            ]
                        },
                        action(o) {}
                    },
                    {
                        label: 'Size',
                        subMenu: reticleSizeSubMenu
                    }
                ]
            },
            {
                label: {
                    content: [self.hpxGridCheckbox, 'HEALPix grid']
                },
                action(o) {
                    let newVal = self.toggleCheckbox(self.hpxGridCheckbox);
                    self.aladin.showHealpixGrid(newVal)

                    self._attach();
                }
            },
            {
                label: {
                    content: [self.sampBtn, 'SAMP']
                },
            },
            {
                label: 'Features',
                subMenu: [
                    {
                        label: 'Stack',
                        selected: self.menu.isEnabled('stack'),
                        action(o) {
                            toggleWindow('stack')
                            toggleWindow('overlay')
                            toggleWindow('survey')

                            self._attach();
                        }
                    },
                    {
                        label: 'Simbad',
                        selected: self.menu.isEnabled('simbad'),
                        action(o) {
                            toggleWindow('simbad');

                            self._attach();
                        }
                    },
                    {
                        label: 'Go to',
                        selected: self.menu.isEnabled('goto'),
                        action(o) {                            
                            toggleWindow('goto');

                            self._attach();
                        }
                    },
                    {
                        label: 'Grid',
                        selected: self.menu.isEnabled('grid'),
                        action(o) {
                            toggleWindow('grid');

                            self._attach();
                        }
                    },
                    {
                        label: 'FullScreen',
                        selected: self.menu.isEnabled('fullscreen'),
                        action(o) {
                            toggleWindow('fullscreen');

                            self._attach();
                        }
                    }
                ]
            },
            {
                label: {
                    icon: {
                        tooltip: {content: 'Documentation about Aladin Lite', position: {direction: 'left'}},
                        iconURL: helpIconBtn,
                        cssStyle: {
                            cursor: 'help',
                        }
                    },
                    content: 'Help'
                },
                subMenu: [
                    {
                        label: 'Aladin Lite API',
                        action(o) {
                            Utils.openNewTab('https://aladin.cds.unistra.fr/AladinLite/doc/API/')
                        }
                    },
                    {
                        label: {
                            content: 'Contact us',
                            tooltip: { content: 'For bug reports, discussions, feature ideas...', position: {direction: 'bottom'} }
                        },
                        subMenu: [
                            {
                                label: 'GitHub',
                                action(o) {
                                    Utils.openNewTab('https://github.com/cds-astro/aladin-lite/issues')
                                }
                            },
                            {
                                label: 'by email',
                                action(o) {
                                    Utils.openNewTab('mailto:matthieu.baumann@astro.unistra.fr,thomas.boch@astro.unistra.fr?subject=Aladin Lite issue&body=message%20goes%20here')
                                }
                            }
                        ],
                    },
                    {
                        label: 'General documentation',
                        
                        action(o) {
                            Utils.openNewTab('https://aladin.cds.unistra.fr/AladinLite/doc/')
                        }
                    },
                    {
                        label: Layout.horizontal({layout: ['Examples'], tooltip: { content: 'How to embed Aladin Lite <br \>into your own webpages!', position: {direction: 'bottom'}}}),
                        action(o) {
                            Utils.openNewTab('https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/')
                        }
                    }
                ]
            }
        ]);
    }

    _show() {
        super.show({
            position: {
                nextTo: this.menu.controls['settings'],
                direction: 'bottom',
            }
        })
    }

    static singleton;

    static getInstance(aladin, menu) {
        if (!Settings.singleton) {
            Settings.singleton = new Settings(aladin, menu);
        }

        return Settings.singleton;
    }
}
 