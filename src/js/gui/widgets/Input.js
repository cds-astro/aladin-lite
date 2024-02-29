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
        if (options && options.type === 'select') {
            el = document.createElement('select');
        } else {
            el = document.createElement('input');
        }

        super(el, options);

        this._show()

        this.attachTo(target, position)
    }

    _show() {
        this.el.innerHTML = '';

        if (this.options.classList) {
            this.options.classList.forEach((className) => this.addClass(className))
        }

        if (this.options.type === "checkbox") {
            this.el.type = this.options.type;

            this.el.checked = this.options.checked;

            if (this.options.click) {
                this.el.removeEventListener('click', this.action);
                this.action = this.options.click;

                this.el.addEventListener('click', this.action);
            }    
        } else if (this.options.type === "select") {            
            if (this.options.options) {
                let innerHTML = "";

                for (const option of this.options.options) {
                    innerHTML += "<option>" + option + "</option>";
                }
                this.el.innerHTML = innerHTML;
            }

            if (this.options.value) {
                this.el.value = this.options.value;
            }

            if (this.options.change) {
                this.el.removeEventListener('change', this.action);

                this.action = this.options.change;
                this.el.addEventListener('change', this.action);
            }    
        } else {
            this.el.type = this.options.type;

            if (this.options.type === "number" || this.options.type === "range") {
                this.el.step = "any";
            }

            if (this.options.type === "text") {
                this.el.enterkeyhint = "send";
            }

            if (this.options.autocomplete) {
                const autocomplete = this.options.autocomplete
                if (autocomplete instanceof Object && autocomplete !== null) {
                    let datalist = document.createElement('datalist');

                    autocomplete.options.forEach((o) => {
                        let option = document.createElement('option');
                        option.value = o;
                        datalist.appendChild(option);
                    })

                    datalist.id = 'ticks-' + this.options.name;
                    this.el.setAttribute('list', datalist.id);
                    
                    if (this.el.querySelector('#' + datalist.id)) {
                        this.el.querySelector('#' + datalist.id).remove()
                    }
                    this.el.appendChild(datalist);

                    this.el.autocomplete = 'on';
                } else {
                    this.el.autocomplete = autocomplete;
                }
            }

            if (this.options.step) {
                this.el.step = this.options.step;
            }
            if (this.options.min) {
                this.el.min = this.options.min;
            }

            if (this.options.max) {
                this.el.max = this.options.max;
            }

            if (this.options.value || this.options.value === 0) {
                this.el.value = this.options.value;
            }

            if (this.options.placeholder) {
                this.el.placeholder = this.options.placeholder;
            }

            if (this.options.change) {
                if (this.options.type === 'color' || this.options.type === 'range' || this.options.type === "text") {
                    this.el.removeEventListener('input', this.action);
                    this.action = (e) => {
                        this.options.change(e, this);
                    };
                    this.el.addEventListener('input', this.action);
                } else {
                    this.el.removeEventListener('change', this.action);
                    this.action = this.options.change;
                    this.el.addEventListener('change', this.action);
                }
            }
        }

        // add the personnalized style
        if (this.options.type)
            this.addClass("aladin-input-" + this.options.type);

        if (this.options.label) {
            this.el.id = this.options.label;
        }

        if (this.options.actions) {
            for (const what in this.callbacks) {
                this.el.removeEventListener(what, this.callbacks[what]);
            }

            this.callbacks = this.options.actions;

            for (const what in this.callbacks) {
                this.el.addEventListener(what, this.callbacks[what]);
            }
        }

        if (this.options.name) {
            this.el.name = this.options.name;
        }

        this.el.classList.add('aladin-input');
        this.el.classList.add('aladin-dark-theme');

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        /*// Add padding for inputs except color ones
        if (Utils.hasTouchScreen() && this.options.type !== "color") {
            // Add a little padding 
            this.el.style.padding = "0.5em";
        }*/

        super._show()
    }

    /*setPlaceholder(placeholder) {
        this.el.placeholder = placeholder;
    }*/

    update(options) {
        // if no options given, use the previous one set
        if (options) {
            this.options = {...this.options, ...options};
        }

        this._show();
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
            name: options.name || 'color',
            type: 'color',
            value: options.value || '#000000',
            change
        });

        return el;
    }

    static slider(options) {
        let change = options.change || ((e) => {});
        let el = new Input({
            name: options.name || 'slider',
            type: 'range',
            min: options.min || 0.0,
            max: options.max || 1.0,
            change,
            ...options
        });

        return el;
    }

    static checkbox(options) {
        let el = new Input({
            name: options.name || 'checkbox',
            type: 'checkbox',
            ...options
        });

        return el;
    }

    static number(options) {
        let el = new Input({
            name: options.name || 'number',
            type: 'number',
            ...options
        });

        return el;
    }

    static text(options) {
        let el = new Input({
            name: options.name || 'text',
            type: 'text',
            ...options
        });

        return el;
    }

    static select(options) {
        let el = new Input({
            name: options.name || 'select',
            type: 'select',
            ...options
        });

        return el;
    }
}
