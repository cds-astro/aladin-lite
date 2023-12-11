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
 import uploadIconUrl from '../../../../assets/icons/upload.svg';

 /*
 options = {
     action: (connector) => {
 
     }
     tooltip
 }
 */
export class FileLoaderActionButton extends ActionButton {
    // Constructor
    constructor(options) {
        super({
            iconURL: uploadIconUrl,
            tooltip: options.tooltip,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'help',
                ...options.cssStyle
            },
            action(e) {
                let fileLoader = document.createElement('input');
                fileLoader.type = 'file';
                // Case: The user is loading a FITS file
        
                fileLoader.addEventListener("change", (e) => {    
                    let file = e.target.files[0];
        
                    if (options.action) {
                        options.action(file)
                    }
                });
        
                fileLoader.click();
            }
        })

        this.addClass('medium-sized-icon')
    }
}