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

import { ALEvent } from "../../events/ALEvent";

import { DOMElement } from "../Widgets/Widget";

import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import { SurveyCtxMenu } from "../CtxMenu/SurveyCtxMenu";

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
 * @extends CtxMenuActionButtonOpener
 */
 export class SurveyInfoActionButton extends CtxMenuActionButtonOpener {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin) {
        super({
            ctxMenu: new SurveyCtxMenu(aladin),
            tooltip: {content: 'Want to change the main survey?', position: { direction: 'bottom' }},
            content: 'Main survey',
            cssStyle: {
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                borderColor: 'white',
                color: 'white',
                padding: '4px',
            },
        })

        this.aladin = aladin;
        /*
        ctxMenu.show({
            e: e,
            position: {
                anchor: el,
                direction: 'bottom',
            },
            
        })
        // add the special brightness enhanced hover effect
        let items = ctxMenu
            .element()
            .querySelectorAll('.aladin-context-menu-item')
        for (let item of items) {
            item.classList.add('aladin-survey-item')
        }
        */

        //super(el)

        this._addListeners()
    }

    _addListeners() {
        ALEvent.HIPS_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, (e) => {
            const layer = e.detail.layer;
            if (layer.layer === 'base') {
                this.update({
                    content: layer.name,
                })
            }
        });
    }
}
