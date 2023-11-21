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

import { Utils } from "../Utils";
import { DOMElement } from "./Widget";

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
    header: 'htmlCode' or DOM Element object,
    cssStyle: {...}
    submit(e) {},
    subInputs: [
        {
            label: "ID",
            type: "text",
            value: "the placeholder value...",
        },
        ..
    ]
*/
export class Form extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement('form');
        el.className = "aladin-form";

        let innerEl = Form._createInput(options, el);
        el.appendChild(innerEl);

        super(el, options);
        this.attachTo(target, position)
    }

    static _createInput(layout, formEl) {
        let isInput = false;
        let inputEl, labelEl;

        if (layout.type === "text" || layout.type === "number" || layout.type === "color") {
            inputEl = document.createElement('input');
            inputEl.type = layout.type;
            inputEl.classList.add('aladin-input');

            if (layout.type === "number") {
                inputEl.step = "any";
            }

            if (layout.value || layout.value === 0) {
                inputEl.value = layout.value;
            }

            inputEl.name = layout.name;
            inputEl.id = layout.label;

            if (layout.placeholder) {
                inputEl.placeholder = layout.placeholder;
            }

            isInput = true;
        } else if (layout.type === "checkbox") {
            inputEl = document.createElement('input');
            inputEl.type = "checkbox";
            inputEl.classList.add('aladin-input');

            inputEl.checked = layout.checked;
            inputEl.name = layout.name;
            inputEl.id = layout.label;

            isInput = true;
        } else if (layout.type === "select") {
            inputEl = document.createElement('select');
            inputEl.classList.add('aladin-input');
            inputEl.id = layout.label;
            inputEl.name = layout.name;

            if (layout.options) {
                let innerHTML = "";

                for (const option of layout.options) {
                    innerHTML += "<option>" + option + "</option>";
                }
                inputEl.innerHTML = innerHTML;
            }

            if (layout.value) {
                inputEl.value = layout.value;
            }

            isInput = true;
        }

        labelEl = document.createElement('label');
        if (layout.labelContent) {
            DOMElement.appendTo(layout.labelContent, labelEl);
        } else {
            labelEl.textContent = layout.label;
        }

        if (inputEl) {
            labelEl.for = inputEl.id;
        }

        if (layout.actions) {
            for (const what in layout.actions) {
                const actionFunc = layout.actions[what];
                inputEl.addEventListener(what, (e) => actionFunc(e, inputEl));
            }
        }

        if (isInput) {
            let divEl = document.createElement("div");
            divEl.classList.add("aladin-form-input");

            divEl.appendChild(labelEl);
            divEl.appendChild(inputEl);

            return divEl;
        }

        if (layout.subInputs) {
            let groupEl = document.createElement('div');
            groupEl.classList.add("aladin-form-input-group");
            for (const property in layout.cssStyle) {
                groupEl.style[property] = layout.cssStyle[property];
            }

            if (layout.submit) {
                formEl.addEventListener('submit', (e) => {
                    e.preventDefault();

                    layout.submit(e)
                });
            }

            if (layout.header) {
                let headerEl = document.createElement('div');
                headerEl.className = "aladin-form-group-header";

                Utils.appendTo(layout.header, headerEl);
                Utils.appendTo(headerEl, groupEl);
            }

            layout.subInputs.forEach((subInput) => {
                let inputEl = Form._createInput(subInput, formEl)
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
                if (input.type === "checkbox") {
                    input.checked = value;
                } else {
                    input.value = value;
                }

                return;
            }
        }
    }

    getInput(name) {
        let inputs = this.el.querySelectorAll('.aladin-input');

        for (let input of inputs) {
            if (input.name === name) {
                return input;
            }
        }
    }
}
