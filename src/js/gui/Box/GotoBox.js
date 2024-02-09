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
        let textField = Input.text({
            label: "Go to:",
            name: "goto",
            type: "text",
            placeholder: 'Object name/position',
            tooltip: {
                global: true,
                aladin,
                content: 'Search for an object name/position'
            },
            //autocapitalize: 'off',
            autocomplete: 'off',
            autofocus: true,
            actions: {
                keydown: (e) => {
                    textField.removeClass('aladin-unknownObject'); // remove red border

                    if (e.key === 'Enter') {
                        let object = textField.get();
                        textField.el.blur();

                        aladin.gotoObject(
                            object,
                            {
                                error: function () {
                                    textField.addClass('aladin-unknownObject');
                                }
                            }
                        );
                    }
                }
            }
        });

        super(aladin, {content: textField})

        this.addClass('aladin-box-night');
        this.textField = textField;
    }

    _hide() {
        if (this.textField) {
            this.textField.set('')
        }

        super._hide()
    }

    static singleton;

    static getInstance(aladin) {
        if (!GotoBox.singleton) {
            GotoBox.singleton = new GotoBox(aladin);
        }

        return GotoBox.singleton;
    }
}
