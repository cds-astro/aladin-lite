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
 import showIconUrl from '../../../../assets/icons/show.svg';
 import hideIconUrl from '../../../../assets/icons/hide.svg';
 import removeIconUrl from '../../../../assets/icons/remove.svg';
 import { AladinUtils } from "../../AladinUtils.js";
import A from "../../A.js";
import { Utils } from "../../../js/Utils";
 
export class OverlayStack extends ContextMenu {

    static predefinedCats = {
        simbad: {url: 'https://axel.u-strasbg.fr/HiPSCatService/SIMBAD', options: {id: 'simbad', name: 'SIMBAD', shape: 'circle', sourceSize: 8, color: '#318d80'}},
        gaia: {url: 'https://axel.u-strasbg.fr/HiPSCatService/I/355/gaiadr3', options: {id: 'gaia-dr3', name: 'Gaia DR3', shape: 'square', sourceSize: 8, color: '#6baed6'}},
        twomass: {url: 'https://axel.u-strasbg.fr/HiPSCatService/II/246/out', options: {id: '2mass', name: '2MASS', shape: 'plus', sourceSize: 8, color: '#dd2233'}}
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
    }

    _addListeners() {
        let self = this;

        let updateImageList = () => {
            // If it is shown, update it
            if (!self.isHidden) {
                // show will update the content of the stack
                self._show();
            }
        };

        updateImageList();
        
        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateImageList();
        });
    }

    attach() {
        const overlays = Array.from(this.aladin.getOverlays()).reverse().map((overlay) => {
            return overlay;
        });

        let self = this;

        let layout = [{
            label: 'Add overlay',
            subMenu: [
                {
                    label: 'Catalogue',
                    subMenu: [
                        {
                            label: 'Simbad',
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
                            label: 'More...',
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
                    label: 'MOC',
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
                                    label: 'Circle',
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
                                    label: 'Rect',
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
                                    label: 'Polygon',
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
                iconURL:  overlay.isShowing ? showIconUrl : hideIconUrl,
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                    verticalAlign: 'middle',
                    marginRight: '2px',
                },
                tooltip: {content: overlay.isShowing ? 'Hide' : 'Show', position: {direction: 'bottom'}},
                action(e, btn) {
                    if (overlay.isShowing) {
                        overlay.hide()
                        btn.update({iconURL: hideIconUrl, tooltip: {content: 'Show'}});
                    } else {
                        overlay.show()
                        btn.update({iconURL: showIconUrl, tooltip: {content: 'Hide'}});
                    }
                }
            });

            let deleteBtn = new ActionButton({
                iconURL: removeIconUrl,
                size: 'small',
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                    verticalAlign: 'middle'
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

        super.attach(layout);
    }

    _addOverlayIcon(overlay) {
        var tooltipText = '';
        var iconSvg = '';
        if (overlay.type == 'catalog' || overlay.type == 'progressivecat') {
            var nbSources = overlay.getSources().length;
            tooltipText = nbSources + ' source' + (nbSources > 1 ? 's' : '');

            iconSvg = AladinUtils.SVG_ICONS.CATALOG;
        }
        else if (overlay.type == 'moc') {
            tooltipText = 'Coverage: ' + (100 * overlay.skyFraction()).toFixed(2) + ' % of sky';

            iconSvg = AladinUtils.SVG_ICONS.MOC;
        }
        else if (overlay.type == 'overlay') {
            iconSvg = AladinUtils.SVG_ICONS.OVERLAY;
        }

        let changeSVGSize = (svg, size) => {
            let str = svg.replace(/FILLCOLOR/g, 'black')
            let elt = document.createElement('div');
            elt.innerHTML = str;

            elt.querySelector('svg').setAttribute('width', size);
            elt.querySelector('svg').setAttribute('height', size);

            return elt.innerHTML;
        };
        // retrieve SVG icon, and apply the layer color
        var svgBase64 = window.btoa(changeSVGSize(iconSvg.replace(/FILLCOLOR/g, overlay.color), '1rem'));
        let icon = new ActionButton({
            size: 'small',
            tooltip: {content: tooltipText, position: {direction: 'bottom'}},
            cssStyle: {
                backgroundImage: 'url("data:image/svg+xml;base64,' + svgBase64 + '")',
            }
        });
        icon.addClass('svg-icon')

        return icon;
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
    }

    _hide() {
        this.mode = 'stack';
        
        let catBox = CatalogQueryBox.getInstance(this.aladin);
        catBox._hide();
       
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
