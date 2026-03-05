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
import { Layout } from "./Layout";
import { ActionButton } from "./Widgets/ActionButton";
import { DOMElement } from "./Widgets/Widget";
/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Toolbar.js
 *
 * A layout grouping widgets horizontaly
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class Toolbar extends Layout {
    /**
     * Toolbar instance constructor
     *
     * @class
     * @constructs Toolbar
     * @param {Object} options - Options object
     * @param {DOMElement} target - The parent element.
     */
    constructor(options, target) {
        let position = (options && options.position) || 'topleft';
        delete options.position;

        super(
            [],
            options,
            target,
        )

        this.position = position;
        this.vertical = options && options.vertical === true;

        this.toggled = null;
        this.widgets = {};
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
        widget.update({disabled: false})
    }

    disable(name) {
        if (!this.has(name)) {
            return;
        }

        let widget = this.widgets[name];
        widget.update({disabled: true})
    }

    /**
     * Add custom clickable actions to the toolbar
     *
     * @memberof Toolbar
     * @param {string} name - A name identifier for your new action
     * @param {Object} widget - Your clickable action, same as for defining an action button
     * @param {boolean} [widget.toggled=false] - Whether the button is initially toggled.
     * @param {function} [widget.action] - The callback function to execute when the button is clicked.
     * @param {string} [widget.title] - The title attribute for the button.
     * @param {Object} [widget.icon] - An icon object for the button.
     * @param {boolean} [widget.disabled=false] - Whether the button is initially disabled.
     * @param {CSSStyleSheet} [widget.cssStyle] - The CSS styles to apply to the button.
     * @param {Object} [widget.tooltip] - A tooltip appearing when the user hovers the button.
     * @param {string} [widget.size] - The size of the button. Can be 'medium' or 'small'
     *
     * @example
     *   aladin.getToolbar()
     *       .add('action_custom', {
     *           icon: {
     *              url: yourIconUrl,
     *               size: "medium"
     *           },
     *           action(_) {
     *               console.log("do a thing")
     *           }
     *       });
     */
    add(name, widget) {
        if (!(widget instanceof DOMElement)) {
            widget = new ActionButton(widget)
        }

        switch (this.position) {
            case 'topleft':
                widget.update({openDirection: 'right'})
                this.update({position: {
                    anchor: 'left top'
                }})
                break;
            case 'topright':
                widget.update({openDirection: 'left'})
                this.update({position: {
                    anchor: 'right top'
                }})
                break;
            case 'bottomleft':
                widget.update({openDirection: 'top'})
                this.update({position: {
                    anchor: 'left bottom'
                }})
                break;
            case 'bottomright':
                widget.update({openDirection: 'top right'})
                this.update({position: {
                    anchor: 'right bottom'
                }})
                break;
            default:
                break;
        }

        const action = widget.options.action;
        widget.update({
            action: (o) => {
                // toggle off the current toggled widget
                this._toggleOffWidget(widget)
                this.toggled = widget;

                action(o)
            }
        })

        this.widgets[name] = widget;

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