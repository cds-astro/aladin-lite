// Copyright 2023 - UDS/CNRS
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
 * File GridSettingsCtxMenu
 *
 * Author: Matthieu Baumann [CDS]
 *
 *****************************************************************************/

import { ALEvent } from "../../events/ALEvent.js";
import { Input } from "../Widgets/Input.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Color } from "../../Color.js";
import thicknessLineIcon from './../../../../assets/icons/thickness.svg';

export let GridSettingsCtxMenu = (function () {

    let GridSettingsCtxMenu = {};

    GridSettingsCtxMenu.getLayout = function (aladin) {
        let colorInput = Input.color({
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
        });

        let opacitySlider = Input.slider({
            name: 'opacity',
            type: 'range',
            min: 0.0,
            max: 1.0,
            value: aladin.getGridOptions().opacity,
            change(e) {
                aladin.setCooGrid({opacity: +e.target.value})
            }
        });

        const labelSizeSlider = Input.slider({
            name: 'labelSize',
            type: 'range',
            tooltip: {
                content: 'size'
            },
            min: 0.0,
            max: 1.0,
            value: 0.5,
            change(e) {
                aladin.setCooGrid({labelSize: Math.round(+e.target.value * 20)})
            }
        });

        const thicknessLineBtn = ActionButton.createSmallSizedIconBtn({
            icon: {
                url: thicknessLineIcon,
                monochrome: true,
            },
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

        ALEvent.COO_GRID_UPDATED.listenedBy(aladin.aladinDiv, function (e) {
            let color = e.detail.color;

            let hexColor = Color.rgbToHex(Math.round(255 * color.r), Math.round(255 * color.g), Math.round(255 * color.b));
            colorInput.set(hexColor)
        });

        return {
            label: 'Grid',
            subMenu: [
                {
                    label: {
                        content: [colorInput, 'Color '],
                    },
                },
                {
                    label: {
                        content: ['Opacity ', opacitySlider],
                    },
                },
                {
                    label: {
                        content: ['Label', labelSizeSlider]
                    },
                }
            ]
        }
    }

    return GridSettingsCtxMenu;

})();
