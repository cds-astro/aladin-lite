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

import { ALEvent } from "../../events/ALEvent.js";
import { Layout } from "../Layout.js";
import { ContextMenu } from "../Widgets/ContextMenu.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { HiPSSelectorBox } from "../Box/HiPSSelectorBox.js";
import searchIconUrl from '../../../../assets/icons/search.svg';
import showIconUrl from '../../../../assets/icons/show.svg';
import hideIconUrl from '../../../../assets/icons/hide.svg';
import removeIconUrl from '../../../../assets/icons/remove.svg';
import editIconUrl from '../../../../assets/icons/edit.svg';
import { ImageFITS } from "../../ImageFITS.js";
import { LayerEditBox } from "../Box/SurveyEditBox.js";
import { Utils } from "../../Utils.ts";

export class Stack extends ContextMenu {
    static previewImagesUrl = {
        'allWISE/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_allWISE_color.jpg',
        'DECaPS/DR1/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DECaLS_DR5_color.jpg',
        'DSS2/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_color.jpg',
        'DSS2/red': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_red.jpg',
        'Fermi/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Fermi_color.jpg',
        'GALEXGR6/AIS/NUV': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GALEXGR6_7_color.jpg',
        'GLIMPSE360': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GLIMPSE360.jpg',
        'VTSS/Ha': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_VTSS_Ha.jpg',
        'SPITZER/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SPITZER_color.jpg',
        'IRIS/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_IRIS_color.jpg',
        'Mellinger/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Mellinger_color.jpg',
        'PanSTARRS/DR1/color-i-r-g': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_PanSTARRS_DR1_color-z-zg-g.jpg',
        'PanSTARRS/DR1/color-z-zg-g': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_PanSTARRS_DR1_color-z-zg-g.jpg',
        '2MASS/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_2MASS_color.jpg',
        'AKARI/FIS/Color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_AKARI_FIS_Color.jpg',
        'SWIFT_BAT_FLUX': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SWIFT_BAT_FLUX.jpg',
        'Finkbeiner': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Finkbeiner.jpg',
        'XMM/PN/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_XMM_PN_color.jpg',
        'SDSS9/color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SDSS9_color.jpg',
    };

    // Constructor
    constructor(aladin, menu) {
        super(aladin, {hideOnClick: false, hideOnResize: false});
        this.aladin = aladin;
        this.anchor = menu.controls["Stack"];
        //this.fsm = new StackLayerOpenerFSM(aladin, menu);

        window.addEventListener("resize", (e) => {
            this._hide();
        })
        
        document.addEventListener('click', () => {
            if (this.mode === 'stack') {
                this._hide()
            }
        });
        this.mode = 'stack';

        this._addListeners();
    }

    _addListeners() {
        let self = this;

        let updateImageList = () => {
            const layers = Array.from(self.aladin.getImageOverlays()).reverse().map((name) => {
                let overlay = self.aladin.getOverlayImageLayer(name);
                return overlay;
            });

            self.attach({layers});
            // If it is shown, update it
            if (!self.isHidden) {
                self._show()
            }
        };

        updateImageList();
        
        ALEvent.HIPS_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });

        ALEvent.HIPS_LAYER_RENAMED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();    
        });

        ALEvent.HIPS_LAYER_SWAP.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });
    }

    attach(options) {
        const layers = options && options.layers || [];

        let layout = [{
            label: 'Add a survey',
            subMenu: [{
                label: Layout.horizontal({
                        layout: [
                            ActionButton.createIconBtn({
                                iconURL: searchIconUrl,
                                tooltip: {content: 'From our database...', position: { direction: 'bottom' }},
                                cssStyle: {
                                    backgroundPosition: 'center center',
                                    backgroundColor: '#bababa',
                                    border: '1px solid rgb(72, 72, 72)',
                                    cursor: 'help',
                                },
                            }),
                            'Search for a progressive survey'
                        ]
                    }),
                    action(e) {
                        e.stopPropagation();
                        e.preventDefault();

                        self._hide();

                        self.hipsSelectorBox = new HiPSSelectorBox({
                            position: {
                                anchor: self.anchor,
                                direction: 'bottom'
                            },
                            layer: Utils.uuidv4(),
                        }, self.aladin);
                        self.hipsSelectorBox._show();

                        self.mode = 'hips';
                    }
                },
                ContextMenu.fileLoaderItem({
                    label: 'FITS image file',
                    action(file) {
                        let url = URL.createObjectURL(file);

                        const image = self.aladin.createImageFITS(
                            url,
                            file.name,
                            undefined,
                            (ra, dec, fov, _) => {
                                // Center the view around the new fits object
                                self.aladin.gotoRaDec(ra, dec);
                                self.aladin.setFoV(fov * 1.1);
                            },
                            undefined
                        );

                        self.aladin.setOverlayImageLayer(image, Utils.uuidv4())
                    }
                }),
            ]
        }];

        let self = this;
        let selectedLayer = self.aladin.getSelectedLayer();

        for(const layer of layers) {
            const name = layer.name;
            let backgroundUrl = this._findPreviewImageUrl(layer);
            let cssStyle = {
                height: 'fit-content',
            };

            if (backgroundUrl) {
                cssStyle = {
                    backgroundSize: '100%',
                    backgroundImage: 'url(' + backgroundUrl + ')',
                    ...cssStyle
                }
            }

            let showBtn = ActionButton.createIconBtn({
                iconURL: showIconUrl,
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: 'hidden',
                    width: '18px',
                    height: '18px',
                    verticalAlign: 'middle',
                    marginRight: '2px',
                },
                tooltip: {content: 'Hide', position: {direction: 'bottom'}},
                action(e, btn) {
                    let opacity = layer.getOpacity();
                    if (opacity === 0.0) {
                        layer.setOpacity(1.0);
                        btn.update({iconURL: showIconUrl, tooltip: {content: 'Hide'}});
                    } else {
                        layer.setOpacity(0.0);
                        btn.update({iconURL: hideIconUrl, tooltip: {content: 'Show'}});
                    }
                }
            });

            let deleteBtn = ActionButton.createIconBtn({
                iconURL: removeIconUrl,
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: 'hidden',
                    width: '18px',
                    height: '18px',
                    verticalAlign: 'middle'
                },
                disable: layer.layer === 'base',
                tooltip: {content: 'Remove', position: {direction: 'left'}},
                action(e) {
                    self.aladin.removeImageLayer(layer.layer);
                }
            });
            let editBtn = ActionButton.createIconBtn({
                iconURL: editIconUrl,
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: 'hidden',
                    width: '18px',
                    height: '18px',
                    verticalAlign: 'middle',
                    marginRight: '2px',
                },
                tooltip: {content: 'Edit', position: {direction: 'bottom'}},
                action: (e) => {
                    e.stopPropagation();
                    e.preventDefault();

                    self._hide();

                    self.aladin.selectLayer(layer.layer);
                    self.attach({layers})

                    let editBox = LayerEditBox.getInstance(self.aladin, self.anchor);
                    editBox.update({layer})
                    editBox._show();

                    self.mode = 'edit';
                }
            });

            let item = Layout.horizontal({
                layout: [
                    '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;' + (selectedLayer === layer.layer ? 'border: 1px solid white;' : '') + '">' + name + '</div>',
                    Layout.horizontal({layout: [showBtn, editBtn, deleteBtn]})
                ],
                cssStyle: {
                    display: 'flex',
                    alignItems: 'center',
                    listStyle: 'none',
                    justifyContent: 'space-between',
                    width: '100%',
                }
            });

            layout.push({
                label: item,
                cssStyle: cssStyle,
                hover(e) {
                    showBtn.el.style.visibility = 'visible'
                    editBtn.el.style.visibility = 'visible'
                    deleteBtn.el.style.visibility = 'visible'
                },
                unhover(e) {
                    showBtn.el.style.visibility = 'hidden'
                    editBtn.el.style.visibility = 'hidden'
                    deleteBtn.el.style.visibility = 'hidden'
                },
                action(o) {
                    self.aladin.selectLayer(layer.layer);
                    // recompute the stack
                    self.attach({layers})
                    self._show()
                }
            })
        }

        super.attach(layout);
    }

    _findPreviewImageUrl(layer) {
        if (layer instanceof ImageFITS) {
            return;
        }

        const creatorDid = layer.properties.creatorDid;
        
        for (const key in Stack.previewImagesUrl) {
            if (creatorDid.includes(key)) {
                return Stack.previewImagesUrl[key];
            }
        }
    }

    _show() {
        super.show({
            position: {
                anchor: this.anchor,
                direction: 'bottom',
            },
            cssStyle: {
                width: '15em',
                color: 'white',
                backgroundColor: 'black',
            }
        })
    }

    _hide() {
        // go back to the display stack state
        let editBox = LayerEditBox.getInstance(this.aladin, this.anchor);
        editBox._hide();

        if (this.hipsSelectorBox) {
            this.hipsSelectorBox._hide();
        }

        this.mode = 'stack';

        super._hide();
    }

    static singleton;

    static getInstance(aladin, menu) {
        if (!Stack.singleton) {
            Stack.singleton = new Stack(aladin, menu);
        }

        return Stack.singleton;
    }
}
   