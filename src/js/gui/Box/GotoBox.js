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

import { Box } from "../Widgets/Box.js";
import { Input } from "../Widgets/Input.js";
import { Layout } from "../Layout.js";
import { SearchTextInput } from "../Input/InputTextSearch.js";
 
export class GotoBox extends Box {
    // Constructor
    constructor(aladin) {
        /*let content = Layout.horizontal([
            'Go to:',
            Input.text({
                //tooltip: {content: 'Search for a VizieR catalogue', position: {direction :'bottom'}},
                label: "Go to:",
                name: "goto",
                type: "text",
                placeholder: 'Object name/position',
                autocomplete: 'off',
                change(e, self) {
                    self.addEventListener('blur', (event) => {});
                }
            })
        ]);*/
        let textField = new SearchTextInput(aladin, {
            cssStyle: {
                width: '15rem'
            }
        });

        super(aladin, {content: textField, cssStyle: {backgroundColor: 'transparent', padding: '0 0 0 0.2rem'}})

        this.addClass('aladin-box-night');
        this.textField = textField;
    }

    _hide() {
        if (this.textField) {
            this.textField.set('')
        }

        super._hide()
    }
}
