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

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { ALEvent } from "../../events/ALEvent.js";
import { Icon } from "../Widgets/Icon.js";
import infoIconUrl from '../../../../assets/icons/info.svg';

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

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
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

import { Input } from "../Widgets/Input.js";
 
export class Dropdown extends Input {
     // constructor
     constructor(aladin, options) {
        let self;
        aladin.view.catalogCanvas.addEventListener('click', (e) => {
            self.el.blur();
        });

        options.autocomplete = {options: options.options || []};
        delete options.options;

        super({
            type: 'text',
            actions: {
                focus(_) {
                    self.removeClass('aladin-valid')
                    self.removeClass('aladin-not-valid')
                },
                dblclick(e) {
                    self.set('')

                    if (self.options.input) {
                        self.options.input(e)
                    }
                },
                input(e) {
                    self.removeClass('aladin-valid')
                    self.removeClass('aladin-not-valid')

                    if (e.data === undefined) {
                        // select
                        self.options.action(e)

                        let value = e.target.value;
                        self.set('');
                        
                        if (self.options.input) {
                            self.options.input(e)
                        }

                        self.set(value)
                    } else {
                        if (self.options.input) {
                            self.options.input(e)
                        }
                    }
                },
                keydown(e) {
                    if (!e.key) {
                        return;
                    }

                    e.stopPropagation();
                    // ignore navigation keys
                    if (e.key === 'Enter') {
                        self.options.action(e)
                    }
                },
            },
            ...options
        })
        this.el.classList.add('search')

        self = this;
    }

    update(options) {
        if (options.options) {
            options.autocomplete = {options: options.options || []};
            delete options.options;
        }

        super.update(options)
    }
};