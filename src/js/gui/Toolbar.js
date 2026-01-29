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
import { Layout } from "./Layout";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Widgets/layout/Horizontal.js
 *
 * A layout grouping widgets horizontaly
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class Toolbar extends Layout {
    /**
     * Create a layout
     * @param {layout: Array.<DOMElement | String>} layout - Represents the structure of the Tabs
     * @param {Object} options - Options object
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(widgets, options, target, position = "beforeend") {
        let layout = Object.values(widgets);
        super(
            layout,
            {
                vertical: true,
                ...options
            },
            target,
            position
        )
        
        this.toggled = null;
        let self = this;

        for (let widget of this.layout) {
            const action = widget.options.action;
            widget.update({
                action: (o) => {
                    // toggle off the current toggled widget
                    self._toggleOffWidget(widget)
                    self.toggled = widget;

                    action(o)
                }
            })
        }

        this.widgets = widgets;
    }

    // Close the toggled widget if the user clicks on another one
    _toggleOffWidget(widget) {
        if (this.toggled && this.toggled !== widget) {
            let canBeClosed = this.toggled && this.toggled.close;
            if (canBeClosed) {
                this.toggled.close();
            }

            this.toggled = null;
        }
    }

    has(name) {
        return name in this.widgets;
    }

    enabled(name) {
        if (!this.has(name)) {
            return false;
        }

        let widget = this.widgets[name];
        return widget.el.disabled === false;
    }

    enable(name) {
        if (!this.has(name)) {
            return;
        }

        let widget = this.widgets[name];
        widget.update({disable: false})
    }

    disable(name) {
        if (!this.has(name)) {
            return;
        }

        let widget = this.widgets[name];
        widget.update({disable: true})
    }

    add(name, widget) {
        this.widgets[name] = widget;

        const action = widget.options.action;
        widget.update({
            action: (o) => {
                // toggle off the current toggled widget
                this._toggleOffWidget(widget)
                this.toggled = widget;

                action(o)
            }
        })

        this.appendLast(widget);
    }

    remove(name) {
        let widget = this.widgets[name];

        if (this.toggled === widget)
            this.toggled = null;

        this.removeItem(widget);

        delete this.widgets[name];
        widget.remove()
    }
}