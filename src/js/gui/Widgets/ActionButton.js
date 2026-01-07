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
import { Icon } from "./Icon";
import { Layout } from "../Layout";

import infoIconUrl from "../../../../assets/icons/info.svg"
import targetIconUrl from "../../../../assets/icons/target.svg";
import removeIconUrl from "../../../../assets/icons/remove.svg";

import A from "../../A";
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
 * @property {Object} [opt.icon] - An icon object for the button.
 * @property {boolean} [opt.disabled=false] - Whether the button is initially disabled.
 * @property {HTMLElement|string|Widget} [opt.content] - The content to be added to the button.
 * @property {CSSStyleSheet} [opt.cssStyle] - The CSS styles to apply to the button.
 * @property {Object} [opt.tooltip] - A tooltip.
 * @property {Object|string} [opt.position] - The position of the button.
 * @property {string} [opt.size] - The size of the button. Can be 'medium' or 'small'

 *   - If an object:
 *     - `{ nextTo: DOMElement, direction: 'left' | 'right' | 'top' | 'bottom' }`
 *     - `{ top: number, left: number }`
 *     - `{ anchor: 'left top' | 'left center' | 'left bottom' | 'right top' | 'right center' | 'right bottom' | 'center top' | 'center center' | 'center bottom' }`
 *   - If a string: One of the following values: "beforebegin", "afterbegin", "beforeend", "afterend".
 *
 * @example
 * const actionButton = new ActionButton({
    size: 'small',
    content: '❌',
    //tooltip: {content: 'Close the window', position: {direction: 'bottom'}},
    action(e) {
        self._hide();
    },
    cssStyle: {
        position: 'absolute',
    },
    position: {
        top: 0,
        right: 0,
    }
});
 */
export class ActionButton extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement('button');
        el.classList.add('aladin-btn');

        // add it to the dom
        super(el, options);
        this._show();

        this.attachTo(target, position)
    }

    _show() {
        this.el.innerHTML = '';
        this.el.removeEventListener('click', this.action);

        if (this.options.toggled === true) {
            this.addClass('toggled');
        } else if (this.options.toggled === false) {
            this.removeClass('toggled');
        }

        if (this.options.size === 'small') {
            this.addClass('aladin-small-sized-icon')
        } else if (this.options.size === 'medium') {
            this.addClass('aladin-medium-sized-icon')
        }

        if (this.options.action) {
            this.action = (e) => {
                e.stopPropagation();
                e.preventDefault();
    
                this.options.action(e, this);
            };

            this.el.addEventListener('click', this.action);
        }

        if (this.options.title) {
            this.el.setAttribute('title', this.options.title);
        }

        let layout = [];
        if (this.options.icon) {
            layout.push(new Icon(this.options.icon));
        }

        if (this.options.disabled) {
            this.el.disabled = true;
            this.addClass('disabled')
        } else {
            this.el.disabled = false;
            this.removeClass('disabled')
        }

        // Add the content to the dom
        // Content can be a DOM element, just plain text or another Widget instance
        if (this.options.content) {
            layout.push(this.options.content);
        }

        if (layout.length > 0) {
            if (layout.length === 1) {
                this.appendContent(layout[0])
            } else {
                this.appendContent(new Layout(layout))
            }
        }

        // trigger el added
        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.position) {
            this.setPosition(this.options.position)
        }

        if (this.options.classList) {
            this.element().classList.add(this.options.classList)
        }

        super._show();
    }

    click() {
        this.el.click()
    }

    static createIconBtn(opt, target, position = 'beforeend') {
        let btn = new ActionButton({...opt, size: 'medium'}, target, position);

        return btn;
    }

    static createSmallSizedIconBtn(opt, target, position = 'beforeend') {
        let btn = new ActionButton({...opt, size: 'small'}, target, position);

        return btn;
    }

    static create(opt, info, target, position = 'beforeend') {
        opt['info'] = info || undefined;

        return new ActionButton(opt, target, position);
    }

    static BUTTONS(aladin) {
        return {
            infoHiPS: (options) => {
                return new ActionButton({
                    icon: {
                        size: 'small',
                        monochrome: true,
                        url: infoIconUrl,
                    },
                    tooltip: {
                        position: {direction: "top"},
                        content: "More about that survey?"
                    },
                    action(e) {
                        window.open(options && options.url);
                    },
                    ...options
                })
            },
            targetHiPSLocation: (options) => {
                let ra = options && options.ra;
                let dec = options && options.dec;
                let fov = options && options.fov;
                return new ActionButton({
                    icon: {
                        size: 'small',
                        monochrome: true,
                        url: targetIconUrl,
                    },
                    disabled: ra === undefined || dec === undefined || fov === undefined,
                    tooltip: {
                        content: "Target interesting sky location",
                    },
                    action(e) {
                        if (fov !== undefined && ra !== undefined && dec !== undefined) {
                            aladin.setFoV(+fov)
                            aladin.gotoObject(ra + ' ' + dec);
                        }
                        
                    },
                    ...options
                })
            },
            addMOC: (options) => {
                let name = options && options.name;
                let url = options && options.url;

                let button = new ActionButton({
                    size: "small",
                    icon: {
                        url: Icon.dataURLFromSVG({ svg: Icon.SVG_ICONS.MOC }),
                        size: "small",
                        monochrome: true,
                    },
                    tooltip: {
                        content: "Add coverage",
                        position: { direction: "top" },
                    },
                    action: (e) => {
                        // load the moc
                        let moc = A.MOCFromURL(
                            url,
                            { name },
                            () => {
                                if (aladin.statusBar) {
                                    aladin.statusBar.appendMessage({
                                        message:
                                            "Coverage of " +
                                            name +
                                            " loaded",
                                        duration: 2000,
                                        type: "info",
                                    });
                                }
                            }
                        );

                        aladin.addMOC(moc);
                    },
                    ...options
                })

                return button;
            },
            remove: (action) => {
                return new ActionButton({
                    icon: {
                        url: removeIconUrl,
                        monochrome: true,
                    },
                    size: "small",
                    tooltip: {
                        content: "Remove",
                    },
                    action
                })
            },
            close: (widget) => {
                return new ActionButton({
                    size: 'small',
                    content: '❌',
                    action(_) {
                        widget.close();
                    },
                    cssStyle: {
                        position: 'absolute',
                        top: 0,
                        right: 0,
                    },
                });
            },
        }
    }
}
