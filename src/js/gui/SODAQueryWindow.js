// Copyright 2023 - UDS/CNRS
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
 * File gui/SODAQueryWindow.js
 *
 * A form window aiming to query a SODA service cutout
 *
 *
 * Author: Matthieu Baumann [CDS]
 *
 *****************************************************************************/

import { Utils } from '../Utils';
import { ActionButton } from './Widgets/ActionButton.js';
import targetIconImg from '../../../assets/icons/target.svg';
import { Form } from './Widgets/Form.js';
import { Layout } from './Layout.js';

export class SODAQueryWindow {
    constructor(aladin) {
        this.aladin = aladin;
        this.isShowing = false;
    }

    hide() {
        if(this.mainEl) {
            this.mainEl.remove();
        }

        this.isShowing = false;
    }

    show(aladinInstance) {
        this.isShowing = true;

        this.mainEl = document.createElement('div');
        this.mainEl.classList.add('aladin-box', 'aladin-anchor-left');
        this.mainEl.style.display = 'initial';
        this.mainEl.style.width = '230px';

        this.mainEl.innerHTML = '<a class="aladin-closeBtn">&times;</a><div class="aladin-horizontal-list"></div>';


        let self = this;
        for (const key in this.formParams.inputParams) {
            let inputParam = this.formParams.inputParams[key];

            let header;
            if (key === "Circle") {
                const circleSelectBtn = ActionButton.createIconBtn({
                    iconURL: targetIconImg,
                    cssStyle: {
                        backgroundColor: '#bababa',
                        borderColor: '#484848',
                    },
                    tooltip: {content: 'Circular selection<br/><i><font size="-2">Click, drag and release to define the circle</font></i>'},
                    action(e) {
                        self.aladin.select('circle', (s) => {
                            const {x, y, r} = s;
        
                            const [ra, dec] = self.aladin.pix2world(x, y);
                            const dist = self.aladin.angularDist(x, y, x + r, y);
        
                            self.form.set('ra', ra);
                            self.form.set('dec', dec);
                            self.form.set('rad', dist);
                        });
                    }
                });
    
                // Header of the CIRCLE form group
                header = Layout.horizontal({layout: ['<div>Circle</div>', circleSelectBtn]});
            } else {
                header = key;
            }

            inputParam.header = header;
        }

        const listOfInputParams = Object.keys(this.formParams["inputParams"]).map((name) => name).join(', ');

        let infoBtn = ActionButton.createIconBtn({
            content: '📡',
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
            },
            tooltip: {content: 'This is the form to request the SODA server located at: <br/><a target="_blank" href="' + this.formParams["baseUrl"]  + '">' + this.formParams["baseUrl"] + '</a><br/>The list of input params is:<br/>' + listOfInputParams},
            action(e) {}
        });
        let layoutForm = {
            name: 'header',
            type: 'group',
            header: Layout.horizontal({layout: [infoBtn, '<div class="aladin-box-title">Cutout service form</div>']}),
            cssStyle: {
                borderStyle: 'none',
                margin: '0',
            },
            subInputs: []
        };
        for(const key in this.formParams.inputParams) {
            let inputParam = this.formParams.inputParams[key];
            layoutForm.subInputs.push(inputParam);
        }

        this.form = new Form(
            layoutForm,
            this.mainEl,
        );


        // Add the form inputs
        let submitFormDiv = Layout.horizontal({layout: ['<button class="aladin-btn aladin-validBtn" type="submit">Submit</button>', ' <button class="aladin-btn aladin-cancelBtn">Cancel</button>']}, this.mainEl);

        this.aladin.aladinDiv.appendChild(this.mainEl);
        this.mainEl.querySelector(".aladin-closeBtn")
            .addEventListener(
                "click",
                () => { this.hide(); }
            );
        this.mainEl.querySelector(".aladin-cancelBtn")
            .addEventListener(
                "click",
                () => { this.hide(); }
            );

        this.mainEl.querySelector(".aladin-validBtn")
            .addEventListener("click", (e) => {
                e.preventDefault();
                let params = this.form.values();
                // Construct the SODA url
                let url = new URL(this.formParams.baseUrl)
                if (params['ra'] && params['dec'] && params['rad']) {
                    url.searchParams.append('CIRCLE', params['ra'] + ' ' + params['dec'] + ' ' + params['rad']);
                }

                if (params['ramin'] && params['ramax'] && params['decmin'] && params['decmax']) {
                    url.searchParams.append('RANGE', params['ramin'] + ' ' + params['ramax'] + ' ' + params['decmin'] + ' ' + params['decmax']);
                }

                if (params['fmin'] && params['fmax']) {
                    url.searchParams.append('BAND', params['fmin'] + ' ' + params['fmax']);
                }

                if (params['ID']) {
                    url.searchParams.append('ID', params['ID']);
                }

                let loadingBtn = ActionButton.create(
                    ActionButton.DEFAULT_BTN["loading"],
                    'Waiting to get the image response...',
                    submitFormDiv
                );

                let name = url.searchParams.toString();
                // Tackle cors problems
                Utils.loadFromUrls([url, Utils.handleCORSNotSameOrigin(url)], {timeout: 30000})
                    .then((response) => response.blob())
                    .then((blob) => {
                        const url = URL.createObjectURL(blob);
                        try {
                            let image = aladinInstance.createImageFITS(url, name);   
                            aladinInstance.setOverlayImageLayer(image, Utils.uuidv4())
                        } catch(e) {
                            throw('Fail to interpret ' + url + ' as a fits file')
                        }
                    })
                    .catch((e) => {
                        window.alert(e)
                    })
                    .finally(() => {
                        loadingBtn.remove();
                    })
            });
    }

    setParams(params) {
        this.formParams = params;
    }
}