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
import hipsIconUrl from "../../../../assets/icons/hips.svg";
import { Input } from "../Widgets/Input.js";
import { Layout } from "../Layout.js";
import A from "../../A.js";
import { Utils } from "../../Utils.ts";
import { ActionButton } from "../Widgets/ActionButton.js";
import infoIconUrl from "../../../../assets/icons/info.svg"
import { Icon } from "../Widgets/Icon.js";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Box/HiPSCompositeBox.js
 *
 * The code source of the interface for creating a new composite HiPS survey from multiple surveys
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

function fillHiPSHierarchy(name, hips, path, hierarchy) {
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

export class HiPSCompositeBox extends Box {
    static HiPSList = {};

    constructor(aladin, options) {
        let self;

        // Search tree
        MocServer.getAllHiPSes().then((HiPSes) => {
            HiPSBrowserBox.HiPSList = {}

            let hipsHierarchy = {};
            // Fill the HiPSList from the MOCServer

            // Build a hierarchy w.r.t sorted by regime
            HiPSes.forEach((h) => {
                let name = h.obs_title;
                name = name.replace(/:|\'/g, '');

                HiPSBrowserBox.HiPSList[name] = h;

                if (h.client_category) {
                    let path = h.client_category

                    fillHiPSHierarchy(name, h, path, hipsHierarchy)
                }
            });
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
                content: 'HiPS url, ID or keyword accepted',
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
                    self.infoCurrentHiPSBtn.update({
                        disable: true,
                    })

                    searchDropdown.removeClass('aladin-valid')
                    searchDropdown.removeClass('aladin-not-valid')
                },
            },
        });

        let infoCurrentHiPSBtn = new ActionButton({
            disable: true,
            icon: {
                size: 'medium',
                monochrome: true,
                url: infoIconUrl,
            },
            tooltip: {
                global: true,
                aladin,
                content: "More about that survey?"
            }
        });

        super(
            {
                close: true,
                header: {
                    title: Layout.horizontal([new Icon({
                        size: 'medium',
                        url: hipsIconUrl,
                        monochrome: true,
                    }), "HiPS Compositor"]),
                    draggable: true,
                },
                content: Layout.vertical([
                    Layout.horizontal([searchDropdown, infoCurrentHiPSBtn]),
                ]),
                ...options,
            },
            aladin.aladinDiv
        );

        self = this;

        this.searchDropdown = searchDropdown;
        this.aladin = aladin;

        this.infoCurrentHiPSBtn = infoCurrentHiPSBtn;

        this._addListeners();
    }

    _addListeners() {}

    _addHiPS(id, name) {
        let self = this;

        self.searchDropdown.update({value: name, title: name});

        let hips = A.imageHiPS(id, {
            name,
            successCallback: (hips) => {
                self.searchDropdown.removeClass('aladin-not-valid');
                self.searchDropdown.addClass('aladin-valid');

                self.infoCurrentHiPSBtn.update({
                    disable: false,
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
        this.aladin.setOverlayImageLayer(hips, self.layer);
    }

    _show(options) {
        // Regenerate a new layer name
        this.layer = (options && options.layer) || Utils.uuidv4();
        super._show(options)
    }
}
