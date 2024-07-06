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

import { Utils } from '../../Utils';
import { ActionButton } from '../Widgets/ActionButton.js';
import { Form } from '../Widgets/Form.js';
import { Layout } from '../Layout.js';
import { Box } from '../Widgets/Box';
import { ConeSearchActionButton } from '../Button/ConeSearch';
import { Coo } from '../../libs/astro/coo';
import { Angle } from '../../libs/astro/angle';

export class ServiceQueryBox extends Box {
    constructor(aladin) {
        let self;
        // Define the form once for all
        let form = new Form({
            submit: {
                action(params) {
                    // Construct the SODA url
                    let url = new URL(self.service.baseUrl)

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

                    /*let loadingBtn = ActionButton.create(
                        ActionButton.DEFAULT_BTN["loading"],
                        'Waiting to get the image response...',
                        submitFormDiv
                    );*/

                    let name = url.searchParams.toString();
                    // Tackle CORS problems
                    Utils.loadFromUrls([url, Utils.handleCORSNotSameOrigin(url)], {timeout: 30000, dataType: 'blob'})
                        .then((blob) => {
                            const url = URL.createObjectURL(blob);

                            try {
                                let image = self.aladin.createImageFITS(url, {name});   
                                self.aladin.setOverlayImageLayer(image, Utils.uuidv4())
                            } catch(e) {
                                throw('Fail to interpret ' + url + ' as a fits file')
                            }
                        })
                        .catch((e) => {
                            window.alert(e)
                        })
                }
            },
            subInputs: []
        });

        super(
            {
                header: {
                    draggable: true,
                    title: 'Service query'
                },
                content: form
            },
            aladin.aladinDiv
        )

        this.form = form;
        this.aladin = aladin;

        self = this;
    }

    attach(service) {
        this.service = service;
        let self = this;

        let subInputs = []
        for (const param in this.service.inputParams) {

            let subInput = this.service.inputParams[param];

            let header;
            switch (param) {
                case 'ID':
                    const listOfInputParams = Object.keys(this.service["inputParams"]).map((name) => name).join(', ');

                    header = Layout.horizontal(['ID', new ActionButton({
                        size: 'small',
                        content: '📡',
                        tooltip: {
                            content: 'This is the form to request the SODA server located at: <br/>' + 
                                '<a target="_blank" href="' + this.service["baseUrl"]  + '">' + this.service["baseUrl"] + '</a><br/>' +
                                'The list of input params is:<br/>' + listOfInputParams
                        },
                    })]);
                    break;
                case 'Circle':
                    let csBtn = new ConeSearchActionButton({
                        tooltip: {content: 'Cone selection', position: {direction: 'left'}},
                        onBeforeClick(e) {
                            self._hide();
                        },
                        action(c) {
                            // convert to ra, dec and radius in deg
                            try {
                                let [ra, dec] = self.aladin.pix2world(c.x, c.y);
                                let radius = self.aladin.angularDist(c.x, c.y, c.x + c.r, c.y);
                
                                //var hlon = this.lon/15.0;
                                //var strlon = Numbers.toSexagesimal(hlon, this.prec+1, false);
                                let coo = new Coo(ra, dec, 7);
                                let [lon, lat] = coo.format('d2');
            
                                let fov = new Angle(radius, 1).degrees();
                                //selectorBtn.update({tooltip: {content: 'center: ' + ra.toFixed(2) + ', ' + dec.toFixed(2) + '<br\>radius: ' + radius.toFixed(2), position: {direction: 'left'}}})    
                                self.form.set('ra', lon)
                                self.form.set('dec', lat)
                                self.form.set('rad', fov)
                            } catch (e) {
                                alert(e, 'Cone search out of projection')
                            }
            
                            self._show()
                        }
                    }, self.aladin)

                    header = Layout.horizontal(['Circle', csBtn]);
                    break;
                default:
                    header = param;
                    break;
            }

            subInput.header = header;
            subInputs.push(subInput)
        }

        this.form.update({
            subInputs,
        })
    }
}