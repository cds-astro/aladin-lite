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

import { Box } from "../Widgets/Box.js";
import { Input } from "../Widgets/Input.js";
import { Layout } from "../Layout.js";
import { ALEvent } from "../../events/ALEvent.js";
import { Color } from "../../Color.js";
import { ContextMenu } from "../Widgets/ContextMenu.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import thicknessLineIcon from './../../../../assets/icons/thickness.svg';
import labelSizeIcon from './../../../../assets/icons/font-size.svg';

export class GridBox extends Box {
    // Constructor
    constructor(aladin) {
        let colorInput = new Input({
            layout: {
                name: 'gridColor',
                type: 'color',
                value: (() => {
                    let c = aladin.getGridOptions().color;
                    const cHex = Color.rgbToHex(c.r * 255, c.g * 255, c.b * 255)
                    return cHex;
                })(),
                change(e) {
                    aladin.setCooGrid({color: e.target.value})
                }
            }
        });
        colorInput.addClass("aladin-input-color");

        let sliderOpacity = new Input({
            layout: {
                name: 'opacitySlider',
                type: 'range',
                min: 0.0,
                max: 1.0,
                value: aladin.getGridOptions().opacity,
                change(e) {
                    aladin.setCooGrid({opacity: +e.target.value})
                }
            }
        });
        sliderOpacity.addClass("aladin-input-range")

        const labelSizeBtn = new ActionButton({
            iconURL: labelSizeIcon,
            tooltip: {content: 'Change the label size', position: {direction: 'left'}},
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                cursor: 'pointer',
                width: '20px',
                height: '20px',
                padding: '0',
            },
            action(e) {
                let ctxMenu = ContextMenu.getInstance(aladin);
                ctxMenu._hide();

                let ctxMenuLayout = [];
                const fontSize = 5; // 10px
                for (let em = 1; em <= 5; em++) {
                    let pxSize = fontSize * em;
                    ctxMenuLayout.push({
                        label: em + 'em',
                        action(o) {
                            aladin.setCooGrid({labelSize: pxSize})
                        }
                    })
                }

                ctxMenu.attach(ctxMenuLayout);
                ctxMenu.show({
                    e: e,
                    position: {
                        nextTo: labelSizeBtn,
                        direction: 'bottom',
                    }
                })
            }
        });

        const thicknessLineBtn = new ActionButton({
            iconURL: thicknessLineIcon,
            tooltip: {content: 'Grid line thickness', position: {direction: 'left'}},
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                cursor: 'pointer',
                width: '20px',
                height: '20px',
                padding: '0',
            },
            action(e) {
                let ctxMenu = ContextMenu.getInstance(aladin);
                ctxMenu._hide();

                let ctxMenuLayout = [];
                for (let thickness = 1; thickness <= 5; thickness++) {
                    ctxMenuLayout.push({
                        label: thickness + 'px',
                        action(o) {
                            aladin.setCooGrid({thickness: thickness})
                        }
                    })
                }

                ctxMenu.attach(ctxMenuLayout);
                ctxMenu.show({
                    e: e,
                    position: {
                        nextTo: thicknessLineBtn,
                        direction: 'bottom',
                    }
                })
            }
        });
        let enableCheckbox = Input.checkbox({
            name: 'enableGrid',
            tooltip: {content: 'Enable/disable the grid', position: {direction: 'left'}},
            type: 'checkbox',
            checked: aladin.getGridOptions().enabled,
            click(e) {
                aladin.setCooGrid({enabled: enableCheckbox.get()})
            }
        });
        sliderOpacity.addClass("aladin-input-range")
        const layout = Layout.horizontal({
            layout: [
                enableCheckbox,
                labelSizeBtn,
                thicknessLineBtn,
                colorInput,
                sliderOpacity
            ]
        })

        layout.addClass('aladin-grid-frame');

        ALEvent.COO_GRID_UPDATED.listenedBy(aladin.aladinDiv, function (e) {
            let color = e.detail.color;

            let hexColor = Color.rgbToHex(Math.round(255 * color.r), Math.round(255 * color.g), Math.round(255 * color.b));
            colorInput.set(hexColor)
        });

        super({
            content: layout,
            cssStyle: {
                padding: '2px',
                'background-color': "#000",
            },
        }, aladin.aladinDiv)

        this.aladin = aladin;
    }

    _hide() {
        super._hide()

        let ctxMenu = ContextMenu.getInstance(this.aladin);
        ctxMenu._hide();
    }

    static singleton;

    static getInstance(aladin) {
        if (!GridBox.singleton) {
            GridBox.singleton = new GridBox(aladin);
        }

        return GridBox.singleton;
    }
}
