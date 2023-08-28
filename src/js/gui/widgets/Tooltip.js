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

import './Utils.js';

/* Add a tooltip on a already added Element on the DOM */
export class Tooltip {
    constructor(target, innerHTML) {
        // Creation of the DOM element
        let el = document.createElement('span');
        el.classList.add('aladin-tooltip');

        // Insert it into the DOM tree
        let wrapperEl = document.createElement('div');
        wrapperEl.classList.add('aladin-tooltip-container');

        let targetParent = target.parentNode;
        let targetIndex = Array.prototype.indexOf.call(targetParent.children, target);
        
        let element = targetParent.removeChild(target);

        wrapperEl.appendChild(element);
        wrapperEl.appendChild(el);

        targetParent.insertChildAtIndex(wrapperEl, targetIndex);

        this.el = el;
        this._show(innerHTML)
    }

    _show(innerHTML) {
        this.el.innerHTML = innerHTML;
    }

    attach(innerHTML) {
        this._show(innerHTML)
    }
}
