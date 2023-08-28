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
import { Widget } from "./Widget";

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

export class ActionButton extends Widget {
    constructor(target, opt, position = "beforeend") {
        let el = document.createElement('button');
        el.classList.add('aladin-btn', 'aladin-24px-icon');

        // add it to the dom
        super(el, target, opt, position);

        // add a tooltip on it
        this.tooltip = new Tooltip(this.el, this.opt.info);
    }

    _show() {
        this.el.removeEventListener('click', this.action);

        this.action = (e) => {
            e.stopPropagation();
            e.preventDefault();

            this.opt.action(e);
        };
        this.el.addEventListener('click', this.action);

        if (this.opt.iconURL) {
            let img = document.createElement('img');
            img.src = this.opt.iconURL;
            img.style.objectFit = 'contain';

            this.el.appendChild(img);
        }

        if (this.opt.content) {
            this.el.textContent = this.opt.content;
        }

        if (this.opt.disable) {
            this.el.disabled = true;
        } else {
            this.el.disabled = false;
        }

        if (this.tooltip)
            this.tooltip.attach(this.opt.info);

        super._show();
    }
}


