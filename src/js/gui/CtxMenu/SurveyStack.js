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
import { ImageLayer } from "../../ImageLayer.js";
import searchIconImg from '../../../../assets/icons/search.svg';


export class Stack extends ContextMenu {
    static previewImagesUrl = {
        'AllWISE color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_allWISE_color.jpg',
        'DECaPS DR1 color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DECaLS_DR5_color.jpg',
        'DSS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_color.jpg',
        'DSS2 Red (F+R)': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_red.jpg',
        'Density map for Gaia EDR3 (I/350/gaiaedr3)' : undefined,
        'Fermi color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Fermi_color.jpg',
        'GALEXGR6_7 NUV': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GALEXGR6_7_color.jpg',
        'GLIMPSE360': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GLIMPSE360.jpg',
        'Halpha': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_VTSS_Ha.jpg',
        'IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SPITZER_color.jpg',
        'IRIS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_IRIS_color.jpg',
        'Mellinger colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Mellinger_color.jpg',
        'PanSTARRS DR1 color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_PanSTARRS_DR1_color-z-zg-g.jpg',
        'PanSTARRS DR1 g': undefined,
        '2MASS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_2MASS_color.jpg',
        'AKARI colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_AKARI_FIS_Color.jpg',
        'SWIFT': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SWIFT_BAT_FLUX.jpg',
        'VTSS-Ha': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Finkbeiner.jpg',
        'XMM PN colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_XMM_PN_color.jpg',
        'SDSS9 colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SDSS9_color.jpg',
    };

    // Constructor
    constructor(aladin) {
        super(aladin, {hideOnClick: false});
        this.aladin = aladin;
        //this.anchor = menu.controls["Stack"];
        //this.fsm = new StackLayerOpenerFSM(aladin, menu);

        /*window.addEventListener("resize", (e) => {
            this._hide();
        })*/

        /*document.addEventListener('click', e => {
            if (!self.el.contains(e.target) && this.mode === 'stack') {
                this._hide()
            }
        });*/
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
            subMenu: [
                {
                    label: {
                        icon: {
                            iconURL: searchIconUrl,
                            tooltip: {content: 'From our database...', position: { direction: 'right' }},
                            cssStyle: {
                                backgroundPosition: 'center center',
                                backgroundColor: '#bababa',
                                border: '1px solid rgb(72, 72, 72)',
                                cursor: 'help',
                            },
                        },
                        content: 'Search a survey'
                    },
                    action(e) {
                        /*if (e) {
                            e.stopPropagation();
                            e.preventDefault();
                        }*/

                        self._hide();

                        self.hipsSelectorBox = new HiPSSelectorBox({
                                position: self.position,
                            }, 
                            (HiPSId) => {
                                let name = Utils.uuidv4()
                                self.aladin.setOverlayImageLayer(HiPSId, name)

                                self.mode = 'stack';
                                self._show();
                            },
                            self.aladin
                        );

                        self.hipsSelectorBox._show();

                        self.mode = 'hips';
                    }
                },
                ContextMenu.fileLoaderItem({
                    label: 'FITS image file',
                    accept: '.fits',
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
                                //self.aladin.selectLayer(image.layer);
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
                iconURL: layer.getOpacity() === 0.0 ? hideIconUrl : showIconUrl,
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                    width: '18px',
                    height: '18px',
                    verticalAlign: 'middle',
                    marginRight: '2px',
                },
                tooltip: {content: layer.getOpacity() === 0.0 ? 'Show' : 'Hide', position: {direction: 'bottom'}},
                action(e, btn) {
                    e.preventDefault();
                    e.stopPropagation();
                    
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
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
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
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
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

                    let editBox = LayerEditBox.getInstance(self.aladin, {position: self.position});
                    editBox.update({layer})
                    editBox._show();

                    self.mode = 'edit';
                }
            });

            let layerClassName = 'a' + layer.layer.replace(/[.\/ ]/g, '')

            let item = Layout.horizontal({
                layout: [
                    '<div class="' + layerClassName + '" style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;' + (selectedLayer === layer.layer ? 'border: 1px solid white;' : '') + '">' + name + '</div>',
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

            let l = {
                label: item,
                cssStyle,
                hover(e) {
                    showBtn.el.style.visibility = 'visible'
                    editBtn.el.style.visibility = 'visible'
                    deleteBtn.el.style.visibility = 'visible'
                },
                unhover(e) {
                    showBtn.el.style.visibility = 'hidden'
                    editBtn.el.style.visibility = 'hidden'
                    deleteBtn.el.style.visibility = 'hidden'
                }
            };

            if (layer.layer === "base") {
                l.subMenu = [{
                    label: {
                        icon: {
                            iconURL: searchIconImg,
                            tooltip: {content: 'Find a specific survey <br /> in our database...', position: { direction: 'bottom' }},
                            cssStyle: {
                                backgroundPosition: 'center center',
                                backgroundColor: '#bababa',
                                border: '1px solid rgb(72, 72, 72)',
                                cursor: 'help',
                            },
                        },
                        content: 'Search for a new survey'
                    },
                    action(o) {
                        self._hide();
        
                        self.hipsBox = new HiPSSelectorBox({
                                position: self.position,
                            },
                            (HiPSId) => {            
                                self.aladin.setOverlayImageLayer(HiPSId, 'base');
                                self.mode = 'stack';
                                self._show();
                            },
                            self.aladin
                        );
        
                        self.hipsBox._show()
        
                        self.mode = 'hips';
                    }
                }];
        
                let layers = ImageLayer.LAYERS.sort(function (a, b) {
                    if (!a.order) {
                        return a.name > b.name ? 1 : -1;
                    }
                    return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
                });
        
                for(let layer of layers) {
                    let backgroundUrl = Stack.previewImagesUrl[layer.name];
                    let cssStyle = {
                        height: '2.5em',
                    };
                    if (backgroundUrl) {
                        cssStyle = {
                            backgroundSize: '100%',
                            backgroundImage: 'url(' + backgroundUrl + ')',
                            ...cssStyle
                        }
                    }
        
                    l.subMenu.push({
                        //selected: layer.name === aladin.getBaseImageLayer().name,
                        label: '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px">' + layer.name + '</div>',
                        cssStyle: cssStyle,
                        action(e) {
                            let cfg = ImageLayer.LAYERS.find((l) => l.name === layer.name);
                            let newLayer;
                            
                            // Max order is specific for surveys
                            if (cfg.subtype === "fits") {
                                // FITS
                                newLayer = self.aladin.createImageFITS(
                                    cfg.url,
                                    cfg.name,
                                    cfg.options,
                                );
                            } else {
                                // HiPS
                                newLayer = self.aladin.createImageSurvey(
                                    cfg.id,
                                    cfg.name,
                                    cfg.url,
                                    undefined,
                                    cfg.maxOrder,
                                    cfg.options
                                );
                            }
                
                            self.aladin.setOverlayImageLayer(newLayer, 'base');
                            //self._hide();
                        },
                        hover(e, item) {
                            item.style.filter = 'brightness(1.5)';
                        },
                        unhover(e, item) {
                            item.style.filter = 'brightness(1.0)';
                        }
                    })
                }
            }

            l.action = (o) => {
                let oldLayerClassName = 'a' + self.aladin.getSelectedLayer().replace(/[.\/ ]/g, '')
                self.el.querySelector('.' +  oldLayerClassName).style.removeProperty('border')
                self.aladin.selectLayer(layer.layer);
                self.el.querySelector('.' + layerClassName).style.border = '1px solid white';
            }

            layout.push(l);
        }

        super.attach(layout);
    }

    _findPreviewImageUrl(layer) {
        if (layer instanceof ImageFITS) {
            return;
        }

        if (!layer.properties || !layer.properties.creatorDid) {
            return;
        }

        const creatorDid = layer.properties.creatorDid;
        
        for (const key in Stack.previewImagesUrl) {
            if (creatorDid.includes(key)) {
                return Stack.previewImagesUrl[key];
            }
        }
    }

    _show(options) {
        this.position = (options && options.position) || this.position || { anchor: 'center center'}; 

        super.show({
            position: this.position,
            cssStyle: {
                maxWidth: '15em',
                backgroundColor: 'black',
            }
        })

        let subMenus = this.element().querySelectorAll(".aladin-context-sub-menu");
        
        let defaultHiPSMenu = subMenus[subMenus.length - 1];
        defaultHiPSMenu.style.maxHeight = Math.min(500, this.aladin.aladinDiv.offsetHeight) + 'px';
        defaultHiPSMenu.style.overflowY = 'scroll';
    }

    _hide() {
        // go back to the display stack state
        if (this.position) {
            let editBox = LayerEditBox.getInstance(this.aladin, {position: this.position});
            editBox._hide();
        }

        if (this.hipsSelectorBox) {
            this.hipsSelectorBox._hide();
        }

        this.mode = 'stack';

        super._hide();
    }
}
   