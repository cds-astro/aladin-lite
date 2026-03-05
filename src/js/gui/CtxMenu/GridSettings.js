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
import { Color } from "../../Color.js";

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
