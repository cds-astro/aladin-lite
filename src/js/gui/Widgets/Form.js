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

import { ActionButton } from "./ActionButton";
import { DOMElement } from "./Widget";
import { Input } from "./Input";
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
        el.onsubmit = (e) => {
            e.preventDefault();
        };
        el.className = "aladin-form";

        super(el, options);

        this.attachTo(target, position)

        this._show()
    }

    _show() {
        this.el.innerHTML = '';

        let layout = [];
        if (this.options && this.options.subInputs) {
            this.options.subInputs.forEach(subInput => {
                layout.push(this._createInput(subInput))
            });
        }

        let self = this;
       
        // submit button
        if (this.options && this.options.submit) {
            this.submit = new ActionButton({
                ...this.options.submit,
                cssStyle: {
                    cursor: 'pointer',
                },
                action(e) {
                    e.preventDefault();
                    e.stopPropagation();

                    if (self.options.submit.action)
                        self.options.submit.action(self.values())
                }
            })
            layout.push(this.submit);
        }

        this.appendContent(Layout.vertical(layout))
        super._show();
    }

    _createInput(layout) {
        if (layout instanceof DOMElement || !layout.subInputs) {
            let input;
            let label = document.createElement('label');
            if (layout instanceof DOMElement) {
                input = layout;
                label.textContent = input.options.label;
            } else {
                input = new Input(layout);
                
                if (layout.labelContent) {
                    DOMElement.appendTo(layout.labelContent, label);
                } else {
                    label.textContent = layout.label;
                }
            }

            label.for = input.el.id;

            let item = new Layout([label, input]);
            item.addClass("aladin-form-input")

            return item;
        } else {
            let groupLayout = [];
            if (layout.header) {
                groupLayout.push(layout.header);
            }

            layout.subInputs.map((subInput) => {
                let input = this._createInput(subInput)
                groupLayout.push(input)
            });

            let item = new Layout({layout: groupLayout});
            item.addClass('aladin-form-group')

            return item;
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
