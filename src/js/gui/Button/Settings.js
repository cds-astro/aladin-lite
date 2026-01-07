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

import { Layout } from "../Layout.js";
import { Input } from "../Widgets/Input.js";
import { Color } from "../../Color.js";
import { ALEvent } from "../../events/ALEvent.js";
import { SAMPActionButton } from "../Button/SAMP.js";
import helpIconBtn from '../../../../assets/icons/help.svg';
import { Utils } from "../../Utils";
import { GridSettingsCtxMenu } from "./../CtxMenu/GridSettings.js";
import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import settingsIconUrl from './../../../../assets/icons/settings.svg';
import { SimbadPointer } from "./SimbadPointer.js";
import { GridEnabler } from "./GridEnabler.js";
import { Stack } from "./Stack.js";
import { ColorPicker } from "./ColorPicker.js";
import { ShareActionButton } from "./ShareView.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/ActionButton.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/
/**
 * Class representing a Tabs layout
 * @extends CtxMenuActionButtonOpener
 */
 export class SettingsButton extends CtxMenuActionButtonOpener {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        super({
            icon: {
                size: 'medium',
                monochrome: true,
                url: settingsIconUrl
            },
            classList: ['aladin-settings-control'],
            tooltip: {
                content: 'Some general settings for the<br/>coordinate grid, the reticle or tools to enable',
                position: {
                    direction: 'right'
                }
            },
            ctxMenu: undefined,
            ...options
        }, aladin);

        this.aladin = aladin;
        let ctxMenu = this._buildLayout()
        this.update({ctxMenu})
    }

    _buildLayout() {
        let self = this;
        let aladin = this.aladin;
        let backgroundColorInput = Input.color({
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

        let reticleColorInput = Input.color({
            value: new Color(aladin.getReticle().getColor()).toHex(),
            name: 'reticleColor',
            change(e) {
                let hex = e.target.value;
                aladin.setDefaultColor(hex)
            }
        });

        // Event received from aladin
        ALEvent.BACKGROUND_COLOR_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            const {r, g, b} = e.detail.color;

            let hex = Color.rgbToHex(r, g, b);
            backgroundColorInput.set(hex)
        });

        ALEvent.RETICLE_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            const color = e.detail.color;
            let hex = new Color(color).toHex();

            reticleColorInput.set(hex)
        });

        const toggleCheckbox = (checkbox) => {
            const pastVal = checkbox.get();
            const curVal = !pastVal;

            checkbox.set(curVal)

            return curVal;
        };

        let hpxGridCheckbox = Input.checkbox({
            name: 'hpxgrid', checked: aladin.healpixGrid(),
            click(e) {
                let newVal = toggleCheckbox(hpxGridCheckbox);
                aladin.showHealpixGrid(newVal)
            }
        })
        let reticleCheckbox = Input.checkbox({
            name: 'reticle',
            checked: aladin.isReticleDisplayed(),
            click(e) {
                let newVal = toggleCheckbox(reticleCheckbox);
                aladin.showReticle(newVal)
            }
        })

        let reticle = aladin.getReticle();

        let sliderReticleSize = Input.slider({
            name: 'reticleSize',
            type: 'range',
            min: 0.0,
            max: 50,
            value: reticle.getSize(),
            change(e) {
                reticle.update({size: e.target.value})
            }
        });

        let sampBtn = new SAMPActionButton({
            size: 'small',
            action(conn) {
                if (conn.isConnected()) {
                    conn.unregister();
                } else {
                    conn.register();
                }

                //self._hide()
            }
        }, aladin);

        return [
            GridSettingsCtxMenu.getLayout(aladin),
            {
                label: {
                    content: ['Reticle']
                },
                subMenu: [
                    {
                        label: {
                            content: [reticleCheckbox, 'Show/Hide']
                        },
                        mustHide: false,
                        action(o) {
                            let newVal = toggleCheckbox(reticleCheckbox);
                            aladin.showReticle(newVal)
                        }
                    },
                    {
                        label: {
                            content: [reticleColorInput, 'Color']
                        },
                    },
                    {
                        label: Layout.horizontal(['Size', sliderReticleSize]),
                    }
                ]
            },
            {
                label: {
                    content: [backgroundColorInput, 'Back color']
                },
                mustHide: false,
            },
            {
                label: {
                    content: 'Light/Dark mode'
                },
                mustHide: false,
                action(o) {
                    const currentTheme = aladin.aladinDiv.getAttribute("data-theme");
                    const newTheme = currentTheme === "dark" ? "light" : "dark";
                    aladin._applyTheme(newTheme);
                    localStorage.setItem("theme", newTheme);
                }
            },
            {
                label: {
                    content: [hpxGridCheckbox, 'HEALPix grid']
                },
                mustHide: false,
                action(o) {
                    let newVal = toggleCheckbox(hpxGridCheckbox);
                    aladin.showHealpixGrid(newVal)
                }
            },
            {
                label: {
                    content: [sampBtn, 'SAMP']
                },
            },
            {
                label: 'Tools',
                subMenu: [
                    {
                        label: 'Stack',
                        mustHide: false,
                        action: (o) => {
                            let toolbar = aladin.toolbar;
                            if (!toolbar.has('stack')) {
                                toolbar.add('stack', new Stack(aladin));
                            } else {
                                if (toolbar.enabled('stack')) {
                                    toolbar.disable('stack')
                                } else {
                                    toolbar.enable('stack')
                                }
                            }
                        }
                    },
                    {
                        label: 'Simbad',
                        mustHide: false,
                        action: (o) => {
                            let toolbar = aladin.toolbar;
                            if (!toolbar.has('simbad')) {
                                toolbar.add('simbad', new SimbadPointer(aladin));
                            } else {
                                if (toolbar.enabled('simbad')) {
                                    toolbar.disable('simbad')
                                } else {
                                    toolbar.enable('simbad')
                                }
                            }
                        }
                    },
                    {
                        label: 'Grid',
                        mustHide: false,
                        action: (o) => {
                            let toolbar = aladin.toolbar;
                            if (!toolbar.has('grid')) {
                                toolbar.add('grid', new GridEnabler(aladin));
                            } else {
                                if (toolbar.enabled('grid')) {
                                    toolbar.disable('grid')
                                } else {
                                    toolbar.enable('grid')
                                }
                            }
                        }
                    },
                    {
                        label: 'Color picker',
                        mustHide: false,
                        action: (o) => {
                            let toolbar = aladin.toolbar;
                            if (!toolbar.has('picker')) {
                                toolbar.add('picker', new ColorPicker(aladin));
                            } else {
                                if (toolbar.enabled('picker')) {
                                    toolbar.disable('picker')
                                } else {
                                    toolbar.enable('picker')
                                }
                            }
                        }
                    },
                    {
                        label: 'Share view',
                        mustHide: false,
                        action: (o) => {
                            let toolbar = aladin.toolbar;
                            if (!toolbar.has('share')) {
                                toolbar.add('share', new ShareActionButton(aladin));
                            } else {
                                if (toolbar.enabled('share')) {
                                    toolbar.disable('share')
                                } else {
                                    toolbar.enable('share')
                                }
                            }
                        }
                    },
                ]
            },
            {
                label: {
                    icon: {
                        monochrome: true,
                        tooltip: {content: 'Documentation about Aladin Lite', position: {direction: 'top'}},
                        url: helpIconBtn,
                        size: 'small',
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
                        label: Layout.horizontal('Examples', { tooltip: { content: 'How to embed Aladin Lite <br \>into your own webpages!', position: {direction: 'bottom'}}}),
                        action(o) {
                            Utils.openNewTab('https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/')
                        }
                    }
                ]
            }
        ]
    }

}

