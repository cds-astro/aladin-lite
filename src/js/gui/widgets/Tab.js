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
import { Utils } from "../Utils";

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
        let layout = options.layout;
        let cssStyle = options.cssStyle;

        let el = document.createElement("div");
        el.classList.add('aladin-tabs');

        let headerTabEl = document.createElement("div");
        headerTabEl.classList.add('aladin-tabs-head');
        let contentTabEl = document.createElement("div");
        contentTabEl.classList.add('aladin-tabs-content');
        contentTabEl.style.maxWidth = '100%';

        let contentTabOptions = [];
        let tabsEl = [];
        for (const tab of layout) {
            // Create the content tab div
            let contentTabOptionEl = document.createElement("div");
            contentTabOptionEl.classList.add('aladin-tabs-content-option');
            contentTabOptionEl.style.display = 'none';

            Utils.appendTo(tab.content, contentTabOptionEl);

            contentTabOptions.push(contentTabOptionEl);

            // Create the Tab element
            let tabEl = document.createElement('div');
            tabEl.className = 'aladin-tabs-head-tab';
            if (tab.title) {
                tabEl.title = tab.title;
            }

            // Apply the css to the tab
            for (const property in tab.cssStyle) {
                tabEl.style[property] = tab.cssStyle[property];
            }

            tabsEl.push(tabEl);

            tabEl.addEventListener('click', (e) => {
                e.stopPropagation();
                e.preventDefault();
                
                for (let contentTabOptionEl of contentTabOptions) {
                    contentTabOptionEl.style.display = 'none';
                    contentTabOptionEl.classList.remove('aladin-tabs-content-option-selected')
                }

                contentTabOptionEl.style.display = 'block';
                contentTabOptionEl.classList.add('aladin-tabs-content-option-selected')

                for(let t of tabsEl) {
                    t.classList.remove('aladin-tabs-head-tab-selected')
                }

                tabEl.classList.add('aladin-tabs-head-tab-selected')
            });

            Utils.appendTo(tab.label, tabEl);
            Utils.appendTo(tabEl, headerTabEl);

            if (tab.info) {
                new Tooltip({content: tab.info}, tabEl);
            }
        }

        contentTabOptions[0].style.display = 'block';
        contentTabOptions[0].classList.add('aladin-tabs-content-option-selected');

        tabsEl[0].classList.add('aladin-tabs-head-tab-selected')

        for(let contentTabOptionEl of contentTabOptions) {
            // Add it to the view
            Utils.appendTo(contentTabOptionEl, contentTabEl);
        }

        Utils.appendTo(headerTabEl, el);
        Utils.appendTo(contentTabEl, el);

        super(el, options);

        this.setCss(cssStyle)
        this.attachTo(target, position);

        this._show();
    }

    _show() {
        super._show();
    }
}
