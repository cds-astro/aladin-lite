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

import { Tooltip } from "./Tooltip";
import { SAMPActionButton } from '../Button/SAMP.js';

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Tab.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/
import { DOMElement } from "./Widget";
import { Layout } from "../Layout";
/**
 * Class representing a Tabs layout
 * @extends DOMElement
 */
export class Tabs extends DOMElement {
    /**
     * Create a Tabs layout
     * @param {{layout: Array.<{cssStyle: Object, label: String|DOMElement, title: String, content: String|DOMElement}>, cssStyle: Object}} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, target, position = "beforeend") {
        let contentTabOptions = [];
        let tabsLayout = [];

        for (const tab of options.layout) {
            // Create the content tab div
            let contentTabOptionEl = document.createElement("div");
            contentTabOptionEl.style.display = 'none';

            if (tab.content instanceof DOMElement) {
                // And add it to the DOM
                tab.content.attachTo(contentTabOptionEl);
            } else if (opt.label instanceof Element) {                
                contentTabOptionEl.insertAdjacentElement('beforeend', tab.content);
            } else {
                let wrapEl = document.createElement('div');
                wrapEl.innerHTML = tab.content;
                contentTabOptionEl.insertAdjacentElement('beforeend', wrapEl);
            }

            contentTabOptions.push(contentTabOptionEl);

            // Create the Tab element
            tab.label.update({
                action(e) {
                    e.stopPropagation();
                    e.preventDefault();
                    
                    for (let contentTabOptionEl of contentTabOptions) {
                        contentTabOptionEl.style.display = 'none';
                    }

                    contentTabOptionEl.style.display = 'block';
                    for (const t of options.layout) {
                        t.label.update({toggled: false});
                    }

                    tab.label.update({toggled: true})
                },
            });
            tab.label.addClass('tab')

            tabsLayout.push(tab.label);
        }

        if (options.aladin && options.aladin.samp) {
            tabsLayout.push(SAMPActionButton.sendSources(options.aladin))
        }

        let contentTabEl = document.createElement("div");
        contentTabEl.style.maxWidth = '100%';

        contentTabOptions[0].style.display = 'block';
        tabsLayout[0].update({toggled: true})
        for(let contentTabOptionEl of contentTabOptions) {
            // Add it to the view
            contentTabEl.appendChild(contentTabOptionEl)
        }

        let el = new Layout([
            new Layout({layout: tabsLayout, orientation: 'horizontal'}),
            contentTabEl
        ]);

        super(el, options);
        this._show();

        this.attachTo(target, position);
    }

    _show() {
        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle)
        }

        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        super._show();
    }
}
