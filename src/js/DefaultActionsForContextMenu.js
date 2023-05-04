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
 * File DefaultActionsForContextMenu
 *
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import { GenericPointer } from "./GenericPointer.js";

export let DefaultActionsForContextMenu = (function () {

    let DefaultActionsForContextMenu = {};

    DefaultActionsForContextMenu.getDefaultActions = function (aladinInstance) {
        return [
            {
                label: "Copy position", action(o) {
                    var r = document.createRange();
                    r.selectNode(o.target);
                    window.getSelection().removeAllRanges();
                    window.getSelection().addRange(r);
                    try {
                        let successful = document.execCommand('copy');
                        let msg = successful ? 'successful' : 'unsuccessful';
                        //console.log('Copying text command was ' + msg);
                    } catch (err) {
                        console.error('Oops, unable to copy to clipboard');
                    }
                    window.getSelection().removeAllRanges();
                }
            },
            {
                label: "Take snapshot", action(o) { aladinInstance.exportAsPNG(); }
            },
            {
                label: "Add",
                subMenu: [
                    {
                        label: 'New image layer', action(o) {
                            aladinInstance.addNewImageLayer();
                        }
                    },
                    {
                        label: 'New catalogue layer', action(o) {
                            aladinInstance.stack._onAddCatalogue();
                        }
                    },
                ]
            },
            {
                label: "Load local file",
                subMenu: [
                    {
                        label: 'Load FITS MOC', action(o) {
                            let input = document.createElement('input');
                            input.type = 'file';
                            input.onchange = _ => {
                                let files = Array.from(input.files);

                                files.forEach(file => {
                                    const url = URL.createObjectURL(file);
                                    let moc = A.MOCFromURL(url, { name: file.name });
                                    aladinInstance.addMOC(moc);
                                });
                            };
                            input.click();
                        }
                    },
                    {
                        label: 'Load VOTable', action(o) {
                            let input = document.createElement('input');
                            input.type = 'file';
                            input.onchange = _ => {
                                let files = Array.from(input.files);

                                files.forEach(file => {
                                    const url = URL.createObjectURL(file);
                                    let catalogue = A.catalogFromURL(url, { name: file.name, onClick: 'showTable'}, null, false);
                                    aladinInstance.addCatalog(catalogue);
                                });
                            };
                            input.click();
                        }
                    }
                ]
            },
            {
                label: "What is this?", action(e) {
                    GenericPointer(aladinInstance.view, e);
                }
            },
            {
                label: "HiPS2FITS cutout", action(o) {
                    const a = aladinInstance;
                    let hips2fitsUrl = 'https://alasky.cds.unistra.fr/hips-image-services/hips2fits#';
                    let radec = a.getRaDec();
                    let fov = Math.max.apply(null, a.getFov());
                    let hipsId = a.getBaseImageLayer().id;
                    let proj = a.getProjectionName();
                    hips2fitsUrl += 'ra=' + radec[0] + '&dec=' + radec[1] + '&fov=' + fov + '&projection=' + proj + '&hips=' + encodeURIComponent(hipsId);
                    window.open(hips2fitsUrl, '_blank');
                }
            },
            {
                label: "Select sources", action(o) {
                    const a = aladinInstance;

                    a.select();
                }
            },
        ]
    }

    return DefaultActionsForContextMenu;

})();
