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
 * File gui/Box/ShortLivedBox.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { Box } from "../Widgets/Box.js";

export class ShortLivedBox extends Box {
    // Constructor
    constructor(aladin) {
        super(
            {
                close: false,
                cssStyle: {
                    color: 'white',
                    backgroundColor: 'black',
                    borderRadius: '3px',
                    padding: 0,
                }
            },
            aladin.aladinDiv
        )
    }

    _show(options) {
        let duration = options.duration || 1000;
        
        // clear the past timeout
        if (this.idxTimeout) {
            clearTimeout(this.idxTimeout)
        }

        if (duration !== 'unlimited') {
            this.idxTimeout = setTimeout(() => {
                this._hide()
            }, duration)
        }

        super._show(options);
    }
}
