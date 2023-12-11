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
 import targetIconUrl from '../../../../assets/icons/target.svg';

 /*
 options = {
     action: (connector) => {
 
     }
     tooltip
 }
 */
export class ConeSearchActionButton extends ActionButton {
    // Constructor
    constructor(options, aladin) {
        super({
            iconURL: targetIconUrl,
            tooltip: options.tooltip,
            disable: options.disable,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'pointer',
                ...options.cssStyle
            },
            action(e) {
                if (options.onBeforeClick) {
                    options.onBeforeClick(e);
                }

                aladin.select('circle', c => {
                    options.action(c)
                })
            }
        })

        this.addClass('medium-sized-icon')
    }
}