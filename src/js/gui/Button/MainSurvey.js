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

import { ActionButton } from "../Widgets/ActionButton";
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
 export class MainSurveyActionButton extends ActionButton {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        super({
            ...options,
            tooltip: {content: 'Survey name<br/>Click to change it!', position: { direction: 'bottom' }},
            content: 'Main survey',
            cssStyle: {
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                borderColor: 'white',
                color: 'white',
                padding: '4px',
            },
        })

        this._addListeners(aladin)
    }

    _hide() {
        super._hide()
    }

    _addListeners(aladin) {
        ALEvent.HIPS_LAYER_ADDED.listenedBy(aladin.aladinDiv, (e) => {
            const layer = e.detail.layer;
            if (layer.layer === 'base') {
                this.update({
                    content: layer.name,
                })
            }
        });
    }
}
