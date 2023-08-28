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

import { Widget } from "./Widget";

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
    name: 'ID',
    type: 'group',
    headerEl: 'htmlCode' or DOM Element object,
    subInputs: [{
        label: "ID",
        type: "text",
        value: inputParam.get("value")
    }
*/
export class Form extends Widget {
    constructor(target, layout, opt, position = "beforeend") {
        let el = this._createInput(layout);

        // add it to the dom
        super(el, target, opt, position);
    }

    _createInput(layout) {
        if (layout.type === "text" || layout.type === "number") {
            let inputEl = document.createElement('input');
            inputEl.type = layout.type;
            inputEl.classList.add('aladin-input');

            if (layout.type === "number") {
                inputEl.step = "any";
            }

            inputEl.value = layout.value;
            inputEl.name = layout.label;

            let labelEl = document.createElement('label');
            labelEl.textContent = layout.label;
            labelEl.for = input.id;

            let divEl = document.createElement("div");
            divEl.classList.add(labelEl.textContent, "aladin-form-input");

            divEl.appendChild(labelEl);
            divEl.appendChild(inputEl);

            return divEl;
        } else if (layout.type === "group") {
            let groupEl = document.createElement('div');
            groupEl.classList.add(layout.name, "aladin-form-input-group");

            if (layout.header instanceof Element) {
                groupEl.innerHTML = '<div class="aladin-form-group-header"></div>';
                groupEl.firstChild.appendChild(layout.header);
            } else {
                groupEl.innerHTML = '<div class="aladin-form-group-header">' + input.header + '</div>';
            }

            layout.subInputs.forEach((subInput) => {
                let inputEl = this._createInput(subInput)
                groupEl.appendChild(inputEl);
            });

            return groupEl;
        }
    }

    _show() {
        super._show();
    }
}
