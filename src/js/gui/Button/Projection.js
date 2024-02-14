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

import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import projectionSvg from '../../../../assets/icons/projection.svg';
import { ProjectionEnum } from "../../ProjectionEnum";
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
 export class ProjectionActionButton extends CtxMenuActionButtonOpener {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        //let ctxMenu = ;
        super({
            iconURL: projectionSvg,
            tooltip: {content: 'Change the view projection', position: {direction: 'top'}},
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                cursor: 'pointer',
            },
            ...options
        }, aladin);

        let ctxMenu = this._buildLayout(aladin);
        this.update({ctxMenu})

        this.addClass('medium-sized-icon')
    }

    _buildLayout(aladin) {
        let layout = [];
        let self = this;

        let aladinProj = aladin.getProjectionName();
        for (const key in ProjectionEnum) {
            let proj = ProjectionEnum[key];
            layout.push({
                label: proj.label,
                selected: aladinProj === key,
                action(o) {
                    aladin.setProjection(key)

                    let ctxMenu = self._buildLayout(aladin);
                    self.update({ctxMenu});
                }
            })
        }

        return layout;
    }

    _show() {
        super._show()
    }
}
