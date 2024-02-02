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
import { Tooltip } from "./Tooltip";
import { Utils } from "../../Utils";
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
export class Input extends DOMElement {
        /**
     * Create a layout
     * @param {{layout: {type: String, name: String, value: Number | String, placeholder: Number | String, change: Function } | {type: String, name: String, checked: Boolean, change: Function } | { type: String, name: String, value: String, options: Array.<String>, change: Function }, cssStyle: Object}} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, target, position = "beforeend") {
        let el;
        super(el, options);

        this._show()
    }

    _show() {
        this.el.innerHTML = '';
        let layout = this.options.layout;

        if (layout.type === "checkbox") {
            this.el = document.createElement('input');
            this.el.type = layout.type;

            this.el.checked = layout.checked;

            if (layout.click) {
                this.el.removeEventListener('click', this.action);
                this.action = layout.click;

                this.el.addEventListener('click', this.action);
            }    
        } else if (layout.type === "select") {
            this.el = document.createElement('select');
            
            if (layout.options) {
                let innerHTML = "";

                for (const option of layout.options) {
                    innerHTML += "<option>" + option + "</option>";
                }
                this.el.innerHTML = innerHTML;
            }

            if (layout.value) {
                this.el.value = layout.value;
            }

            if (layout.change) {
                this.el.removeEventListener('change', this.action);

                this.action = layout.change;
                this.el.addEventListener('change', this.action);
            }    
        } else {
            this.el = document.createElement('input');
            this.el.type = layout.type;

            if (layout.type === "number" || layout.type === "range") {
                this.el.step = "any";
            }

            if (layout.type === "text") {
                this.el.enterkeyhint = "send";
            }

            if (layout.autocomplete) {
                this.el.autocomplete = layout.autocomplete;
            }

            if (layout.step) {
                this.el.step = layout.step;
            }
            if (layout.min) {
                this.el.min = layout.min;
            }
            if (layout.max) {
                this.el.max = layout.max;
            }

            if (layout.value || layout.value === 0) {
                this.el.value = layout.value;
            }

            if (layout.placeholder) {
                this.el.placeholder = layout.placeholder;
            }

            if (layout.change) {
                if (layout.type === 'color' || layout.type === 'range' || layout.type === "text") {
                    this.el.removeEventListener('input', this.action);
                    this.action = (e) => {
                        layout.change(e, this);
                    };
                    this.el.addEventListener('input', this.action);
                } else {
                    this.el.removeEventListener('change', this.action);
                    this.action = layout.change;
                    this.el.addEventListener('change', this.action);
                }
            }

            /*if (layout.autofocus) {
                this.el.autofocus = true;
            }*/
        }

        if (layout.actions) {
            for (const what in this.callbacks) {
                this.el.removeEventListener(what, this.callbacks[what]);
            }

            this.callbacks = layout.actions;

            for (const what in this.callbacks) {
                this.el.addEventListener(what, this.callbacks[what]);
            }
        }

        if (layout.name) {
            this.el.name = layout.name;
        }

        this.el.classList.add('aladin-input');

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        // Add padding for inputs except color ones
        if (Utils.hasTouchScreen() && layout.type !== "color") {
            // Add a little padding 
            this.el.style.padding = "0.5em";
        }

        super._show()
    }

    get() {
        if (this.el.type === "checkbox") {
            return this.el.checked;
        } else {
            return this.el.value;
        }
        
    }

    set(value) {
        if (this.el.type === "checkbox") {
            this.el.checked = value;
        } else {
            this.el.value = value;
        }
    }

    static color(options) {
        let change = options.change || ((e) => {});
        let el = new Input({
            layout: {
                name: options.name || 'color',
                type: 'color',
                value: options.value || '#000000',
                change
            }
        });
        el.addClass("aladin-input-color");

        return el;
    }

    static slider(options) {
        let change = options.change || ((e) => {});
        let el = new Input({
            cssStyle: options.cssStyle,
            tooltip: options.tooltip,
            layout: {
                name: options.name || 'slider',
                type: 'range',
                min: options.min || 0.0,
                max: options.max || 1.0,
                value: options.value,
                ticks: options.ticks,
                change
            }
        });
        el.addClass("aladin-input-range")

        return el;
    }

    static checkbox(options) {
        let el = new Input({
            cssStyle: options.cssStyle,
            tooltip: options.tooltip,
            layout: {
                name: options.name || 'checkbox',
                type: 'checkbox',
                checked: options.checked,
                click: options.click
            }
        });
        el.addClass("aladin-input-checkbox");

        return el;
    }

    static number(options) {
        let el = new Input({
            cssStyle: options.cssStyle,
            tooltip: options.tooltip,
            layout: {
                name: options.name || 'number',
                type: 'number',
                value: options.value,
                change: options.change,
                placeholder: options.placeholder,
            }
        });
        el.addClass("aladin-input-number");

        return el;
    }

    static text(options) {
        let el = new Input({
            cssStyle: options.cssStyle,
            tooltip: options.tooltip,
            layout: {
                name: options.name || 'text',
                type: 'text',
                ...options
            }
        });
        el.addClass("aladin-input-text");

        return el;
    }

    static select(options) {
        let el = new Input({
            cssStyle: options.cssStyle,
            tooltip: options.tooltip,
            layout: {
                name: options.name || 'select',
                type: 'select',
                ...options
            }
        });
        el.addClass("aladin-input-select");

        return el;
    }
}
