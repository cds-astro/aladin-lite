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
    header: 'htmlCode' or DOM Element object,
    cssStyle: {...}
    subInputs: [
        {
            label: "ID",
            type: "text",
            value: "the placeholder value..."
        },
        ..
    ]
*/
export class Form extends Widget {
    constructor(layout, opt, target, position = "beforeend") {
        let el = Form._createInput(layout);

        // add it to the dom
        super(el, opt, target, position);
    }

    static _createInput(layout) {
        if (layout.type === "text" || layout.type === "number") {
            let inputEl = document.createElement('input');
            inputEl.type = layout.type;
            inputEl.classList.add('aladin-input');

            if (layout.type === "number") {
                inputEl.step = "any";
            }

            inputEl.value = layout.value;
            inputEl.name = layout.name;
            inputEl.id = layout.label;

            let labelEl = document.createElement('label');
            labelEl.textContent = layout.label;
            labelEl.for = inputEl.id;

            let divEl = document.createElement("div");
            divEl.classList.add(labelEl.textContent, "aladin-form-input");

            divEl.appendChild(labelEl);
            divEl.appendChild(inputEl);

            return divEl;
        } else if (layout.type === "group") {
            let groupEl = document.createElement('div');
            groupEl.classList.add(layout.name, "aladin-form-input-group");
            for (const property in layout.cssStyle) {
                groupEl.style[property] = layout.cssStyle[property];
            }

            if (layout.header instanceof Element) {
                groupEl.innerHTML = '<div class="aladin-form-group-header"></div>';
                groupEl.firstChild.appendChild(layout.header);
            } else if (layout.header instanceof Widget) {
                let el = layout.header.element();
                groupEl.innerHTML = '<div class="aladin-form-group-header"></div>';
                groupEl.firstChild.appendChild(el);
            } else {
                groupEl.innerHTML = '<div class="aladin-form-group-header">' + layout.header + '</div>';
            }

            layout.subInputs.forEach((subInput) => {
                let inputEl = Form._createInput(subInput)
                groupEl.appendChild(inputEl);
            });

            return groupEl;
        }
    }

    values() {
        let inputs = this.el.querySelectorAll('.aladin-input');

        let values = {};
        for (let input of inputs) {
            values[input.name] = input.value;
        }

        return values;
    }

    set(name, value) {
        let inputs = this.el.querySelectorAll('.aladin-input');
        for (let input of inputs) {
            if (input.name === name) {
                input.value = value;

                return;
            }
        }
    }

    _show() {
        super._show();
    }
}
