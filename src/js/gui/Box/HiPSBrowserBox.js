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

import { MocServer } from "../../MocServer.js";

import { Box } from "../Widgets/Box.js";
import { Dropdown } from "../Input/Dropdown.js";
import filterOnUrl from "../../../../assets/icons/filter-on.svg";
import filterOffUrl from "../../../../assets/icons/filter-off.svg";
import { Input } from "../Widgets/Input.js";
import { TogglerActionButton } from "../Button/Toggler.js";
import { Layout } from "../Layout.js";
import { HiPSFilterBox } from "./HiPSFilterBox.js";
import A from "../../A.js";
import { Utils } from "../../Utils.ts";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/HiPSBrowserBox.js
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

export class HiPSBrowserBox extends Box {
    static HiPSList = {};

    constructor(aladin, options) {
        let self;

        MocServer.getAllHiPSes().then((HiPSes) => {
            // Fill the HiPSList from the MOCServer
            HiPSes.forEach((h) => {
                HiPSBrowserBox.HiPSList[h.obs_title] = h;
            });

            // Initialize the autocompletion without any filtering
            self._filterHiPSList({})
        });

        const _parseHiPS = (e) => {
            const value = e.target.value;

            let image;
            // A user can put an url
            try {
                image = new URL(value).href;
            } catch (e) {
                // Or he can select a HiPS from the list given
                const hips = HiPSBrowserBox.HiPSList[value];
                console.log(hips)
                if (hips) {
                    image = hips.ID || hips.hips_service_url;
                } else {
                    // Finally if not found, interpret the input text value as the HiPS (e.g. ID)
                    image = value;
                }
            }


            if (image) {
                self._addHiPS(image)
            }
        };

        let searchDropdown = new Dropdown(aladin, {
            name: "HiPS browser",
            placeholder: "Browser a HiPS by an URL, ID or keywords",
            tooltip: {
                content: 'Type an url, CDS ID or a name to search for a HiPS',
                position: {
                    direction: 'bottom'
                }
            },
            actions: {
                focus(e) {
                    searchDropdown.removeClass('aladin-valid')
                    searchDropdown.removeClass('aladin-not-valid')
                },
                keydown(e) {
                    e.stopPropagation();

                    if (e.key === 'Enter') {
                        e.preventDefault()
                        _parseHiPS(e)
                    }
                },
                input(e) {
                    searchDropdown.removeClass('aladin-valid')
                    searchDropdown.removeClass('aladin-not-valid')
                },
                change(e) {
                    _parseHiPS(e)
                },
            },
        });

        let filterEnabler = Input.checkbox({
            name: "filter-enabler",
            checked: false,
            tooltip: {
                content: "Filter off",
            },
            click(e) {
                let on = e.target.checked;
                self.filterBox.enable(on);

                if (!on) {
                    // if the filter has been disabled we also need to update
                    // the autocompletion list of the search dropdown
                    // We give no filter params
                    self._filterHiPSList({});
                }

                filterBtn.update({
                    icon: {
                        url: on ? filterOnUrl : filterOffUrl,
                        monochrome: true,
                    },
                });

                filterEnabler.update({
                    tooltip: {
                        content: on
                            ? "Filtering on"
                            : "Filtering off",
                    },
                    checked: on,
                });
            },
        });

        let filterBtn = new TogglerActionButton({
            icon: {
                url: filterOffUrl,
                monochrome: true,
            },
            size: "small",
            tooltip: {
                content: "Want to filter HiPS surveys by criteria ?",
                position: { direction: "top" },
            },
            toggled: false,
            actionOn: (e) => {
                self.filterBox._show({
                    position: {
                        nextTo: filterBtn,
                        direction: "right",
                        aladin,
                    },
                });
            },
            actionOff: (e) => {
                self.filterBox._hide();
            },
        });

        super(
            {
                close: true,
                header: {
                    title: "HiPS browser",
                },
                onDragged: () => {
                    if (self.filterBtn.toggled) {
                        self.filterBtn.toggle();
                    }
                },
                classList: ['aladin-HiPS-browser-box'],
                content: Layout.vertical([
                    Layout.horizontal(["Filter:", filterEnabler, filterBtn]),
                    Layout.horizontal(["Search:", searchDropdown]),
                    //addBtn,
                ]),
                ...options,
            },
            aladin.aladinDiv
        );

        this.filterBox = new HiPSFilterBox(aladin, {
            callback: (params) => {
                self._filterHiPSList(params);
            },
        })
        this.filterBox._hide();

        this.searchDropdown = searchDropdown;
        this.filterBtn = filterBtn;
        this.aladin = aladin;

        self = this;

        this.filterCallback = (HiPS, params) => {
            if (!HiPS.obs_regime || (
                params.regime &&
                HiPS.obs_regime &&
                params.regime.toLowerCase() !==
                    HiPS.obs_regime.toLowerCase()
            )) {
                return false;
            }

            if (Array.isArray(params.spatial) && HiPS.ID && !(params.spatial.includes(HiPS.ID))) {
                return false;
            }

            if (!HiPS.hips_tile_width || !HiPS.hips_order)
                return false;

            if (params.resolution) {
                let pixelHEALPixOrder = Math.log2(HiPS.hips_tile_width) + HiPS.hips_order;
                let resPixel = Math.sqrt(Math.PI / (3*Math.pow(4, pixelHEALPixOrder)));

                if (resPixel > params.resolution)
                    return false;
            }

            return true;
        };
    }

    _addHiPS(id) {
        let self = this;
        let hips = A.imageHiPS(id, {
            successCallback: (hips) => {
                console.log('success')
                self.searchDropdown.removeClass('aladin-not-valid');
                self.searchDropdown.addClass('aladin-valid');
            },
            errorCallback: (e) => {
                self.searchDropdown.removeClass('aladin-valid');
                self.searchDropdown.addClass('aladin-not-valid');
            }
        });
        this.aladin.setOverlayImageLayer(hips, self.layer);
    }

    // This method is executed only if the filter is enabled
    _filterHiPSList(params) {
        let self = this;
        let HiPSIDs = [];

        for (var key in HiPSBrowserBox.HiPSList) {
            let HiPS = HiPSBrowserBox.HiPSList[key];

            // apply filtering
            if (
                self.filterCallback &&
                self.filterCallback(HiPS, params)
            ) {
                // search with the name or id
                HiPSIDs.push(HiPS.obs_title);
            }
        }

        self.searchDropdown.update({ options: HiPSIDs });
    }

    _hide() {
        if (this.filterBox)
            this.filterBox.signalBrowserStatus(true)

        if (this.filterBtn && this.filterBtn.toggled) {
            this.filterBtn.toggle();
        }

        super._hide()
    }

    _show(options) {
        // Regenerate a new layer name
        this.layer = Utils.uuidv4()

        if (this.filterBox)
            this.filterBox.signalBrowserStatus(false)

        super._show(options)
    }
}
