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
import { MocServer } from "../../MocServer.js";

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { Coo } from "../../libs/astro/coo.js";
import { Form } from "../Widgets/Form.js";
import { Angle } from "../../libs/astro/angle.js";
import A from "../../A.js";
import { Dropdown } from "../Input/Dropdown.js";
import { ConeSearchActionButton } from "../Button/ConeSearch.js";
import targetIconUrl from '../../../../assets/icons/target.svg';
import hipsIconUrl from '../../../../assets/icons/hips.svg';
import { ActionButton } from "../Widgets/ActionButton.js";

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/Box/CatalogQueryBox.js
 *
 * Author: Thomas Boch[CDS], Matthieu Baumann[CDS]
 *****************************************************************************/

 export class CatalogQueryBox extends Box {
    static catalogs = {};
    static catTitle2Id = {};
    constructor(aladin, options) {
        // Query the mocserver
        MocServer.getAllCatalogHiPSes()
            .then((catalogs) => {
                catalogs.forEach((cat) => {
                    //console.log(cat)
                    CatalogQueryBox.catalogs[cat.ID] = cat;
                    CatalogQueryBox.catTitle2Id[cat.obs_title] = cat.ID;
                });

                //console.log(CatalogQueryBox.catalogs)
                searchDropdown.update({options: Object.keys(CatalogQueryBox.catTitle2Id)})
            })

        const fnIdSelected = function(type, params) {
            if (type=='coneSearch') {
                let errorCallback = (e) => {
                    alert(e + '.\nThe table might contain no data for the cone search specified.');
                }
                if (params.baseURL.includes('/vizier.')) {
                    A.catalogFromVizieR(
                        params.id.replace('CDS/', ''),
                        params.ra + ' ' + params.dec,
                        params.radiusDeg,
                        {limit: params.limit, onClick: 'showTable', hoverColor: 'red'},
                        (catalog) => {
                            aladin.addCatalog(catalog)
                        },
                        errorCallback
                    );
                }
                else if (params.baseURL.includes('/simbad.')) {
                    A.catalogFromSimbad(
                        params.ra + ' ' + params.dec,
                        params.radiusDeg,
                        {limit: params.limit, onClick: 'showTable', hoverColor: 'red'},
                        (catalog) => {
                            aladin.addCatalog(catalog)
                        },
                        errorCallback
                    );
                }
                else {
                    let url = params.baseURL;
                    if (! url.endsWith('?')) {
                        url += '?';
                    }
                    url += 'RA=' + params.ra + '&DEC=' + params.dec + '&SR=' + params.radiusDeg;
                    A.catalogFromURL(
                        url,
                        {limit: params.limit, onClick: 'showTable', hoverColor: 'red'},
                        (catalog) => {
                            aladin.addCatalog(catalog)
                        },
                        errorCallback
                    );
                }
            }
            else if (type=='hips') {
                const hips = A.catalogHiPS(params.hipsURL, {onClick: 'showTable', name: params.id});
                aladin.addCatalog(hips);
            } else if (type=='votable') {
                A.catalogFromURL(params.url, {name: params.url, hoverColor: 'red'}, (catalog) => {
                    aladin.addCatalog(catalog)
                    params.success()
                }, params.error);
            }
        };

        const _parseEntry = (e) => {
            // parse the value
            const value = e.target.value;
        
            // A user can put an url
            try {
                let votableUrl = new URL(value).href;

                self.fnIdSelected('votable', {
                    url: votableUrl,
                    success: () => {
                        searchDropdown.addClass('aladin-valid');
                    },
                    error: () => {
                        searchDropdown.addClass('aladin-not-valid')
                        self.csForm.submit.update({disabled: true})
                        self.hipsCatLoad.update({disabled: true});
                    }
                })
            } catch (e) {
                // Or he can select a HiPS from the list given
                const catID = CatalogQueryBox.catTitle2Id[value]; 
                const catalog = CatalogQueryBox.catalogs[catID];

                if (catalog) {
                    self._selectItem(catalog, aladin);
                    searchDropdown.addClass('aladin-valid');
                } else {
                    // consider it as a cat ID and search in catalogs for it
                    const foundCat = Object.values(CatalogQueryBox.catalogs)
                        .find((c) => c.ID === value);
                    if (foundCat) {
                        self._selectItem(foundCat, aladin);
                        searchDropdown.addClass('aladin-valid')
                    } else {
                        searchDropdown.addClass('aladin-not-valid')
                        self.csForm.submit.update({disabled: true})
                        self.hipsCatLoad.update({disabled: true});
                    }
                }
            }
        }

        let searchDropdown = new Dropdown(aladin, {
            name: 'catalogs',
            placeholder: "Type ID, title, keyword or URL",
            tooltip: {
                global: true,
                aladin,
                content: 'HiPS url, ID or keyword accepted',
            },
            action: (e) => {
                _parseEntry(e)
            }
        });

        let self;

        let hipsCatLoad = new ActionButton({
            icon: {
                monochrome: true,
                url: hipsIconUrl,
                size: 'small',
            },
            tooltip: {
                content: "Load the progressive tiled catalog.<br/>Adapted for rendering big catalogs",
                position: {direction: "bottom"}
            },
            content: 'HiPS',
            disabled: true,
            action() {
                self.fnIdSelected('hips', {
                    hipsURL: self.selectedItem.hips_service_url,
                    id: self.selectedItem.ID,
                })

                self._hide();

                self.callback && self.callback();
            }
        })

        let selectorBtn = new ConeSearchActionButton({
            tooltip: {content: 'Select the area to query the catalogue with', position: {direction: 'left'}},
            onBeforeClick(e) {
                self._hide();
            },
            action(circle) {
                // convert to ra, dec and radius in deg
                try {
                    let [ra, dec] = aladin.pix2world(circle.x, circle.y, 'icrs');
                    let radius = aladin.angularDist(circle.x, circle.y, circle.x + circle.r, circle.y, 'icrs');
    
                    //var hlon = this.lon/15.0;
                    //var strlon = Numbers.toSexagesimal(hlon, this.prec+1, false);
                    let coo = new Coo(ra, dec, 7);
                    let [lon, lat] = coo.format('s2');

                    let fov = new Angle(radius, 1).format();
                    form.set('ra', lon)
                    form.set('dec', lat)
                    form.set('rad', fov)
                } catch (e) {
                    alert(e, 'Cone search out of projection')
                }

                self._show()
            }
        }, aladin)

        let [ra, dec] = aladin.getRaDec();
        let centerCoo = new Coo(ra, dec, 5);
        let [defaultRa, defaultDec] = centerCoo.format('s2');

        let fov = aladin.getFov();
        let fovAngle = new Angle(Math.min(fov[0], fov[1]) / 2, 1).format()

        let form = new Form({
            submit: {
                disabled: true,
                icon: {
                    monochrome: true,
                    url: targetIconUrl,
                    size: 'small',
                    cssStyle: {
                        cursor: 'help',
                    }
                },
                content: "Query",
                tooltip: { position: {direction: 'bottom'}, content: 'Call the cone search service'},
                action(values) {
                    self._hide();

                    let coo = new Coo();
                    coo.parse(values.ra + ' ' + values.dec)
    
                    let rad = new Angle();
                    rad.parse(values.rad)

                    self.fnIdSelected('coneSearch', {
                        baseURL: self.selectedItem.cs_service_url,
                        id: self.selectedItem.ID,
                        ra: coo.lon,
                        dec: coo.lat,
                        radiusDeg: rad.degrees(),
                        limit: values.limit
                    })
    
                    self.callback && self.callback()
                }
            },
            subInputs: [
                {
                    type: 'group',
                    header: Layout.horizontal([selectorBtn, 'Cone search']),
                    subInputs: [
                        {
                            label: "RA:",
                            name: "ra",
                            type: "text",
                            value: defaultRa,
                            placeholder: 'Right ascension',
                            actions: {
                                change(e, input) {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "Dec:",
                            name: "dec",
                            type: "text",
                            value: defaultDec,
                            placeholder: 'Declination',
                            actions: {
                                change(e, input) {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        },
                        {
                            label: "Rad:",
                            name: "rad",
                            type: 'text',
                            value: fovAngle,
                            placeholder: 'Radius',
                            actions: {
                                change(e, input) {
                                    input.addEventListener('blur', (event) => {});
                                },
                            }
                        }
                    ]
                },
                {
                    type: 'group',
                    header: 'Max number of sources',
                    subInputs: [{
                        label: "Limit:",
                        name: "limit",
                        step: '1',
                        value: 1000,
                        type: "number",
                        placeholder: 'Limit number of sources',
                        actions: {
                            change(e, input) {
                                input.addEventListener('blur', (event) => {});
                            },
                        }
                    }]
                },
            ]
        });

        super({
            close: true,
            header: {
                title: "Catalog browser",
            },
            classList: ['aladin-cat-browser-box'],
            content: Layout.vertical(
                [
                    Layout.horizontal(
                        ["Search:", searchDropdown],
                        {cssStyle:
                            {width: '100%'}
                        }
                    ),
                    Layout.horizontal(
                        ["Progressive catalog:", hipsCatLoad],
                        {
                            cssStyle: {
                                textAlign: "center",
                                display: "flex",
                                alignItems: "center",
                                listStyle: "none",
                                justifyContent: "space-between",
                                width: "100%",
                            },
                        }
                    ),
                    form
                ]
            ),
            ...options
        }, aladin.aladinDiv)

        self = this;
        this.hipsCatLoad = hipsCatLoad;
        this.csForm = form;
        this.searchDropdown = searchDropdown;
        this.fnIdSelected = fnIdSelected;
    }

    _selectItem(item, aladin) {
        this.selectedItem = item;

        if (!item) {
            this.csForm.submit.update({disabled: true})
            this.hipsCatLoad.update({disabled: true});
        } else {
            if (item && item.cs_service_url) {
                this.csForm.submit.update({disabled: false});
            }
            
            if (item && item.hips_service_url) {
                this.hipsCatLoad.update({disabled: false});
            }
        }
    }

    attach(options) {
        this.callback = options.callback;
        super.update(options)
    }

    _hide() {
        if (this.box) {
            this.box.remove();
        }

        super._hide()
    }
}
