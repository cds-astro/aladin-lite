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

import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import skewerSelectionIconArrow from '../../../../assets/icons/skewer_selection-arrow.svg';
import skewerSelectionIcon from '../../../../assets/icons/skewer_selection_black.svg';
import edgeSelectionIconArrow from '../../../../assets/icons/edge_selection-arrow.svg';
import edgeSelectionIcon from '../../../../assets/icons/edge_selection.svg';
import { View } from "../../View.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Button/SelectionMode.js
 *
 * Class representing a button for bringing up a menu for choosing selection mode.
 * The appearance of the button changes depending on which selection mode (View.getSelectionMode())
 * is active.
 *
 * There are two possible selection modes, Edge and Skewer, which affect how footprints are
 * interactively selected.
 *
 * In Edge mode (View.SELECTION_MODE_EDGE), footprints are selected by clicking on their edges.
 *
 * In Skewer mode (View.SELECTION_MODE_SKEWER), footprints are selecting by clicking anywhere
 * inside the footprint.
 *
 * Using a modifier key (Cmd on Mac, Ctrl otherwise) during select toggles the potential selections:
 * - If any of the potential selections are not already selected, those objects are added to the current selections.
 * - If all of the potential selections are already selected, then they are deselected.
 *
 * This uses the CSS class aladin-selectionMode-control.
 *
 * Author: Tom Donaldson (STScI)
 *
 *****************************************************************************/
 export class SelectionMode extends CtxMenuActionButtonOpener {
    /**
     * Class representing a button for bringing up a menu for choosing selection mode.
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {

        // If we're on Mac, the modifier key will be Cmd instead of Ctrl.
        let modifierKey = 'Ctrl';
        const userAgent = window.navigator.userAgent.toLowerCase();
        if (userAgent.indexOf('mac') > -1) {
            modifierKey = 'Cmd';
        }

        // Set the initial button icon based on the current View selection mode.
        const initialMode = aladin.view.getSelectionMode();
        let initialIcon = edgeSelectionIconArrow;
        if (initialMode === View.SELECTION_MODE_SKEWER) {
            initialIcon = skewerSelectionIconArrow;
        }

        super({
            icon: {
                size: 'medium',
                monochrome: true,
                url: initialIcon,
            },
            classList: ['aladin-selectionMode-control'],
            tooltip: {
                content: 'Choose the selection mode<br />(' + modifierKey + ' for multiselect)',
                position: { direction: 'top right', top: '10%', left: '80%' },
            },
            ctxMenu: undefined,
            ...options
        }, aladin);

        this.aladin = aladin;
        this.modifierKey = modifierKey;
        let ctxMenu = this._buildLayout()
        this.update({ctxMenu})
    }

    setCustomIcon(icon) {
        this.update({icon: {
                size: 'medium',
                monochrome: true,
                url: icon
            }})
    }

    _buildLayout() {
        let self = this;
        let aladin = this.aladin;

        return [
            {
                label: {
                    icon: {
                        url: skewerSelectionIcon,
                        monochrome: true,
                    },
                    tooltip: {
                        content: 'Click inside shapes to select.<br />Multiselect with ' + self.modifierKey + '.',
                        position: { direction: 'top right', left: '50%' },
                    },
                    content: "Skewer Selection",
                },
                action: (e) => {
                    aladin.view.setSelectionMode(View.SELECTION_MODE_SKEWER);
                    self.setCustomIcon(skewerSelectionIconArrow);
                },
            },
            {
                label: {
                    icon: {
                        url: edgeSelectionIcon,
                        monochrome: true,
                    },
                    tooltip: {
                        content: 'Click on objects to select.<br />Multiselect with ' + self.modifierKey + '.',
                        position: { direction: 'top right', left: '60%' },
                    },
                    content: "Edge Selection",
                },
                action: (e) => {
                    aladin.view.setSelectionMode(View.SELECTION_MODE_EDGE);
                    self.setCustomIcon(edgeSelectionIconArrow);
                },
            },
        ]
    }

}

