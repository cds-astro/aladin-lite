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
import { DOMElement } from "../Widgets/Widget";
import { Menu } from "./Menu";
import { ViewPortInfo } from "./ViewPortInfo";
import { HiPSInfo } from "./MainHiPSSelector";
import { Utils } from "../Utils";

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
 * Class representing a Tabs layout
 * @extends DOMElement
 */
 export class Toolbar extends DOMElement {
    /**
     * Create a Tabs layout
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(aladin) {
        let hipsSelector = new HiPSInfo(aladin, "base");
        hipsSelector.addClass("aladin-base-survey");

        let toolbar = Layout.toolbar({  
            layout: [
                new ViewPortInfo(aladin),
                hipsSelector,
            ]
        }, aladin.aladinDiv);

        super(toolbar)

        toolbar.appendLast(new Menu(aladin, this));
    }
}
