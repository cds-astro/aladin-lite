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

import { Utils } from "./Utils";

import { DOMElement } from "./Widgets/Widget";
import { Tooltip } from "./Widgets/Tooltip";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/widgets/layout/Horizontal.js
 *
 * A layout grouping widgets horizontaly
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class Layout extends DOMElement {
    /**
     * Create a layout
     * @param {layout: Array.<DOMElement | String>, cssStyle: Object} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options = {layout: []}, target, position = "beforeend") {
        let el = document.createElement('div');

        // The user should also be able to give just a list of DOMElement
        if (options instanceof Array) {
            options['layout'] = options;
        }

        super(el, options);

        if (options.cssStyle) {
            this.setCss(options.cssStyle);
        }

        // 1. Attach self to the target
        this.attachTo(target, position);

        // 2. Once self is attached, attach the children
        if (options.layout) {
            for (const item of options.layout) {
                this.appendContent(item)
            }
        }

        /*
        for(let item of options.layout) {
            if (item instanceof DOMElement) {
                item.attachTo(el);
            } else if (item instanceof Element) {
                el.insertAdjacentElement('beforeend', item);
            } else {
                let wrapEl = document.createElement('div');
                wrapEl.innerHTML = item;
                el.insertAdjacentElement('beforeend', wrapEl);
            }
        }
        */

        // The tooltip has to be set once the element
        // lies in the DOM
        if (options.tooltip) {
            Tooltip.add(options.tooltip, this)
        }

        this.target = target;
    }

    static horizontal(options, target, position = "beforeend") {
        let layout = new Layout(options, target, position);
        layout.addClass('aladin-horizontal-list');

        return layout;
    }

    static vertical(options, target, position = "beforeend") {
        let layout = new Layout(options, target, position);
        layout.addClass('aladin-vertical-list');

        return layout;
    }

    static toolbar(options, target, position = "beforeend") {
        let layout = new Layout(options, target, position);
        layout.addClass('aladin-toolbar');

        return layout;
    }

    /**
     * Append an item at the beginning
     * @param {DOMElement} item - Represents the structure of the Tabs
     */
    appendFirst(item) {
        this.insertItemAtIndex(item, 0);
    }

    /**
     * Remove an item
     * @param {DOMElement} item - Represents the structure of the Tabs
     */
    removeItem(item) {
        let arr = this.options.layout;

        var index = arr.indexOf(item);
        if (index > -1) {
            arr.splice(index, 1);
        }

        this._show();
    }

    /**
     * Append an item at the beginning
     * @param {DOMElement} item - Represents the structure of the Tabs
     */
    appendLast(item) {
        this.insertItemAtIndex(item, this.options.layout.length);
    }

     /**
     * Append an item at a specific index
     * @param {DOMElement} item - Represents the structure of the Tabs
     * @param {Integer} position - The position of the item to insert
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    insertItemAtIndex(item, index) {
        this.options.layout.splice(index, 0, item);
        this._show();
    }

    /**
     * Empty the layout
     * @param {content: String|DOMElement, swappable: Boolean, disabled: Boolean, selected: Boolean} item - Represents the structure of the Tabs
     */
    empty() {
        // remove all the sub elements
        /*for (let elmt of this.options.layout) {
            elmt.remove();
        }*/

        this.options.layout = [];
        this._show();
    }

    _show() {
        this.remove();
        this.el.innerHTML = "";

        // apply css
        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.layout) {
            for (const item of this.options.layout) {
                if (item) {
                    this.appendContent(item)
                }
            }
        }

        // attach to the DOM again
        this.attachTo(this.target);
    }
}
