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

import { DOMElement } from "./Widgets/Widget";
import { Tooltip } from "./Widgets/Tooltip";
import { isJSObject } from "./Utils";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Layout.js
 *
 * A layout grouping widgets horizontaly
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class Layout extends DOMElement {
    constructor(layout, options, target, position = "beforeend") {
        let el = document.createElement('div');

        layout = layout || [];
        super(el, options);

        this.layout = layout;

        if (options && options.cssStyle) {
            this.setCss(options.cssStyle);
        }

        // 1. Attach self to the target
        this.attachTo(target, position);

        // 2. Once self is attached, attach the children
            if (typeof layout === 'string' || layout instanceof String) {
                this.el.innerHTML = layout;
            // otherwise it is an object
            } else if (
                isJSObject(layout)
            ) {
                if (layout.start) {
                    this.appendContent(new Layout(layout.start));
                }

                if (layout.end) {
                    this.appendContent(new Layout(layout.end));
                }

                this.el.style.justifyContent = "space-between";
            } else if (Array.isArray(layout)) {
                // treat it as an array
                for (let item of layout) {
                    if (Array.isArray(item) || isJSObject(item)) {
                        item = new Layout(item)
                    }
                    this.appendContent(item)
                }
            } else {
                const item = layout;
                this.appendContent(item)
            }

            if (options && options.draggable) {
                // retrieve the children and add the drag listeners
                let draggableFn = options.draggable;
                let firstSelected = null;

                this.el.childNodes.forEach(div => {
                    div.addEventListener("click", () => {
                        // If nothing selected yet → select this one
                        if (!firstSelected) {
                            firstSelected = div;
                            div.classList.add("aladin-item-selected");
                            return;
                        }

                        // If clicking the same one again → unselect
                        if (firstSelected === div) {
                            div.classList.remove("aladin-item-selected");
                            firstSelected = null;
                            return;
                        }

                        // Otherwise: swap the two elements
                        let a = firstSelected;
                        let b = div;

                        let temp = document.createElement("div");
                        a.parentNode.insertBefore(temp, a);
                        b.parentNode.insertBefore(a, b);
                        temp.parentNode.insertBefore(b, temp);
                        temp.remove();

                        // Exec callback
                        draggableFn(a, b)

                        // Clear selection
                        a.classList.remove("aladin-item-selected");
                        firstSelected = null;
                    });
                });
            }

        // The tooltip has to be set once the element
        // lies in the DOM
        if (options && options.tooltip) {
            Tooltip.add(options.tooltip, this)
        }

        if (options && options.position) {
            this.setPosition(options.position)
        }

        if (options && options.vertical && options.vertical === true) {
            this.addClass('aladin-vertical-list')
        } else {
            this.addClass('aladin-horizontal-list')
        }

        if (options && options.classList) {
            this.addClass(options.classList)
        }
    }

    static horizontal(layout, options, target, position = "beforeend") {
        return new Layout(layout, options, target, position);
    }

    static nested(layout, options, target, position = "beforeend") {
        let horizontalLayout = new Layout(layout, options, target, position);
        horizontalLayout.removeClass('aladin-horizontal-list');

        return horizontalLayout;
    }

    static vertical(layout, options, target, position = "beforeend") {
        let verticalLayout = new Layout(layout, {...options, vertical: true}, target, position);
        verticalLayout.addClass('aladin-vertical-list');

        return verticalLayout;
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
        let arr = this.layout;

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
        this.insertItemAtIndex(item, this.layout.length);
    }

     /**
     * Append an item at a specific index
     * @param {DOMElement} item - Represents the structure of the Tabs
     * @param {Integer} position - The position of the item to insert
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    insertItemAtIndex(item, index) {
        this.layout.splice(index, 0, item);
        this._show();
    }

    empty() {
        // remove all the sub elements
        this.layout = [];
        this._show();
    }

    _show() {
        //this.remove();
        this.el.innerHTML = "";

        // apply css
        if (this.options && this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.layout) {
            for (const item of this.layout) {
                if (item) {
                    this.appendContent(item)
                }
            }
        }

        if (this.options && this.options.position) {
            this.setPosition(this.options.position)
        }
    }
}