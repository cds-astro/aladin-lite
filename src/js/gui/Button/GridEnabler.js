// Copyright 2013 - UDS/CNRS
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


/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { ActionButton } from "../Widgets/ActionButton.js";
import gridIcon from './../../../../assets/icons/grid.svg';

export class GridEnabler extends ActionButton {
    // Constructor
    constructor(aladin) {
        const computeTooltip = (enabled) => {
            const content = enabled ? 'Hide the coordinate grid' : 'Display the coordinate grid'
            return {
                content,
                position: {
                    direction: 'top right'
                }
            }
        }

        let gridEnabled = aladin.getGridOptions().enabled;
        let self;
        super({
            icon: {
                size: 'medium',
                monochrome: true,
                url: gridIcon
            },
            classList: ['aladin-grid-control'],
            tooltip: computeTooltip(gridEnabled),
            toggled: gridEnabled,
            action(o) {
                const isGridEnabled = aladin.getGridOptions().enabled;
                const enabled = !isGridEnabled;
                aladin.setCooGrid({enabled})

                self.update({toggled: enabled, tooltip: computeTooltip(enabled)})

                if (aladin.statusBar) {
                    aladin.statusBar.removeMessage('grid')

                    if (enabled) {
                        aladin.statusBar.appendMessage({
                            id: 'grid',
                            message: 'Grid enabled!',
                            duration: 2000,
                            type: 'info'
                        })
                    }
                }
            }
        })
        self = this;
    }
}