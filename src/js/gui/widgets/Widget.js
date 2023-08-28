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

export class Widget {
    constructor(el, target, opt, position = 'beforeend') {
        this.opt = opt;
        this.el = el;

        target.insertAdjacentElement(position, el);

        this._show();
    }

    _show() {
        // CSS style elements
        for (const property in this.opt) {
            this.el.style[property] = this.opt[property];
        }
    }

    attach(opt) {
        this.opt = {...this.opt, ...opt};

        this._show();
    }
}
