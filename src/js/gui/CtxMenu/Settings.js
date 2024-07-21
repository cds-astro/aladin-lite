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
import helpIconBtn from '../../../../assets/icons/help.svg';
import { Utils } from "../../Utils";
import { GridSettingsCtxMenu } from "./GridSettings.js";

export class SettingsCtxMenu extends ContextMenu {
    // Constructor
    constructor(aladin, options) {
        super(aladin, {hideOnClick: true});
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

        let reticleColor = new Color(aladin.getReticle().getColor())
        self.reticleColorInput = Input.color({
            value: reticleColor.toHex(),
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
        self.reticleCheckbox = Input.checkbox({
            name: 'reticle',
            checked: this.aladin.isReticleDisplayed(),
            click(e) {
                let newVal = self.toggleCheckbox(self.reticleCheckbox);
                self.aladin.showReticle(newVal)
            }
        })

        this.features = options && options.features;

        let sampBtn = new SAMPActionButton({
            size: 'small',
            action(conn) {
                if (conn.isConnected()) {
                    conn.unregister();
                } else {
                    conn.register();
                }

                self._hide()
            }
        }, aladin);
        this.sampBtn = sampBtn;

        this.attach();
    }

    attach() {
        let self = this;

        const toggleFeature = (name) => {
            let feature = self.features[name];
            if(feature.isHidden) {
                feature._show();
            } else {
                feature._hide();
            }
        }

        let reticle = self.aladin.getReticle();

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

        let options = [
            //ProjectionCtxMenu.getLayout(self.aladin),
            GridSettingsCtxMenu.getLayout(self.aladin),
            {
                label: {
                    content: ['Reticle']
                },
                subMenu: [
                    {
                        label: {
                            content: [self.reticleCheckbox, 'Show/Hide']
                        },
                        mustHide: false,
                        action(o) {
                            let newVal = self.toggleCheckbox(self.reticleCheckbox);
                            self.aladin.showReticle(newVal)
        
                            self.attach();
                        }
                    },
                    {
                        label: {
                            content: [self.reticleColorInput, 'Color']
                        },
                    },
                    {
                        label: Layout.horizontal(['Size', sliderReticleSize]),
                    }
                ]
            },
            {
                label: {
                    content: [self.backgroundColorInput, 'Back color']
                },
            },
            {
                label: {
                    content: [self.hpxGridCheckbox, 'HEALPix grid']
                },
                mustHide: false,
                action(o) {
                    let newVal = self.toggleCheckbox(self.hpxGridCheckbox);
                    self.aladin.showHealpixGrid(newVal)

                    self.attach();
                }
            },
            {
                label: {
                    content: [self.sampBtn, 'SAMP']
                },
            },
            {
                label: 'Tools',
                subMenu: [
                    {
                        label: 'Stack',
                        selected: !self.features['stack'].isHidden,
                        action(o) {
                            toggleFeature('stack')
                        }
                    },
                    {
                        label: 'Simbad',
                        selected: !self.features['simbad'].isHidden,
                        action(o) {
                            toggleFeature('simbad');
                        }
                    },
                    {
                        label: 'Grid',
                        selected: !self.features['grid'].isHidden,
                        action(o) {
                            toggleFeature('grid');
                        }
                    }
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
                        label: Layout.horizontal({layout: ['Examples'], tooltip: { content: 'How to embed Aladin Lite <br \>into your own webpages!', position: {direction: 'bottom'}}}),
                        action(o) {
                            Utils.openNewTab('https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/')
                        }
                    }
                ]
            }
        ]
        super.attach(options);
    }

    show(options) {
        this.attach();

        this.position = (options && options.position) || this.position || { anchor: 'center center'}; 

        super.show({
            position: this.position,
        })
    }
}
 