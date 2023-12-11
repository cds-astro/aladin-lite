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
import { ContextMenu } from "../Widgets/ContextMenu";
import projectionSvg from '../../../../assets/icons/grid.svg';
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
    constructor(aladin) {
        let ctxMenu = ContextMenu.getInstance(aladin);
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

        ctxMenu.attach(layout);

        super({
            ctxMenu,
            iconURL: projectionSvg,
            tooltip: {content: 'Change the view projection', position: {direction: 'bottom'}},
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                cursor: 'pointer',
            },
        });

        this.addClass('medium-sized-icon')
    }
}
