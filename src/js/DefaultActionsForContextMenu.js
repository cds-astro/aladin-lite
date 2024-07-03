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
import A from "./A.js";
import { CatalogQueryBox } from "./gui/Box/CatalogQueryBox.js";
import cameraIconUrl from '../../assets/icons/camera.svg'
import targetIconUrl from '../../assets/icons/target.svg';
import uploadIconUrl from '../../assets/icons/upload.svg';
import selectIconUrl from '../../assets/icons/select.svg';

export let DefaultActionsForContextMenu = (function () {

    let DefaultActionsForContextMenu = {};

    DefaultActionsForContextMenu.getDefaultActions = function (aladinInstance) {
        const a = aladinInstance;

        const selectObjects = (selection) => {
            a.view.selectObjects(selection);
        };
        return [
            {
                label: "Copy position", action(o) {
                    let msg;
                    let text = o.target.innerText;
                    if (!text) {
                        return false;
                    }

                    navigator.clipboard.writeText(text)
                        .then(() => {
                            msg = 'successful'
                            if (aladinInstance.statusBar) {
                                aladinInstance.statusBar.appendMessage({
                                    message: 'Reticle location saved!',
                                    duration: 2000,
                                    type: 'info'
                                })
                            }
                        })
                        .catch((e) => {
                            msg = 'unsuccessful'
                            console.info('Oops, unable to copy', e);
                        })
                        .finally(() => {
                            console.info('Copying text command was ' + msg);
                        })
                }
            },
            {
                label: {
                    icon: {
                        tooltip: {content: 'Download a PNG image file of the view', position: {direction: 'top'}},
                        monochrome: true,
                        url: cameraIconUrl,
                        size: 'small',
                    },
                    content: 'Take a snapshot'
                },
                action(o) { a.exportAsPNG(); }
            },
            {
                label: "Add",
                subMenu: [
                    {
                        label: 'New image layer', action(o) {
                            a.addNewImageLayer();
                            if (a.menu) {
                                a.menu.open('stack')
                            }
                        }
                    },
                    {
                        label: 'New catalogue layer', action(o) {
                            let catBox = new CatalogQueryBox(a)
                            catBox._show({
                                header: {
                                    title: 'Add a new catalogue',
                                    draggable: true
                                },
                                position: {
                                    anchor :'center center'
                                },
                            });
                        }
                    },
                ]
            },
            {
                label: {
                    icon: {
                        monochrome: true,
                        url: uploadIconUrl,
                        cssStyle: {
                            cursor: 'help',
                        }
                    },
                    content: "Load a local file"
                },
                subMenu: [
                    {
                        label: 'FITS image', action(o) {
                            let input = document.createElement('input');
                            input.type = 'file';
                            input.onchange = _ => {
                                let files = Array.from(input.files);

                                files.forEach(file => {
                                    const url = URL.createObjectURL(file);
                                    const name = file.name;

                                    // Consider other cases
                                    const image = a.createImageFITS(
                                        url,
                                        {name},
                                        (ra, dec, fov, _) => {
                                            // Center the view around the new fits object
                                            a.gotoRaDec(ra, dec);
                                            a.setFoV(fov * 1.1);
                                        }
                                    );

                                    a.setOverlayImageLayer(image, name)
                                });
                            };
                            input.click();
                        }
                    },
                    {
                        label: 'FITS MOC', action(o) {
                            let input = document.createElement('input');
                            input.type = 'file';
                            input.onchange = _ => {
                                let files = Array.from(input.files);

                                files.forEach(file => {
                                    const url = URL.createObjectURL(file);
                                    let moc = A.MOCFromURL(url, { name: file.name });
                                    a.addMOC(moc);
                                });
                            };
                            input.click();
                        }
                    },
                    {
                        label: 'VOTable', action(o) {
                            let input = document.createElement('input');
                            input.type = 'file';
                            input.onchange = _ => {
                                let files = Array.from(input.files);

                                files.forEach(file => {
                                    const url = URL.createObjectURL(file);
                                    A.catalogFromURL(url, { name: file.name, hoverColor: 'yellow', onClick: 'showTable'}, (catalog) => {
                                        a.addCatalog(catalog);
                                    }, false);
                                });
                            };
                            input.click();
                        }
                    }
                ]
            },
            {
                label: {
                    icon: {
                        monochrome: true,
                        url: selectIconUrl,
                        size: 'small',
                    },
                    content: "Select sources"
                },
                subMenu: [
                    {
                        label: 'Circular',
                        action(o) {
                            a.select('circle', selectObjects)
                        }
                    },
                    {
                        label: 'Rectangular',
                        action(o) {        
                            a.select('rect', selectObjects)
                        }
                    }
                ]
            },
            {
                label: {
                    icon: {
                        tooltip: {content: 'Use Sesame, our name resolver!', position: {direction: 'top'}},
                        monochrome: true,
                        url: targetIconUrl,
                        size: 'small',
                    },
                    content: 'What is this?'
                },
                
                action(o, ctxMenu) {
                    GenericPointer(a.view, {
                        x: parseInt(ctxMenu.element().style.left),
                        y: parseInt(ctxMenu.element().style.top)
                    });
                }
            },
            {
                label: "HiPS2FITS cutout", action(o) {
                    let hips2fitsUrl = 'https://alasky.cds.unistra.fr/hips-image-services/hips2fits#';
                    let radec = a.getRaDec();
                    let fov = Math.max.apply(null, a.getFov());
                    let hipsId = a.getBaseImageLayer().id;
                    let proj = a.getProjectionName();
                    hips2fitsUrl += 'ra=' + radec[0] + '&dec=' + radec[1] + '&fov=' + fov + '&projection=' + proj + '&hips=' + encodeURIComponent(hipsId);
                    window.open(hips2fitsUrl, '_blank');
                }
            },
        ]
    }

    return DefaultActionsForContextMenu;

})();
