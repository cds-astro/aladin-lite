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
import { Dropdown } from "../Input/Dropdown.js";
import filterOnUrl from "../../../../assets/icons/filter-on.svg";
import treeIconUrl from "../../../../assets/icons/tree.svg";
import filterOffUrl from "../../../../assets/icons/filter-off.svg";
import helpIconUrl from "../../../../assets/icons/help.svg";

import { Input } from "../Widgets/Input.js";
import { WidgetTogglerButton } from "../Button/Toggler.js";
import { Layout } from "../Layout.js";
import { HiPSFilterBox } from "./HiPSFilterBox.js";
import A from "../../A.js";
import { Utils } from "../../Utils.ts";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Icon } from "../Widgets/Icon.js";
import { Tree } from "../Widgets/Tree.js";
import { ALEvent } from "../../events/ALEvent.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Box/HiPSBrowserBox.js
 *
 * Author: Matthieu Baumann[CDS]
 *****************************************************************************/

function fillHiPSHierarchy(name, hips, path, hierarchy) {
    if (path[path.length - 1] === '/') {
        path = path.substring(0, path.length - 1);
    }

    let folders = path.split('/')
    let curFolder = folders.shift()

    if(curFolder === 'Image') {
        let newPath = folders.join('/')
        fillHiPSHierarchy(name, hips, newPath, hierarchy);
    } else {
        // Some exceptions because the MOCServer client_category field may contain some typos
        if (['X', 'X-ray', 'Xray'].includes(curFolder)) {
            curFolder = 'X-ray'
        }

        if (['Radion', 'Radio'].includes(curFolder)) {
            curFolder = 'Radio'
        }

        if (curFolder === "Deprecated")
            return;

        hierarchy[curFolder] = hierarchy[curFolder] || {};
        if (folders.length == 0) {
            hierarchy[curFolder][name] = hips
        } else {
            let newPath = folders.join('/')
            fillHiPSHierarchy(name, hips, newPath, hierarchy[curFolder])
        }
    }
}

export class HiPSBrowserBox extends Box {
    static HiPSList = {};

    constructor(aladin, options) {
        let self;

        let filter = (item, params) => {
            if (params.regime) {
                if (!item.obs_regime)
                    return false;

                if (params.regime.toLowerCase() !== item.obs_regime.toLowerCase()) {
                    return false;
                }
            }

            if (params.resolution) {
                if (!item.hips_tile_width || !item.hips_order) {
                    return false;
                }

                let pixelHEALPixOrder = Math.log2(item.hips_tile_width) + (+item.hips_order);
                let resPixel = Math.sqrt(Math.PI / (3*Math.pow(4, pixelHEALPixOrder)));

                if (resPixel > params.resolution)
                    return false;
            }

            if (params.title) {
                if (!item.obs_title && !item.ID)
                    return false;

                let obsTitleDoesNotMatch = !item.obs_title.toLowerCase().includes(params.title.toLowerCase());
                let creatorDidDoesNotMatch = !item.ID.toLowerCase().includes(params.title.toLowerCase());

                if (obsTitleDoesNotMatch && creatorDidDoesNotMatch) {
                    return false;
                }
            }

            return true;
        };

        // Search tree
        let searchTree = new Tree({
            // a JS object describing the tree to show
            label: (item) => {
                let name = item.obs_title.replace(/:|\'/g, '');
                return name;
            },
            // a callback called when the user selects a leaf item of the tree
            click: (item) => {
                let image = item.ID || item.hips_service_url;
                let name = item.obs_title || item.ID;
                self._addHiPS(image, name)
            },
            dblclick: (item) => {
                let image = item.ID || item.hips_service_url;
                let name = item.obs_title || item.ID;
                self._addHiPS(image, name)

                self.close();
            },
            // a callback called for filtering
            filter,
            aladin,
        });

        MocServer.getAllHiPSes().then((HiPSes) => {
            HiPSBrowserBox.HiPSList = {}

            let hipsHierarchy = {};
            // Fill the HiPSList from the MOCServer

            // Build a hierarchy w.r.t sorted by regime
            let HiPSIDs = []
            HiPSes.forEach((h) => {
                let name = h.obs_title;
                name = name.replace(/:|\'/g, '');
                HiPSIDs.push(name)

                HiPSBrowserBox.HiPSList[name] = h;

                if (h.client_category) {
                    let path = h.client_category

                    fillHiPSHierarchy(name, h, path, hipsHierarchy)
                } else {
                    let hipsID = h.ID;
                    hipsID = hipsID.replace('/P', '');

                    let path = "Others/" + hipsID;
                    fillHiPSHierarchy(name, h, path, hipsHierarchy)
                }
            });

            self.searchDropdown.update({ options: HiPSIDs });
            self.searchTree.setHierarchy(hipsHierarchy)

            // Initialize the autocompletion without any filtering
            self._filterHiPSList({})
        });

        const _parseHiPS = (e) => {
            const value = e.target.value;

            let image, name;
            // A user can put an url
            try {
                image = new URL(value).href;
                name = image;
            } catch (e) {
                // Or he can select a HiPS from the list given
                const hips = HiPSBrowserBox.HiPSList[value];
                if (hips) {
                    image = hips.ID || hips.hips_service_url;
                    name = hips.obs_title || hips.ID;
                } else {
                    // Finally if not found, interpret the input text value as the HiPS (e.g. ID)
                    image = value;
                    name = value;
                }
            }

            if (image) {
                self._addHiPS(image, name)
            }
        };

        let searchDropdown = new Dropdown(aladin, {
            name: "HiPS browser",
            placeholder: "Browse a HiPS by an URL, ID or keywords",
            tooltip: {
                global: true,
                aladin,
                content: 'HiPS url, ID or keyword accepted.',
            },
            action: (e) => {
                _parseHiPS(e)
            },
            input: (e) => {
                let value = e.target.value;
                self.searchTree.triggerFilter({title: value});

                self.infoCurrentHiPSBtn.update({
                    disabled: true,
                })
            },
        });

        let filterEnabler = Input.checkbox({
            name: "filter-enabler",
            tooltip: { content: "Enable the filter" },
            checked: false,
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
                    checked: on,
                });
            },
        });

        let infoCurrentHiPSBtn = ActionButton.BUTTONS(aladin)
            .infoHiPS({disabled: true})

        let filterBox = new HiPSFilterBox(aladin, {
            callback: (params) => {
                self._filterHiPSList(params);
            },
        })
        let filterBtn = new WidgetTogglerButton({
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
            openPosition: 'right center',
            widget: filterBox,
        });

        let filterNumberElt = document.createElement("div");

        super({
                close: true,
                tooltip: {
                    global: true,
                    aladin,
                    content: 'orange: out of the view, green: in view'
                },
                header: {
                    title: [
                        new Icon({
                            size: 'medium',
                            url: treeIconUrl,
                            monochrome: true,
                        }),
                        "HiPS browser",
                        new Icon({
                            size: 'medium',
                            url: helpIconUrl,
                            monochrome: true,
                            tooltip: {
                                content: 'HiPS:<br/><span class="aladin-indicator aladin-not-found"></span> out of the view<br /><span class="aladin-indicator aladin-valid"></span> in view',
                                mouse: true,
                                aladin
                            },
                            style: {
                                cursor: 'help'
                            }
                        }),
                    ],
                    draggable: true,
                },
                sizeable: true,
                classList: ['aladin-HiPS-browser-box'],
                content: [
                    searchTree,
                    ["Search:", searchDropdown, infoCurrentHiPSBtn],
                    [filterEnabler, filterBtn, filterNumberElt],
                ],
                ...options,
            },
            aladin.aladinDiv
        );

        self = this;

        this.searchTree = searchTree;
        this.filterBox = filterBox;
        this.filterNumberElt = filterNumberElt;
        this.filterBox._hide();

        this.searchDropdown = searchDropdown;
        this.filterBtn = filterBtn;
        this.aladin = aladin;

        this.infoCurrentHiPSBtn = infoCurrentHiPSBtn;

        this.filter = filter;

        filterEnabler.action({target: {checked: true}});

        this._addListeners();

        this._requestMOCServer();
    }

    _addListeners() {
        const requestMOCServerDebounced = Utils.debounce(() => {
            this._requestMOCServer()
        }, 500);

        ALEvent.POSITION_CHANGED.listenedBy(this.aladin.aladinDiv, requestMOCServerDebounced);
        ALEvent.ZOOM_CHANGED.listenedBy(this.aladin.aladinDiv, requestMOCServerDebounced);
    }

    _requestMOCServer() {
        if (this.isHidden && this.searchTree) {
            return;
        }

        let self = this;
        MocServer.getAllHiPSesInsideView(this.aladin)
            .then((HiPSes) => {
                let HiPSIDs = HiPSes.map((x) => x.ID);
                self.searchTree.highlightNodes(HiPSIDs)
            })
    }

    _addHiPS(id, name) {
        let self = this;

        self.searchDropdown.update({value: name, title: name});

        let hips = A.imageHiPS(id, {
            name,
            successCallback: (hips) => {
                self.searchDropdown.removeClass('aladin-not-valid');
                self.searchDropdown.addClass('aladin-valid');

                self.infoCurrentHiPSBtn.update({
                    disabled: false,
                    action(e) {
                        window.open(hips.url);
                    }
                })

                self.aladin.removeUIByName("cube_displayer" + hips.layer)

                if (!hips.cubeDepth)
                    return;

                let numSlices = hips.cubeDepth;
                let idxSlice = hips.cubeFirstFrame;

                hips.setSliceNumber(idxSlice)

                let toStr = (n, paddingBegin = false) => {
                    let s = n.toString();
                    let maxNumDigits = numSlices.toString().length;

                    if (s.length < maxNumDigits) {
                        let r = '&nbsp;'.repeat(maxNumDigits - s.length)
                        if (paddingBegin) {
                            s = r + s 
                        } else {
                            s += r
                        }
                    }

                    return s;
                }

                let updateSlice = () => {
                    slicer.update({
                        value: idxSlice,
                        tooltip: {content: (idxSlice + 1) + '/' + numSlices, position: {direction: 'bottom'}},
                    })

                    hips.setSliceNumber(idxSlice)
                    cubeDisplayer.update({position: cubeDisplayer.position, content: Layout.horizontal([prevBtn, nextBtn, slicer, toStr(idxSlice + 1, true) + '/' + toStr(numSlices, false)])})
                };

                let slicer = Input.slider({
                    label: "Slice",
                    name: "cube_slicer" + hips.layer,
                    ticks: [idxSlice],
                    tooltip: {content: (idxSlice + 1) + '/' + numSlices, position: {direction: 'bottom'}},
                    min: 0,
                    max: numSlices - 1,
                    value: idxSlice,
                    actions: {
                        change: (e) => {
                            idxSlice = Math.round(e.target.value);

                            updateSlice();
                        },
                        input: (e) => {
                            idxSlice = Math.round(e.target.value);

                            slicer.update({
                                value: idxSlice,
                                tooltip: {content: (idxSlice + 1) + '/' + numSlices, position: {direction: 'bottom'}},
                            })
                        }
                    },
                    cssStyle: {
                        width: '300px'
                    }
                });
                                                
                let prevBtn = A.button({
                    size: 'small',
                    content: '<',
                    action(o) {
                        idxSlice = Math.max(idxSlice - 1, 0);
                        updateSlice()
                    }
                })
                                                
                let nextBtn = A.button({
                    size: 'small',
                    content: '>',
                    action(o) {
                        idxSlice = Math.min(idxSlice + 1, numSlices - 1);
                        updateSlice()
                    }
                })

                let cubeDisplayer = A.box({
                    close: true,
                    name: "cube_displayer" + hips.layer,
                    header: {
                        title: 'Player for: ' + hips.name,
                        draggable: true,
                    },
                    content: Layout.horizontal([prevBtn, nextBtn, slicer, toStr(idxSlice + 1, true) + '/' + toStr(numSlices, false)]),
                    position: {anchor: 'center top'},
                });

                self.aladin.addUI(cubeDisplayer)
            },
            errorCallback: (e) => {
                self.searchDropdown.removeClass('aladin-valid');
                self.searchDropdown.addClass('aladin-not-valid');
            }
        });

        self.selected(hips)
    }

    // This method is executed only if the filter is enabled
    _filterHiPSList(params) {
        let self = this;

        let numHiPSMatching = 0;
        for (var key in HiPSBrowserBox.HiPSList) {
            let HiPS = HiPSBrowserBox.HiPSList[key];
            // apply filtering
            if (
                self.filter &&
                self.filter(HiPS, params)
            ) {
                // search with the name or id
                let name = HiPS.obs_title;
                name = name.replace(/:|\'/g, "");

                numHiPSMatching += 1;
            }
        }

        if (self.searchTree) {
            self.searchTree.triggerFilter(params);
        }

        self.filterNumberElt.innerHTML = numHiPSMatching + "/" + Object.keys(HiPSBrowserBox.HiPSList).length;
    }

    _hide() {
        if (this.filterBtn && this.filterBtn.toggled) {
            this.filterBtn.toggle();
        }

        super._hide()
    }

    _show(options) {
        // Regenerate a new layer name
        this.selected = options && options.selected;

        super._show(options)

        this._requestMOCServer();
    }
}
