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

import { Box } from "../../Widgets/Box.js";
import { Form } from "../../Widgets/Form.js";
 
export class GotoBox extends Box {
    // Constructor
    constructor(aladin, parent) {
        let form = new Form({
            name: 'header',
            type: 'group',
            cssStyle: {
                borderStyle: 'none',
                margin: '0',
                width: '14em',
            },
            submit(e) {
                let gotoInput = form.getInput('goto');

                aladin.gotoObject(
                    gotoInput.value,
                    {
                        error: function () {
                            gotoInput.classList.add('aladin-unknownObject');
                        }
                    }
                );
            },
            subInputs: [{
                label: "Go to:",
                name: "goto",
                type: "text",
                placeholder: 'Object name/position',
                actions: {
                    'change': (e, input) => {
                        input.addEventListener('blur', (event) => {});
                    },
                    'keydown': (e, input) => {
                        input.classList.remove('aladin-unknownObject'); // remove red border
                    }
                }
            }]
        });

        super({
            content: form,
            position: {
                anchor: parent,
                direction: 'bottom',
            }
        }, aladin.aladinDiv)
    }

    static singleton;

    static getInstance(aladin, parent) {
        if (!GotoBox.singleton) {
            GotoBox.singleton = new GotoBox(aladin, parent);
        }

        return GotoBox.singleton;
    }
}
