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

import { DOMElement } from "./Widget";
import { ActionButton } from "./ActionButton";
import { Layout } from "../Layout";

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
options = {id: (btn option), id2: btn option, selected: id}
*/
export class RadioButton extends DOMElement {
    /**
     * Create a layout
     * @param {{layout: {type: String, name: String, value: Number | String, placeholder: Number | String, change: Function } | {type: String, name: String, checked: Boolean, change: Function } | { type: String, name: String, value: String, options: Array.<String>, change: Function }, cssStyle: Object}} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, aladin, target, position = 'beforeend') {
        let layout = [];
        // toggle on the selected button
        const toggled = options.selected;

        for (var key in options) {
            if (key !== 'selected') {
                let btnOptions = options[key];
                if (key === toggled) {
                    btnOptions.toggled = true;
                } else {
                    btnOptions.toggled = false;
                }

                let action = btnOptions.action;

                let btn = new ActionButton({
                    ...btnOptions,
                    action(e) {
                        action(e);

                        for (var otherBtn of layout) {
                            otherBtn.update({toggled: false});
                        }

                        btn.update({toggled: true});
                    }
                })

                layout.push(btn);
            }
        }

        let el = Layout.horizontal({layout});

        super(el, options)

        this.aladin = aladin;
    }
}
