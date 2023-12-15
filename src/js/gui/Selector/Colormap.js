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

import { SelectorButton } from "../Widgets/Selector";
/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Form.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

/* 
Exemple of layout object
{
    {
        label: "ID",
        type: "text",
        value: "the placeholder value...",
    },
*/

/*
options = {cmap1: {imgUrl, change}, cmap2: imgUrl, selected: cmap1}
*/
export class CmapSelector extends SelectorButton {
    /**
     * Create a layout
     * @param {{layout: {type: String, name: String, value: Number | String, placeholder: Number | String, change: Function } | {type: String, name: String, checked: Boolean, change: Function } | { type: String, name: String, value: String, options: Array.<String>, change: Function }, cssStyle: Object}} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, aladin, target, position = 'beforeend') {
        for (const cmap in options) {
            if (cmap === 'selected') {
                continue;
            }

            options[cmap] = {
                ...options[cmap],
                cssStyle: {
                    padding: '0px',
                    //border: 'none',
                    //borderRadius: '0',
                    //backgroundColor: 'black',
                    color: 'black',
                    overflow: 'hidden',
                    width: '6em',
                    'font-family': 'monospace',
                },
                content: cmap,
                tooltip: {content: cmap, position: {direction: 'left'}},
            }
        }
        super(options, aladin, target, position)
    }
}
