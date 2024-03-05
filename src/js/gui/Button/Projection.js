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
        super({
            icon: {
                monochrome: true,
                size: 'small',
                url: projectionIconUrl,
            },
            content: [options.verbosity === 'full' ? ProjectionEnum[projectionName].label : ''],
            tooltip: {content: 'Change the view projection', position: {direction: 'bottom left'}},
            cssStyle: {
                cursor: 'pointer',
            },
            ...options
        }, aladin);

        this.aladin = aladin;

        let ctxMenu = this._buildLayout();
        this.update({ctxMenu})

        this._addEventListeners()
    }

    _addEventListeners() {
        let aladin = this.aladin;
        let self = this;

        ALEvent.PROJECTION_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let projName = aladin.getProjectionName();
            let content = self.options.verbosity === 'full' ? ProjectionEnum[projName].label : '';

            self.update({content})
        });
    }

    _buildLayout() {
        let aladin = this.aladin;

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
                    self.update({ctxMenu, content: self.options.verbosity === 'full' ? proj.label : ''});
                }
            })
        }

        return layout;
    }

    update(options) {
        super.update(options);

        if (options.verbosity) {
            let ctxMenu = this._buildLayout();
            let projName = this.aladin.getProjectionName();
            let label = options.verbosity === 'full' ? ProjectionEnum[projName].label : '';
            super.update({ctxMenu, content: label});
        }
    }

    _show() {
        super._show()
    }
}
