// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//

/******************************************************************************
 * Aladin Lite project
 *
 * File GridSettingsCtxMenu
 *
 * Author: Matthieu Baumann [CDS]
 *
 *****************************************************************************/

import { ProjectionEnum } from "../../ProjectionEnum";

export let ProjectionCtxMenu = (function () {

    let ProjectionCtxMenu = {};

    ProjectionCtxMenu.getLayout = function (aladin) {

        /*ALEvent.COO_GRID_UPDATED.listenedBy(aladin.aladinDiv, function (e) {
            let color = e.detail.color;

            let hexColor = Color.rgbToHex(Math.round(255 * color.r), Math.round(255 * color.g), Math.round(255 * color.b));
            colorInput.set(hexColor)
        });*/
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

        return {
            label: {
                content: 'Projection',
                tooltip: { content: ProjectionEnum[aladinProj].label + ' selected', position: { direction: 'bottom'} }
            },
            subMenu: layout
        };
    }

    return ProjectionCtxMenu;

})();
