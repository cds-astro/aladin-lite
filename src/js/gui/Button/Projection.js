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
import { ProjectionEnum } from "../../ProjectionEnum";
import projectionIconUrl from '../../../../assets/icons/projection.svg';
import { ALEvent } from "../../events/ALEvent";
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
        options = options || {};
        options.verbosity = (options && options.verbosity) || 'full';
        let projectionName = aladin.getProjectionName();
        let ctxMenu = _buildLayout(aladin);

        super({
            icon: {
                monochrome: true,
                size: 'medium',
                url: projectionIconUrl,
            },
            classList: ['aladin-projection-control'],
            content: projectionName,
            tooltip: {content: 'Change the view projection', position: {direction: 'bottom left'}},
            ctxMenu,
            ...options
        }, aladin);

        this.aladin = aladin;

        this._addEventListeners()
    }

    _addEventListeners() {
        let aladin = this.aladin;
        let self = this;

        ALEvent.PROJECTION_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let projName = aladin.getProjectionName();
            //let content = self.options.verbosity === 'full' ? ProjectionEnum[projName].label : projName;
            let content = projName;
            self.update({content})
        });
    }
}

function _buildLayout(aladin) {
    let layout = [];

    let aladinProj = aladin.getProjectionName();
    for (const key in ProjectionEnum) {
        let proj = ProjectionEnum[key];

        layout.push({
            label: proj.label,
            selected: aladinProj === key,
            action(o) {
                aladin.setProjection(key)
            }
        })
    }

    return layout;
}
