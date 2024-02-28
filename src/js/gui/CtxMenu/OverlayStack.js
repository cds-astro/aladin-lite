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
import { CatalogQueryBox } from "../Box/CatalogQueryBox.js";
 import { ALEvent } from "../../events/ALEvent.js";
 import { Layout } from "../Layout.js";
 import { ContextMenu } from "../Widgets/ContextMenu.js";
 import { ActionButton } from "../Widgets/ActionButton.js";
 import { AladinUtils } from "../../AladinUtils.js";
import A from "../../A.js";
import { Utils } from "../../../js/Utils";
import { View } from "../../View.js";
import { LayerEditBox } from "../Box/SurveyEditBox.js";
import { HiPSSelectorBox } from "../Box/HiPSSelectorBox.js";
import searchIconUrl from '../../../../assets/icons/search.svg';
import showIconUrl from '../../../../assets/icons/show.svg';
import hideIconUrl from '../../../../assets/icons/hide.svg';
import removeIconUrl from '../../../../assets/icons/remove.svg';
import editIconUrl from '../../../../assets/icons/edit.svg';
import { ImageFITS } from "../../ImageFITS.js";
import { ImageLayer } from "../../ImageLayer.js";
import searchIconImg from '../../../../assets/icons/search.svg';
import { TogglerActionButton } from "../Button/Toggler.js";
import { Icon } from "../Widgets/Icon.js";
 
export class OverlayStack extends ContextMenu {
    static previewImagesUrl = {
        'AllWISE color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_allWISE_color.jpg',
        'DSS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_color.jpg',
        'DSS2 Red (F+R)': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_DSS2_red.jpg',
        'Fermi color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Fermi_color.jpg',
        'GALEXGR6_7 NUV': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GALEXGR6_7_color.jpg',
        'GLIMPSE360': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_GLIMPSE360.jpg',
        'Halpha': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_VTSS_Ha.jpg',
        'IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SPITZER_color.jpg',
        'IRIS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_IRIS_color.jpg',
        'Mellinger colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Mellinger_color.jpg',
        'PanSTARRS DR1 color': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_PanSTARRS_DR1_color-z-zg-g.jpg',
        '2MASS colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_2MASS_color.jpg',
        'AKARI colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_AKARI_FIS_Color.jpg',
        'SWIFT': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SWIFT_BAT_FLUX.jpg',
        'VTSS-Ha': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_Finkbeiner.jpg',
        'XMM PN colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_XMM_PN_color.jpg',
        'SDSS9 colored': 'https://aladin.cds.unistra.fr/AladinLite/survey-previews/P_SDSS9_color.jpg',
    };
    static predefinedCats = {
        simbad: {url: 'https://axel.u-strasbg.fr/HiPSCatService/SIMBAD', options: {id: 'simbad', name: 'SIMBAD', shape: 'circle', sourceSize: 8, color: '#318d80', onClick: 'showTable'}},
        gaia: {url: 'https://axel.u-strasbg.fr/HiPSCatService/I/355/gaiadr3', options: {id: 'gaia-dr3', name: 'Gaia DR3', shape: 'square', sourceSize: 8, color: '#6baed6', onClick: 'showTable'}},
        twomass: {url: 'https://axel.u-strasbg.fr/HiPSCatService/II/246/out', options: {id: '2mass', name: '2MASS', shape: 'plus', sourceSize: 8, color: '#dd2233', onClick: 'showTable'}}
    };
    // Constructor
    constructor(aladin) {
        let self;
        super(aladin, {hideOnClick: (e) => {
            if (self.mode === 'stack') {
                self._hide();
            }
        }});
        self = this;
        this.aladin = aladin;

        this.mode = 'stack';

        this._addListeners();

        this.mocHiPSUrls = {}
    }

    _addListeners() {
        let self = this;

        let updateOverlayList = () => {
            // If it is shown, update it
            if (!self.isHidden) {
                // show will update the content of the stack
                self._show();
            }
        };
        
        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });
        
        ALEvent.HIPS_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.HIPS_LAYER_RENAMED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();    
        });

        ALEvent.HIPS_LAYER_SWAP.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        updateOverlayList();
    }

    attach() {
        let self = this;

        const overlays = Array.from(this.aladin.getOverlays()).reverse().map((overlay) => {
            return overlay;
        });
        const layers = Array.from(self.aladin.getImageOverlays()).reverse().map((name) => {
            let overlay = self.aladin.getOverlayImageLayer(name);
            return overlay;
        });


        let layout = [{
            label: 'Add overlay',
            subMenu: [
                {
                    label: 'Catalogue',
                    subMenu: [
                        {
                            label: {
                                icon: {
                                    url: 'https://aladin.cds.unistra.fr/AladinLite/logos/SIMBAD.svg',
                                    cssStyle: {
                                        width: '3rem',
                                        height: '3rem',
                                        cursor: 'help',
                                    },
                                    action(o) {
                                        window.open('https://simbad.cds.unistra.fr/simbad/')
                                    }
                                },
                                content: 'database',
                                tooltip: {content: 'Click to go to the SIMBAD database', position: {direction: 'bottom'}},

                            },
                            action(o) {
                                o.stopPropagation();
                                o.preventDefault();
                                
                                self._hide();

                                const simbadHiPS = A.catalogHiPS(OverlayStack.predefinedCats.simbad.url, OverlayStack.predefinedCats.simbad.options);
                                self.aladin.addCatalog(simbadHiPS);

                                self.mode = 'stack';
                            }
                        },
                        {
                            label: 'Gaia DR3',
                            action(o) {
                                o.stopPropagation();
                                o.preventDefault();
                                
                                self._hide();

                                const simbadHiPS = A.catalogHiPS(OverlayStack.predefinedCats.gaia.url, OverlayStack.predefinedCats.gaia.options);
                                self.aladin.addCatalog(simbadHiPS);

                                self.mode = 'stack';
                            }
                        },
                        {
                            label: '2MASS',
                            action(o) {
                                o.stopPropagation();
                                o.preventDefault();
                                
                                self._hide();

                                const simbadHiPS = A.catalogHiPS(OverlayStack.predefinedCats.twomass.url, OverlayStack.predefinedCats.twomass.options);
                                self.aladin.addCatalog(simbadHiPS);

                                self.mode = 'stack';
                            }
                        },
                        ContextMenu.fileLoaderItem({
                            label: 'From a VOTable File',
                            accept: '.xml,.vot',
                            action(file) {
                                let url = URL.createObjectURL(file);

                                A.catalogFromURL(
                                    url,
                                    {onClick: 'showTable'},
                                    (catalog) => {
                                        self.aladin.addCatalog(catalog)
                                    },
                                    e => alert(e)
                                );
                            }
                        }),
                        {
                            label: {
                                icon: {
                                    url: searchIconImg,
                                    monochrome: true,
                                    tooltip: {content: 'Find a specific catalogue <br /> in our database...', position: { direction: 'top' }},
                                    cssStyle: {
                                        cursor: 'help',
                                    },
                                },
                                content: 'More...'
                            },
                            action(o) {
                                o.stopPropagation();
                                o.preventDefault();
                                
                                self._hide();

                                let catBox = CatalogQueryBox.getInstance(self.aladin);
                                catBox._show({position: self.position});

                                self.mode = 'search';
                            }
                        },
                    ]
                },
                {
                    label: {
                        icon: {
                            url: Icon.dataURLFromSVG({svg: Icon.SVG_ICONS.MOC}),
                            size: 'small',
                            tooltip: {content: 'Define a selection coverage', position: {direction: 'bottom'}},
                            monochrome: true,
                            cssStyle: {
                                cursor: 'pointer',
                            },
                        },
                        content: 'MOC'
                    },
                    subMenu: [
                        ContextMenu.fileLoaderItem({
                            label: 'FITS File',
                            accept: '.fits',
                            action(file) {
                                let url = URL.createObjectURL(file);

                                let moc = A.MOCFromURL(
                                    url,
                                    {name: file.name, lineWidth: 3.0},
                                );
                                self.aladin.addMOC(moc)
                            }
                        }),
                        {
                            label: 'From selection',
                            subMenu: [
                                {
                                    label: '◌ Circle',
                                    disabled: self.aladin.view.mode !== View.PAN ? {
                                        reason: 'Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)'
                                    } : false,
                                    action(o) {
                                        o.preventDefault();
                                        o.stopPropagation();

                                        self._hide();

                                        self.aladin.select('circle', c => {
                                            let [ra, dec] = self.aladin.pix2world(c.x, c.y);
                                            let radius = self.aladin.angularDist(c.x, c.y, c.x + c.r, c.y);

                                            if (ra && dec && radius) {
                                                let moc = A.MOCFromCircle(
                                                    {ra, dec, radius},
                                                    {name: 'cone', lineWidth: 3.0},
                                                );
                                                self.aladin.addMOC(moc)
                                            } else {
                                                alert('The circle selection might be invalid. ra: ' + ra + 'deg, dec: ' + dec + 'deg, radius: ' + radius + 'deg')
                                            }
                                        })
                                    }
                                },
                                {
                                    label: '⬚ Rect',
                                    disabled: self.aladin.view.mode !== View.PAN ? {
                                        reason: 'Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)'
                                    } : false,
                                    action(o) {
                                        o.stopPropagation();
                                        o.preventDefault();

                                        self._hide();

                                        self.aladin.select('rect', r => {
                                            try {
                                                let [ra1, dec1] = self.aladin.pix2world(r.x, r.y);
                                                let [ra2, dec2] = self.aladin.pix2world(r.x + r.w, r.y);
                                                let [ra3, dec3] = self.aladin.pix2world(r.x + r.w, r.y + r.h);
                                                let [ra4, dec4] = self.aladin.pix2world(r.x, r.y + r.h);
    
                                                let moc = A.MOCFromPolygon(
                                                    {
                                                        ra: [ra1, ra2, ra3, ra4],
                                                        dec: [dec1, dec2, dec3, dec4]
                                                    },
                                                    {name: 'rect', lineWidth: 3.0},
                                                );
                                                self.aladin.addMOC(moc)
                                            } catch(_) {
                                                alert('Selection covers a region out of the projection definition domain.');
                                            }
                                        })
                                    }
                                },
                                {
                                    label: '⛉ Polygon',
                                    disabled: self.aladin.view.mode !== View.PAN ? {
                                        reason: 'Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)'
                                    } : false,
                                    action(o) {
                                        o.stopPropagation();
                                        o.preventDefault();

                                        self._hide();

                                        self.aladin.select('poly', p => {
                                            try {
                                                let ra = []
                                                let dec = []
                                                for (const v of p.vertices) {
                                                    let [lon, lat] = self.aladin.pix2world(v.x, v.y);
                                                    ra.push(lon)
                                                    dec.push(lat)
                                                }
                                                
                                                let moc = A.MOCFromPolygon(
                                                    {ra, dec},
                                                    {name: 'poly', lineWidth: 3.0},
                                                );
                                                self.aladin.addMOC(moc)

                                            } catch(_) {
                                                alert('Selection covers a region out of the projection definition domain.');
                                            }
                                        })
                                    }
                                },
                            ]
                        }
                    ]                    
                }
            ]
        }];

        for(const overlay of overlays) {
            const name = overlay.name;
            let cssStyle = {
                height: 'fit-content',
            };
            let showBtn = new ActionButton({
                size: 'small',
                icon: {
                    url: overlay.isShowing ? showIconUrl : hideIconUrl,
                    monochrome: true,
                },
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                tooltip: {content: overlay.isShowing ? 'Hide' : 'Show', position: {direction: 'bottom'}},
                action(e, btn) {
                    if (overlay.isShowing) {
                        overlay.hide()
                        btn.update({icon: {monochrome: true, url: hideIconUrl}, tooltip: {content: 'Show'}});
                    } else {
                        overlay.show()
                        btn.update({icon: {monochrome: true, url: showIconUrl}, tooltip: {content: 'Hide'}});
                    }
                }
            });

            let deleteBtn = new ActionButton({
                icon: {
                    url: removeIconUrl,
                    monochrome: true,
                },
                size: 'small',
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                tooltip: {
                    content: 'Remove',
                    position: {direction: 'bottom'}
                },
                action(e) {
                    self.aladin.removeLayer(overlay)
                }
            });

            let item = Layout.horizontal({
                layout: [
                    this._addOverlayIcon(overlay),
                    '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;">' + name + '</div>',
                    Layout.horizontal({layout: [showBtn, deleteBtn]})
                ],
                cssStyle: {
                    textAlign: 'center',
                }
            });

            if(!Utils.hasTouchScreen()) {
                layout.push({
                    label: item,
                    cssStyle,
                    hover(e) {
                        showBtn.el.style.visibility = 'visible'
                        deleteBtn.el.style.visibility = 'visible'
                    },
                    unhover(e) {
                        showBtn.el.style.visibility = 'hidden'
                        deleteBtn.el.style.visibility = 'hidden'
                    },
                })
            } else {
                layout.push({
                    label: item,
                    cssStyle
                })
            }
        }

        layout.push({
            label: 'Add survey',
            subMenu: [
                {
                    label: {
                        icon: {
                            url: searchIconUrl,
                            monochrome: true,
                            tooltip: {content: 'From our database...', position: { direction: 'right' }},
                            cssStyle: {
                                cursor: 'help',
                            },
                        },
                        content: 'Search for a survey'
                    },
                    action: (e) => {
                        e.stopPropagation();
                        e.preventDefault();

                        self._hide();

                        let hipsSelectorBox = HiPSSelectorBox.getInstance(self.aladin);
                        // attach a callback
                        hipsSelectorBox.attach( 
                            (HiPSId) => {
                                let name = Utils.uuidv4()
                                self.aladin.setOverlayImageLayer(HiPSId, name)

                                self.mode = 'stack';
                                self._show();
                            }
                        );

                        hipsSelectorBox._show({
                            position: self.position,
                        });

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
        })

        // survey list
        let selectedLayer = self.aladin.getSelectedLayer();

        /*if (!layers) {
            super.attach(layout);
            return;
        }*/

        const defaultLayers = ImageLayer.LAYERS.sort(function (a, b) {
            if (!a.order) {
                return a.name > b.name ? 1 : -1;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });

        for(const layer of layers) {
            let backgroundUrl = layer.properties.url + '/preview.jpg';
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

            let showBtn = ActionButton.createSmallSizedIconBtn({
                icon: {
                    url: layer.getOpacity() === 0.0 ? hideIconUrl : showIconUrl,
                    monochrome: true,
                },
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                tooltip: {content: layer.getOpacity() === 0.0 ? 'Show' : 'Hide', position: {direction: 'bottom'}},
                action(e, btn) {
                    e.preventDefault();
                    e.stopPropagation();
                    
                    let opacity = layer.getOpacity();
                    if (opacity === 0.0) {
                        layer.setOpacity(1.0);
                        btn.update({icon: {monochrome: true, url: showIconUrl}, tooltip: {content: 'Hide'}});
                    } else {
                        layer.setOpacity(0.0);
                        btn.update({icon: {monochrome: true, url: hideIconUrl}, tooltip: {content: 'Show'}});
                    }
                }
            });

            let deleteBtn = ActionButton.createSmallSizedIconBtn({
                icon: {url: removeIconUrl, monochrome: true},
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                disable: layer.layer === 'base',
                tooltip: {content: 'Remove', position: {direction: 'bottom'}},
                action(e) {
                    self.aladin.removeImageLayer(layer.layer);
                }
            });

            let editBtn = ActionButton.createSmallSizedIconBtn({
                icon: {url: editIconUrl, monochrome: true},
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                tooltip: {content: 'Edit', position: {direction: 'bottom'}},
                action: (e) => {
                    e.stopPropagation();
                    e.preventDefault();

                    self._hide();

                    self.aladin.selectLayer(layer.layer);
                    self.attach()

                    let editBox = LayerEditBox.getInstance(self.aladin);
                    editBox.update({layer})
                    editBox._show({position: self.position});

                    self.mode = 'edit';
                }
            });

            let loadMOCBtn = new TogglerActionButton({
                size: 'small',
                cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },
                icon: {url: Icon.dataURLFromSVG({svg: Icon.SVG_ICONS.MOC}), monochrome: true},
                tooltip: {content: 'Add coverage', position: {direction: 'bottom'}},
                toggled: (() => {
                    let overlays = self.aladin.getOverlays();
                    let found = overlays.find((o) => o.type === "moc" && o.name === layer.properties.obsTitle);
                    return found !== undefined;
                })(),
                actionOn: (e) => {
                    let moc = A.MOCFromURL(layer.properties.url + '/Moc.fits', {lineWidth: 3, name: layer.properties.obsTitle});
                    self.aladin.addMOC(moc);

                    self.mocHiPSUrls[layer.properties.url] = moc;
                    loadMOCBtn.update({tooltip: {content: 'Remove coverage', position: {direction: 'bottom'}}})

                    if (self.aladin.statusBar) {
                        self.aladin.statusBar.appendMessage({
                            message: 'Coverage of ' + layer.properties.obsTitle + ' loaded',
                            duration: 2000,
                            type: 'info'
                        })
                    }
                },
                actionOff: (e) => {
                    let moc = self.mocHiPSUrls[layer.properties.url];
                    self.aladin.removeLayer(moc)

                    delete self.mocHiPSUrls[layer.properties.url];

                    loadMOCBtn.update({tooltip: {content: 'Add coverage', position: {direction: 'bottom'}}})

                    if (self.aladin.statusBar) {
                        self.aladin.statusBar.appendMessage({
                            message: 'Coverage of ' + layer.properties.obsTitle + ' removed',
                            duration: 2000,
                            type: 'info'
                        })
                    }
                }
            });

            let layerClassName = 'a' + layer.layer.replace(/[.\/ ]/g, '')

            let btns = [showBtn, editBtn];

            if (layer.subtype !== 'fits') {
                btns.push(loadMOCBtn)
            }
            btns.push(deleteBtn)

            let item = Layout.horizontal({
                layout: [
                    '<div class="' + layerClassName + '" style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;' + (selectedLayer === layer.layer ? 'border: 1px solid white;' : '') + '">' + (layer.name) + '</div>',
                    Layout.horizontal({layout: btns})
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
                classList: 'surveyItem',
                cssStyle,
                hover(e) {
                    showBtn.el.style.visibility = 'visible'
                    editBtn.el.style.visibility = 'visible'
                    deleteBtn.el.style.visibility = 'visible'
                    loadMOCBtn.el.style.visibility = 'visible'
                },
                unhover(e) {
                    showBtn.el.style.visibility = 'hidden'
                    editBtn.el.style.visibility = 'hidden'
                    deleteBtn.el.style.visibility = 'hidden'
                    loadMOCBtn.el.style.visibility = 'hidden'
                }
            };

            l.subMenu = [];
    
            for(let ll of defaultLayers) {
                backgroundUrl = OverlayStack.previewImagesUrl[ll.name];
                if (!backgroundUrl) {
                    backgroundUrl = ll.url + '/preview.jpg'
                }
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
                    label: '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px">' + ll.name + '</div>',
                    cssStyle: cssStyle,
                    action(e) {
                        let cfg = ImageLayer.LAYERS.find((l) => l.name === ll.name);
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
            
                        self.aladin.setOverlayImageLayer(newLayer, layer.layer);
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

            l.subMenu.push({
                label: {
                    icon: {
                        url: searchIconImg,
                        monochrome: true,
                        tooltip: {content: 'Find a specific survey <br /> in our database...', position: { direction: 'top' }},
                        cssStyle: {
                            cursor: 'help',
                        },
                    },
                    content: 'More...'
                },
                action(o) {
                    o.stopPropagation();
                    o.preventDefault();

                    self._hide();

                    let hipsBox = HiPSSelectorBox.getInstance(self.aladin)
                    
                    hipsBox.attach(
                        (HiPSId) => {            
                            self.aladin.setOverlayImageLayer(HiPSId, layer.layer);
                            self.mode = 'stack';
                            self._show();
                        }
                    );
    
                    hipsBox._show({
                        position: self.position,
                    })
    
                    self.mode = 'hips';
                }
            })

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
        // if not found
        return layer.properties.url + '/preview.jpg'
    }

    _addOverlayIcon(overlay) {
        var tooltipText;
        var svg = '';
        if (overlay.type == 'catalog' || overlay.type == 'progressivecat') {
            var nbSources = overlay.getSources().length;
            tooltipText = nbSources + ' source' + (nbSources > 1 ? 's' : '');

            svg = Icon.SVG_ICONS.CATALOG;
        }
        else if (overlay.type == 'moc') {
            tooltipText = 'Coverage: ' + (100 * overlay.skyFraction()).toFixed(2) + ' % of sky';

            svg = Icon.SVG_ICONS.MOC;
        }
        else if (overlay.type == 'overlay') {
            svg = Icon.SVG_ICONS.OVERLAY;
        }

        let tooltip;
        if (tooltipText) {
            tooltip = { content: tooltipText, position: {direction: 'bottom'} }
        }

        // retrieve SVG icon, and apply the layer color
        return new Icon({
            size: 'small',
            url: Icon.dataURLFromSVG({svg, color: overlay.color}),
            tooltip
        });
    }

    _show(options) {
        this.attach();

        this.position = (options && options.position) || this.position || { anchor: 'center center'}; 
        super.show({
            ...options,
            ...{position: this.position},
            cssStyle: {
                backgroundColor: 'black',
                maxWidth: '20em',
                //border: '1px solid white',
            }
        })

        this.element().querySelectorAll(".surveyItem")
            .forEach((surveyItem) => {
                surveyItem.querySelectorAll(".aladin-context-sub-menu")
                    // skip the first menu
                    .forEach((subMenu) => {
                        subMenu.style.maxHeight = '50vh';
                        subMenu.style.overflowY = 'scroll';
                    })
            })
            
    }

    _hide() {
        let catBox = CatalogQueryBox.getInstance(this.aladin);
        catBox._hide();

        let editBox = LayerEditBox.getInstance(this.aladin);
        editBox._hide();

        let hipsSelectorBox = HiPSSelectorBox.getInstance(this.aladin);
        hipsSelectorBox._hide();

        this.mode = 'stack';
       
        super._hide();
    }
    
    static singleton;

    static getInstance(aladin) {
        if (!OverlayStack.singleton) {
            OverlayStack.singleton = new OverlayStack(aladin);
        }

        return OverlayStack.singleton;
    }
}
