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

import { Widget } from "../Widget";

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

export class HorizontalLayout extends Widget {
    constructor(widgets, opt, target, position = "beforeend") {
        let el = document.createElement('div');
        el.classList.add('aladin-horizontal-list');

        let node;
        for(let widget of widgets) {
            if (widget instanceof Element) {
                node = widget;
            } else if (widget instanceof Widget) {
                node = widget.element();
            } else {
                const placeholder = document.createElement("div");
                placeholder.innerHTML = widget;
                node = placeholder.firstElementChild;
            }

            el.appendChild(node);
        }

        // add it to the dom
        super(el, opt, target, position);
    }

    _show() {
        super._show();
    }
}
