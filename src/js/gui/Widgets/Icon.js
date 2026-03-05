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

import { DOMElement } from "./Widget";
import { Tooltip } from "./Tooltip";
/******************************************************************************
 * Aladin Lite project
 *
 * File gui/ActionButton.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

/**
 * Represents an action button that can be added to the DOM.
 *
 * @namespace
 * @typedef {Object} ActionButton
 * @extends DOMElement
 *
 * @param {Object} opt - Options for configuring the action button.
 * @param {HTMLElement} target - The target element to which the button will be attached.
 * @param {Object|string} [position="beforeend"] - The position to insert the button relative to the target.
 *
 * @property {boolean} [opt.toggled=false] - Whether the button is initially toggled.
 * @property {function} [opt.action] - The callback function to execute when the button is clicked.
 * @property {string} [opt.title] - The title attribute for the button.
 * @property {string} [opt.iconURL] - The URL of the icon image for the button.
 * @property {boolean} [opt.disabled=false] - Whether the button is initially disabled.
 * @property {HTMLElement|string|Widget} [opt.content] - The content to be added to the button.
 * @property {CSSStyleSheet} [opt.cssStyle] - The CSS styles to apply to the button.
 * @property {string} [opt.tooltip] - The tooltip text for the button.
 * @property {Object|string} [opt.position] - The position of the button.
 *   - If an object:
 *     - `{ nextTo: DOMElement, direction: 'left' | 'right' | 'top' | 'bottom' }`
 *     - `{ top: number, left: number }`
 *     - `{ anchor: 'left top' | 'left center' | 'left bottom' | 'right top' | 'right center' | 'right bottom' | 'center top' | 'center center' | 'center bottom' }`
 *   - If a string: One of the following values: "beforebegin", "afterbegin", "beforeend", "afterend".
 *
 * @example
 * const actionButton = new ActionButton({
 *   toggled: false,
 *   action: (e) => { /* callback function * },
 *   title: "Click me",
 *   iconURL: "path/to/icon.png",
 *   cssStyle: "color: red;",
 *   tooltip: {
 *     position: {
 *       direction: 'left,
 *     },
 *     content: 'A tooltip'
 *   },
 *   position: { nextTo: someDOMElement, direction: 'right' }
 * }, document.getElementById('container'));
 */
export class Icon extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement('div');

        // add it to the dom
        super(el, options);
        this._show();

        this.addClass('aladin-icon')

        this.attachTo(target, position)
    }

    _show() {
        this.el.innerHTML = '';

        if (this.options.size === 'small') {
            this.addClass('aladin-small-sized-icon')
        } else if (this.options.size === 'medium') {
            this.addClass('aladin-medium-sized-icon')
        }

        if (this.options.title) {
            this.el.setAttribute('title', this.options.title);
        }

        if (this.options.url) {
            let img = document.createElement('img');
            img.src = this.options.url;

            this.el.appendChild(img);
        }

        if (this.options.disabled) {
            this.el.disabled = true;
            this.addClass('disabled')
        } else {
            this.el.disabled = false;
            this.removeClass('disabled')
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.monochrome === undefined || this.options.monochrome === true) {
            this.addClass('aladin-icon-monochrome');
        }

        // trigger el added
        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        if (this.options.position) {
            this.setPosition(this.options.position)
        }

        this.isHidden = false;
    }

    // SVG icons templates are stored here rather than in a CSS, as to allow
    // to dynamically change the fill color
    // Pretty ugly, haven't found a prettier solution yet
    static SVG_ICONS = {
        CATALOG: '<svg xmlns="http://www.w3.org/2000/svg"><polygon points="1,0,5,0,5,3,1,3"  fill="FILLCOLOR" /><polygon points="7,0,9,0,9,3,7,3"  fill="FILLCOLOR" /><polygon points="10,0,12,0,12,3,10,3"  fill="FILLCOLOR" /><polygon points="13,0,15,0,15,3,13,3"  fill="FILLCOLOR" /><polyline points="1,5,5,9"  stroke="FILLCOLOR" /><polyline points="1,9,5,5" stroke="FILLCOLOR" /><line x1="7" y1="7" x2="15" y2="7" stroke="FILLCOLOR" stroke-width="2" /><polyline points="1,11,5,15"  stroke="FILLCOLOR" /><polyline points="1,15,5,11"  stroke="FILLCOLOR" /><line x1="7" y1="13" x2="15" y2="13" stroke="FILLCOLOR" stroke-width="2" /></svg>',
        MOC: '<svg xmlns="http://www.w3.org/2000/svg"><polyline points="0.5,7,2.5,7,2.5,5,7,5,7,3,10,3,10,5,13,5,13,7,15,7,15,9,13,9,13,12,10,12,10,14,7,14,7,12,2.5,12,2.5,10,0.5,10,0.5,7" stroke-width="1" stroke="FILLCOLOR" fill="transparent" /><line x1="1" y1="10" x2="6" y2="5" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="2" y1="12" x2="10" y2="4" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="5" y1="12" x2="12" y2="5" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="7" y1="13" x2="13" y2="7" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="10" y1="13" x2="13" y2="10" stroke="FILLCOLOR" stroke-width="0.5" /></svg>',
        OVERLAY: '<svg xmlns="http://www.w3.org/2000/svg"><polygon points="10,5,10,1,14,1,14,14,2,14,2,9,6,9,6,5" fill="transparent" stroke="FILLCOLOR" stroke-width="2"/></svg>',
        COLOR: '<svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px" viewBox="0 0 122.88 112.75" style="enable-background:new 0 0 122.88 112.75" xml:space="preserve"><style type="text/css">.st0{fill-rule:evenodd;clip-rule:evenodd;fill:#6BBE66;} .st1{fill-rule:evenodd;clip-rule:evenodd;fill:#FF4141;} .st2{fill-rule:evenodd;clip-rule:evenodd;fill:#00A1F1;}</style><g><path class="st0" d="M56.66,105.35c-5.94,4.6-13.4,7.34-21.5,7.34C15.74,112.69,0,96.95,0,77.53c0-13.47,7.58-25.17,18.7-31.07 c1.96,6.96,5.68,13.19,10.65,18.16c4.64,4.64,10.38,8.2,16.79,10.24c-0.06,0.9-0.09,1.81-0.09,2.73 C46.06,88.25,50.07,97.98,56.66,105.35L56.66,105.35z"/><path class="st1" d="M122.88,77.59c0,19.42-15.74,35.16-35.16,35.16S52.56,97,52.56,77.59c0-0.41,0.01-0.82,0.02-1.23 c2.03,0.3,4.11,0.46,6.23,0.46c11.5,0,21.92-4.66,29.46-12.2c5.45-5.45,9.4-12.41,11.17-20.19 C113.1,49.26,122.88,62.28,122.88,77.59L122.88,77.59z"/><path class="st2" d="M93.97,35.16c0,19.42-15.74,35.16-35.16,35.16S23.65,54.58,23.65,35.16S39.39,0,58.81,0 S93.97,15.74,93.97,35.16L93.97,35.16z"/></g></svg>'
    }

    static dataURLFromSVG(icon) {
        let changeSVGSize = (svg, size) => {
            let str = svg.replace(/FILLCOLOR/g, 'black')
            let elt = document.createElement('div');
            elt.innerHTML = str;

            elt.style.width = size;
            elt.style.height = size;

            return elt.innerHTML;
        };

        let color = icon.color || 'black';
        let size = icon.size || '1rem';
        let svg = icon.svg;

        return 'data:image/svg+xml;base64,' + window.btoa(changeSVGSize(svg.replace(/FILLCOLOR/g, color), size));
    }
}
