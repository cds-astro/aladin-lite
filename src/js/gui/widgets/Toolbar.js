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
import { Layout } from "../Layout";
import { Utils } from "../../Utils";
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

/*{
    direction: 'vertical' | 'horizontal',
    cssStyle: {...}
    position: {
            top,
            left
        } \ {
            container: NodeElement
            anchor: 'left top' |
                'left center' |
                'left bottom' |
                'right top' |
                'right center' |
                'right bottom' |
                'center top' |
                'center center' |
                'center bottom'
        }
    }
}*/
export class Toolbar extends Layout {
    /**
     * Create a layout
     * @param {layout: Array.<DOMElement | String>, cssStyle: Object} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options) {
        options.direction = options.direction || 'horizontal';
        options.position = options.position || {anchor: 'left top'};
        options.layout = options.layout || [];

        super(options)

        let direction = options.direction;
        if (direction === 'vertical') {
            this.addClass('aladin-vertical-list')
        } else {
            this.addClass('aladin-horizontal-list')
        }

        this.tools = {};
    }

    add(tools) {
        if (!Array.isArray(tools)) {
            tools = [tools];
        }

        tools.forEach(tool => {
            this.appendAtLast(tool)
        });
    }

    appendAtLast(tool, name) {
        if (!name) {
            name = Utils.uuidv4()
        }
        this.tools[name] = tool;
        this.appendLast(tool);
    }

    appendAtIndex(tool, name, index) {
        this.tools[name] = tool;
        this.insertItemAtIndex(tool, index);
    }

    /* Show a tool */
    show(name) {
        if (name && this.tools[name]) {
            this.tools[name]._show()
        }
    }

    isShown(name) {
        return this.tools[name] && !this.tools[name].isHidden
    }

    /* Hide a tool */
    hide(name) {
        if (name && this.tools[name]) {
            this.tools[name]._hide()
        }
    }

    contains(name) {
        return this.tools[name] !== undefined;
    }
}
