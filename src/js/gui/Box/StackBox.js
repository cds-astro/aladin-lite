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

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/
import { CatalogQueryBox } from "./CatalogQueryBox.js";
import { ALEvent } from "../../events/ALEvent.js";
import { Layout } from "../Layout.js";
import { ContextMenu } from "../Widgets/ContextMenu.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import A from "../../A.js";
import { Utils } from "../../Utils";
import { View } from "../../View.js";
import { HiPSSettingsBox } from "./HiPSSettingsBox.js";
import hipsIconUrl from "../../../../assets/icons/hips.svg";
import treeIconUrl from "../../../../assets/icons/tree.svg";
import showIconUrl from "../../../../assets/icons/show.svg";
import addIconUrl from "../../../../assets/icons/plus.svg";
import hideIconUrl from "../../../../assets/icons/hide.svg";
import removeIconUrl from "../../../../assets/icons/remove.svg";
import settingsIconUrl from "../../../../assets/icons/settings.svg";
import searchIconImg from "../../../../assets/icons/search.svg";
import downloadIconUrl from '../../../../assets/icons/download.svg';
import swapIcon from '../../../../assets/icons/swap.svg'
import { WidgetTogglerButton } from "../Button/Toggler.js";
import { Icon } from "../Widgets/Icon.js";
import { Box } from "../Widgets/Box.js";
import { CtxMenuActionButtonOpener } from "../Button/CtxMenuOpener.js";
import { Image } from "../../Image.js";
import { HiPSBrowserBox } from "./HiPSBrowserBox.js";
import { HiPSCompositeBox } from "./HiPSCompositeBox.js"
import { Catalog } from "../../Catalog.js";
import { ProgressiveCat } from "../../ProgressiveCat.js";
import { Form } from "../Widgets/Form.js";
import { LayerSelector } from "../Input/LayerSelector.js";
import { HiPS } from "../../HiPS.js";

export class OverlayStackBox extends Box {
    /*static previewImagesUrl = {
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
    };*/
    static predefinedCats = {
        simbad: {
            url: "https://axel.cds.unistra.fr/HiPSCatService/SIMBAD",
            options: {
                id: "simbad",
                name: "SIMBAD",
                shape: "circle",
                sourceSize: 8,
                color: "#318d80",
                hoverColor: 'red',
                onlyFootprints: false,
                onClick: "showTable",
                shape: (s) => {
                    let galaxy = ["Seyfert","Seyfert_1", "Seyfert_2","LSB_G","PartofG","RadioG","Gin","GinPair","HII_G","LensedG","BClG","BlueCompG","EmG","GinCl","GinGroup","StarburstG","LINER","AGN", "Galaxy", "GtowardsGroup", "GtowardsCl", "BrightestCG"].some((n) => s.data.main_type.indexOf(n) >= 0);
                    if (!galaxy) return;

                    let a = +s.data.size_maj;
                    let b = +s.data.size_min;

                    let theta = +s.data.size_angle || 0.0;
                    return A.ellipse(s.ra, s.dec, a / 60, b / 60, theta, { color: "cyan" });
                }
            },
        },
        gaia: {
            url: "https://axel.cds.unistra.fr/HiPSCatService/I/355/gaiadr3",
            options: {
                id: "gaia-dr3",
                name: "Gaia DR3",
                shape: "square",
                sourceSize: 8,
                color: "#6baed6",
                onClick: "showTable",
            },
        },
        twomass: {
            url: "https://axel.cds.unistra.fr/HiPSCatService/II/246/out",
            options: {
                id: "2mass",
                name: "2MASS",
                shape: "plus",
                sourceSize: 8,
                color: "#dd2233",
                onClick: "showTable",
            },
        },
    };
    // Constructor
    constructor(aladin) {
        super(
            {
                close: true,
                header: {
                    title: "Stack",
                },
                classList: ["aladin-stack-box"],
                content: [],
            },
            aladin.aladinDiv
        );
        this.aladin = aladin;

        this.mode = "stack";

        this._addListeners();

        this.ui = {};

        let self = this;
        // Add overlay button
        this.addOverlayBtn = new CtxMenuActionButtonOpener(
            {
                icon: {
                    url: addIconUrl,
                    size: "small",
                    monochrome: true,
                },
                openDirection: 'right',
                tooltip: {
                    content: "A catalog, MOC or footprint",
                    position: { direction: "top" },
                },
                ctxMenu: [
                    {
                        label: "Catalogue",
                        subMenu: [
                            {
                                label: {
                                    icon: {
                                        url: "https://aladin.cds.unistra.fr/AladinLite/logos/SIMBAD.svg",
                                        cssStyle: {
                                            width: "3rem",
                                            height: "3rem",
                                            cursor: "help",
                                        },
                                        action(o) {
                                            window.open(
                                                "https://simbad.cds.unistra.fr/simbad/"
                                            );
                                        },
                                    },
                                    content: "database",
                                    tooltip: {
                                        content:
                                            "Click to go to the SIMBAD database",
                                        position: { direction: "bottom" },
                                    },
                                },
                                action(o) {
                                    o.stopPropagation();
                                    o.preventDefault();

                                    const simbadHiPS = A.catalogHiPS(
                                        OverlayStackBox.predefinedCats.simbad
                                            .url,
                                        OverlayStackBox.predefinedCats.simbad
                                            .options
                                    );
                                    self.aladin.addCatalog(simbadHiPS);
                                },
                            },
                            {
                                label: "Gaia DR3",
                                action(o) {
                                    o.stopPropagation();
                                    o.preventDefault();

                                    const simbadHiPS = A.catalogHiPS(
                                        OverlayStackBox.predefinedCats.gaia.url,
                                        OverlayStackBox.predefinedCats.gaia
                                            .options
                                    );
                                    self.aladin.addCatalog(simbadHiPS);
                                },
                            },
                            {
                                label: "2MASS",
                                action(o) {
                                    o.stopPropagation();
                                    o.preventDefault();

                                    //self._hide();

                                    const simbadHiPS = A.catalogHiPS(
                                        OverlayStackBox.predefinedCats.twomass
                                            .url,
                                        OverlayStackBox.predefinedCats.twomass
                                            .options
                                    );
                                    self.aladin.addCatalog(simbadHiPS);
                                },
                            },
                            ContextMenu.fileLoaderItem({
                                label: "From a VOTable File",
                                accept: ".xml,.vot",
                                action(file) {
                                    let url = URL.createObjectURL(file);

                                    A.catalogFromURL(
                                        url,
                                        { onClick: "showTable" },
                                        (catalog) => {
                                            self.aladin.addCatalog(catalog);
                                        },
                                        (e) => alert(e)
                                    );
                                },
                            }),
                            {
                                label: {
                                    icon: {
                                        url: searchIconImg,
                                        monochrome: true,
                                        tooltip: {
                                            content:
                                                "Find a specific catalogue <br /> in our database...",
                                            position: { direction: "top" },
                                        },
                                        cssStyle: {
                                            cursor: "help",
                                        },
                                    },
                                    content: "Browse...",
                                },
                                action(o) {
                                    o.stopPropagation();
                                    o.preventDefault();

                                    if (!self.catBox)
                                        self.catBox = new CatalogQueryBox(aladin);

                                    self.catBox._show({position: {
                                        anchor: 'center center'
                                    }});
                                },
                            },
                        ],
                    },
                    {
                        label: {
                            icon: {
                                url: Icon.dataURLFromSVG({
                                    svg: Icon.SVG_ICONS.MOC,
                                }),
                                size: "small",
                                tooltip: {
                                    content: "Define a selection coverage",
                                    position: { direction: "bottom" },
                                },
                                monochrome: true,
                                cssStyle: {
                                    cursor: "pointer",
                                },
                            },
                            content: "MOC",
                        },
                        subMenu: [
                            ContextMenu.fileLoaderItem({
                                label: "FITS File",
                                accept: ".fits",
                                action(file) {
                                    let url = URL.createObjectURL(file);

                                    let moc = A.MOCFromURL(url, {
                                        name: file.name,
                                    });
                                    self.aladin.addMOC(moc);
                                },
                            }),
                            {
                                label: "From selection",
                                subMenu: [
                                    {
                                        label: "◌ Circle",
                                        disabled:
                                            self.aladin.view.mode !== View.PAN
                                                ? {
                                                      reason: "Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)",
                                                  }
                                                : false,
                                        action(o) {
                                            o.preventDefault();
                                            o.stopPropagation();

                                            self.aladin.select(
                                                "circle",
                                                (c) => {
                                                    try {
                                                        let [ra, dec] =
                                                            self.aladin.pix2world(
                                                                c.x,
                                                                c.y,
                                                                "icrs"
                                                            );
                                                        let radius =
                                                            self.aladin.angularDist(
                                                                c.x,
                                                                c.y,
                                                                c.x + c.r,
                                                                c.y
                                                            );

                                                        // the moc needs a
                                                        let moc = A.MOCFromCone(
                                                            { ra, dec, radius },
                                                            {
                                                                name: "cone",
                                                            }
                                                        );
                                                        self.aladin.addMOC(moc);
                                                    } catch {
                                                        console.error(
                                                            "Circle out of projection. Selection canceled"
                                                        );
                                                    }
                                                }
                                            );
                                        },
                                    },
                                    {
                                        label: "⬚ Rect",
                                        disabled:
                                            self.aladin.view.mode !== View.PAN
                                                ? {
                                                      reason: "Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)",
                                                  }
                                                : false,
                                        action(o) {
                                            o.stopPropagation();
                                            o.preventDefault();

                                            //self._hide();

                                            self.aladin.select("rect", (r) => {
                                                try {
                                                    let [ra1, dec1] =
                                                        self.aladin.pix2world(
                                                            r.x,
                                                            r.y,
                                                            "icrs"
                                                        );
                                                    let [ra2, dec2] =
                                                        self.aladin.pix2world(
                                                            r.x + r.w,
                                                            r.y,
                                                            "icrs"
                                                        );
                                                    let [ra3, dec3] =
                                                        self.aladin.pix2world(
                                                            r.x + r.w,
                                                            r.y + r.h,
                                                            "icrs"
                                                        );
                                                    let [ra4, dec4] =
                                                        self.aladin.pix2world(
                                                            r.x,
                                                            r.y + r.h,
                                                            "icrs"
                                                        );

                                                    let moc = A.MOCFromPolygon(
                                                        {
                                                            ra: [
                                                                ra1,
                                                                ra2,
                                                                ra3,
                                                                ra4,
                                                            ],
                                                            dec: [
                                                                dec1,
                                                                dec2,
                                                                dec3,
                                                                dec4,
                                                            ],
                                                        },
                                                        {
                                                            name: "rect",
                                                        }
                                                    );
                                                    self.aladin.addMOC(moc);
                                                } catch (_) {
                                                    alert(
                                                        "Selection covers a region out of the projection definition domain."
                                                    );
                                                }
                                            });
                                        },
                                    },
                                    {
                                        label: "⛉ Polygon",
                                        disabled:
                                            self.aladin.view.mode !== View.PAN
                                                ? {
                                                      reason: "Exit your current mode<br/>(e.g. disable the SIMBAD pointer mode)",
                                                  }
                                                : false,
                                        action(o) {
                                            o.stopPropagation();
                                            o.preventDefault();

                                            //self._hide();

                                            self.aladin.select("poly", (p) => {
                                                try {
                                                    let ra = [];
                                                    let dec = [];
                                                    for (const v of p.vertices) {
                                                        let [lon, lat] =
                                                            self.aladin.pix2world(
                                                                v.x,
                                                                v.y,
                                                                "icrs"
                                                            );
                                                        ra.push(lon);
                                                        dec.push(lat);
                                                    }

                                                    let moc = A.MOCFromPolygon(
                                                        { ra, dec },
                                                        {
                                                            name: "poly",
                                                        }
                                                    );
                                                    self.aladin.addMOC(moc);
                                                } catch (_) {
                                                    alert(
                                                        "Selection covers a region out of the projection definition domain."
                                                    );
                                                }
                                            });
                                        },
                                    },
                                ],
                            },
                        ],
                    },
                ],
            },
            this.aladin
        );

        this.addHiPSBtn = new CtxMenuActionButtonOpener(
            {
                icon: {
                    url: addIconUrl,
                    size: "small",
                    monochrome: true,
                },
                openDirection: 'right',
                ctxMenu: [
                    {
                        label: {
                            icon: {
                                url: addIconUrl,
                                monochrome: true,
                                tooltip: {
                                    content: "Add a new layer",
                                    position: { direction: "right" },
                                },
                                cssStyle: {
                                    cursor: "help",
                                },
                            },
                            content: "Add a new HiPS",
                        },
                        action: (e) => {
                            e.stopPropagation();
                            e.preventDefault();

                            self.aladin.addNewImageLayer(
                                'P/DSS2/color'
                            );
                        },
                    },
                    {
                        label: {
                            icon: {
                                url: treeIconUrl,
                                monochrome: true,
                                tooltip: {
                                    content: "From our database...",
                                    position: { direction: "right" },
                                },
                                cssStyle: {
                                    cursor: "help",
                                },
                            },
                            content: "Browse HiPS",
                        },
                        action: (e) => {
                            e.stopPropagation();
                            e.preventDefault();

                            if (!aladin.hipsBrowser)
                                aladin.hipsBrowser = new HiPSBrowserBox(aladin);

                            let newLayer = Utils.uuidv4();

                            aladin.hipsBrowser._show({
                                selected: (hips) => {
                                    aladin.setOverlayImageLayer(hips, newLayer);
                                },
                                position: {
                                    anchor: 'center center'
                                }
                            });
                        },
                    },
                    {
                        label: {
                            icon: {
                                url: hipsIconUrl,
                                monochrome: true,
                                tooltip: {
                                    content: "Combine different surveys into a color one!",
                                    position: { direction: "right" },
                                },
                                cssStyle: {
                                    cursor: "help",
                                },
                            },
                            content: "Add a composite HiPS",
                        },
                        disabled: true,
                        action: (e) => {
                            e.stopPropagation();
                            e.preventDefault();

                            if (!self.hipsCompositeBox)
                                self.hipsCompositeBox = new HiPSCompositeBox(aladin);

                            self.hipsCompositeBox._show({position: {
                                anchor: 'center center'
                            }});
                        },
                    },
                    ContextMenu.fileLoaderItem({
                        label: "FITS image file",
                        accept: ".fits",
                        action(file) {
                            let url = URL.createObjectURL(file);

                            const image = self.aladin.createImageFITS(
                                url,
                                {name: file.name},
                                (ra, dec, fov, _) => {
                                    // Center the view around the new fits object
                                    self.aladin.gotoRaDec(ra, dec);
                                    self.aladin.setFoV(fov * 1.1);

                                    URL.revokeObjectURL(url);
                                }
                            );

                            self.aladin.setOverlayImageLayer(
                                image,
                                Utils.uuidv4()
                            );
                        },
                    }),
                    ContextMenu.webkitDir({
                        label: "Load local HiPS",
                        action(files) {
                            // Give a different id at each loading.
                            let id = Utils.uuidv4();
                            let name = files[0].webkitRelativePath.split("/")[0];

                            let hips = self.aladin.createImageSurvey(
                                id,
                                name,
                                files,
                                null,
                                null,
                                {
                                    errorCallback: (e) => {
                                        aladin.addStatusBarMessage({
                                            duration: 2000,
                                            type: 'info',
                                            message: 'Could not add the local HiPS',
                                        })
                                    }
                                }
                            )
                            self.aladin.addNewImageLayer(hips);
                        },
                    }),
                ],
                tooltip: {
                    content: "Add a HiPS or an FITS image",
                    position: { direction: "top" },
                },
            },
            this.aladin
        );

        this.update({ content: this.createLayout() });
    }

    _addListeners() {
        let self = this;

        let updateOverlayList = () => {
            let wasHidden = self.isHidden;
            self._hide();

            // recompute the ui
            // If it is shown, update it
            // show will update the content of the stack
            self.update({ content: self.createLayout() });

            if (!wasHidden) self._show();

        };

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.LAYER_ADDED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.LAYER_SWAPPED.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.LAYER_REMOVED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.LAYER_CHANGED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                const hips = e.detail.layer;
                let ui = self.ui[hips.layer];

                if (!ui) {
                    return;
                }

                // change the ui from parameter changes
                // show button
                const opacity = hips.getOpacity();
                let showBtn = ui.showBtn;
                let hiddenBtn = showBtn.options.icon.url === hideIconUrl;
                
                if (opacity !== 0.0 && hiddenBtn) {
                    showBtn.update({
                        icon: { monochrome: true, url: showIconUrl },
                        tooltip: { content: "Hide" },
                    });
                } else if (opacity === 0.0 && !hiddenBtn) {
                    showBtn.update({
                        icon: { monochrome: true, url: hideIconUrl },
                        tooltip: { content: "Show" },
                    });
                }
            }
        );

        updateOverlayList();
    }

    _hide() {
        for (var key in this.ui) {
            let ui = this.ui[key];
            if (ui.settingsBtn && ui.settingsBtn.toggled) {
                // toggle off
                ui.settingsBtn.toggle();
            }
        }

        if (this.addOverlayBtn) this.addOverlayBtn.close();

        if (this.addHiPSBtn) this.addHiPSBtn.close();

        super._hide();
    }

    delete() {
        if (!this.ui) {
            return
        }

        for (let component of Object.values(this.ui)) {
            for (let elt of Object.values(component)) {
                elt.remove && elt.remove()
            }
        }
    }

    createLayout() {
        this.delete()
        this.ui = {};

        let layout = [[this.addOverlayBtn, "&nbsp;Overlays"]];

        layout = layout.concat(this._createOverlaysList());
        layout.push(
            [
                this.addHiPSBtn,
                "&nbsp;Surveys",
                this.filterEnabler,
                this.filterBtn,
            ],
        );
        layout = layout.concat(this._createSurveysList());
        return Layout.vertical(layout,
            {
                cssStyle: {
                    overflowWrap: "anywhere",
                    wordBreak: "break-word",
                }
            }
        );
    }

    _createOverlaysList() {
        let self = this;
        let aladin = self.aladin;

        let layout = [];
        const overlays = Array.from(this.aladin.getOverlays())
            .reverse()
            .map((overlay) => {
                return overlay;
            });
        // list of overlays
        for (const overlay of overlays) {
            const name = overlay.name;
            let showBtn = new ActionButton({
                size: "small",
                icon: {
                    url: overlay.isShowing ? showIconUrl : hideIconUrl,
                    monochrome: true,
                },
                tooltip: {
                    content: overlay.isShowing ? "Hide" : "Show",
                    position: { direction: "top" },
                },
                action(e, btn) {
                    if (overlay.isShowing) {
                        overlay.hide();
                        btn.update({
                            icon: { monochrome: true, url: hideIconUrl },
                            tooltip: { content: "Show" },
                        });
                    } else {
                        overlay.show();
                        btn.update({
                            icon: { monochrome: true, url: showIconUrl },
                            tooltip: { content: "Hide" },
                        });
                    }
                },
            });
            let optBtn = [
                showBtn,
            ];

            if (overlay.serialize) {
                optBtn.push(new ActionButton({
                    icon: {
                        url: downloadIconUrl,
                        monochrome: true,
                    },
                    size: "small",
                    tooltip: {
                        content: "Download JSON MOC",
                        position: { direction: "top" },
                    },
                    action(e) {
                        let json = overlay.serialize('json');
                        let blob = new Blob([json]);
                        Utils.download(URL.createObjectURL(blob), overlay.name + '.json');
                    },
                }));
            }

            if (overlay instanceof Catalog || overlay instanceof ProgressiveCat) {
                let catSettingsBox = new Box({
                    close: false,
                    content: new Form({
                        subInputs: [
                            {
                                label: 'Size',
                                tooltip: {content: 'Size of the sources', position: {direction: 'right'}},
                                name: 'size',
                                type: 'range',
                                min: 2.0,
                                max: 30.0,
                                value: overlay.sourceSize,
                                change: (e) => {
                                    const size = +e.target.value;
                                    overlay.setSourceSize(size)
                                }
                            },
                            {
                                label: 'Shape',
                                name: 'shape',
                                type: 'select',
                                options: [
                                    { value: "plus", label: "+" },
                                    { value: "rhomb", label: "◇" },
                                    { value: "triangle", label: "△" },
                                    { value: "cross", label: "✕" },
                                    { value: "square", label: "□" },
                                    { value: "circle", label: "○" },
                                ],
                                value: (overlay.shapeFn && "square") || overlay.shape,
                                change: (e) => {
                                    const shape = e.target.value
                                    overlay.setShape(shape)
                                }
                            },
                            {
                                label: 'Color',
                                name: 'color',
                                type: 'color',
                                value: overlay.color,
                                change: (e) => {
                                    let hex = e.target.value;
                                    overlay.setColor(hex)
                                }
                            },
                        ]
                    }),
                }, this.aladin.aladinDiv);
                catSettingsBox._hide()

                // catalog settings
                let catSettingsBtn = new WidgetTogglerButton({
                    icon: { url: settingsIconUrl, monochrome: true },
                    size: "small",
                    tooltip: {
                        content: "Settings",
                        position: { direction: "top" },
                    },
                    toggled: false,
                    enable: (_) => {
                        // toggle off the other settings if opened
                        for (var l in self.ui) {
                            let ui = self.ui[l]

                            if (l != name) {
                                if (ui.settingsBtn)
                                    ui.settingsBtn.close();
                            }
                        }

                        /*let spectraDisplayer = aladin.view.spectraDisplayer;
                        if (spectraDisplayer) {
                            spectraDisplayer.attachHiPS3D(options.layer)
                        } */
                    },
                    widget: catSettingsBox,
                    openDirection: "right"
                });

                optBtn.push(catSettingsBtn);

                if (!(name in self.ui)) {
                    self.ui[name] = {
                        settingsBox: catSettingsBox,
                        settingsBtn: catSettingsBtn,
                        showBtn,
                    };
                }
            }

            optBtn.push(ActionButton.BUTTONS(self.aladin).remove(
                (e) => {
                    self.aladin.removeLayer(overlay);
                }
            ));

            layout.push([
                this._addOverlayIcon(overlay),
                '<div class="aladin-overlay-label">' + name + "</div>",
                optBtn
            ]);
        }

        return layout;
    }

    _createSurveysList() {
        let self = this;

        let aladin = self.aladin;

        const layers = Array.from(aladin.getStackLayers())
            .reverse()
            .map((name) => {
                let overlay = aladin.getOverlayImageLayer(name);
                return overlay;
            });

        // survey list
        let layout = [];

        for (const hips of layers) {
            if (!hips) {
                continue;
            }

            let layerSelect = new LayerSelector({
                layer: hips,
                change(e) {
                    let name = e.target.value;

                    if (name === "More...") {
                        if (!aladin.hipsBrowser) {
                            aladin.hipsBrowser = new HiPSBrowserBox(aladin);
                        }
                        
                        let newLayer = hips.layer;
                        aladin.hipsBrowser._show({
                            selected: (hips) => {

                                self.aladin.setOverlayImageLayer(hips, newLayer);
                            },
                            position: { anchor: "center center" }
                        });
                        return;
                    }

                    let overlayLayer;
                    if (name in LayerSelector.cachedLayers) {
                        // it is an hips
                        let layerOptions = LayerSelector.cachedLayers[name];
                        //if (layerOptions.type === "hips") {
                        //    overlayLayer = A.HiPS(layerOptions.id || layerOptions.url, layerOptions);
                        //} else if (layerOptions.type === "image") {
                        //    overlayLayer = A.image(layerOptions.url, layerOptions);
                        //}
                        overlayLayer = layerOptions.id;
                    } else {
                        overlayLayer = hips
                    }
                    
                    aladin.setOverlayImageLayer(overlayLayer, hips.layer);
                }
            });

            let deleteBtn = ActionButton.createSmallSizedIconBtn({
                icon: { url: removeIconUrl, monochrome: true },
                tooltip: { content: "Remove", position: { direction: "top" } },
                action: (e) => {
                    aladin.removeImageLayer(hips.layer);
                    // remove HiPS cube player if any 
                    aladin.removeUIByName("cube_displayer" + hips.layer)

                    let spectraDisplayer = aladin.view.spectraDisplayer;
                    if (hips instanceof HiPS && spectraDisplayer && hips === spectraDisplayer.hips) {
                        spectraDisplayer._hide()
                    }
                },
            });

            let prevOpacity = null;
            let showBtn = ActionButton.createSmallSizedIconBtn({
                icon: {
                    url: hips.getOpacity() === 0.0 ? hideIconUrl : showIconUrl,
                    monochrome: true,
                },
                tooltip: {
                    content: hips.getOpacity() === 0.0 ? "Show" : "Hide",
                    position: { direction: "top" },
                },
                action(e, btn) {
                    e.preventDefault();
                    e.stopPropagation();

                    let opacity = hips.getOpacity();
                    if (opacity === 0.0) {
                        let newOpacity = prevOpacity || 1.0;
                        prevOpacity = null;
                        hips.setOpacity(newOpacity);
                        btn.update({
                            icon: { monochrome: true, url: showIconUrl },
                            tooltip: { content: "Hide" },
                        });
                    } else {
                        prevOpacity = opacity;
                        hips.setOpacity(0.0);
                        btn.update({
                            icon: { monochrome: true, url: hideIconUrl },
                            tooltip: { content: "Show" },
                        });
                    }
                },
            });

            let settingsBox = new HiPSSettingsBox(self.aladin);
            settingsBox._hide();

            let settingsBtn = new WidgetTogglerButton({
                icon: { url: settingsIconUrl, monochrome: true },
                size: "small",
                tooltip: {
                    content: "Settings",
                    position: { direction: "top" },
                },
                toggled: false,
                enable: (_) => {
                    // toggle off the other settings if opened
                    for (var l in self.ui) {
                        let ui = self.ui[l]

                        if (l != hips.layer) {
                            ui.settingsBtn.close();
                        }
                    }

                    settingsBox.update({ layer: hips });
                },
                widget: settingsBox,
                openDirection: "right",
            });

            let loadMOCBtn = ActionButton.BUTTONS(self.aladin)
                .addMOC({
                    name: hips.name,
                    url: hips.url + '/Moc.fits'
                });

            self.layer2swap = null;
            let swapBtn = new ActionButton({
                size: "small",
                icon: {
                    url: swapIcon,
                    size: "small",
                    monochrome: true,
                },
                tooltip: {
                    content: "Click on this button for both layers you want to swap",
                    position: { direction: "top" },
                },
                toggled: false,
                action: (_) => {
                    let toggled = swapBtn.options.toggled;
                    if (!toggled) {
                        if (!self.layer2swap) {
                            self.layer2swap = hips;
                        } else {
                            self.aladin.view.swapLayers(self.layer2swap.layer, hips.layer);
                        }
                    } else {
                        if (self.layer2swap) {
                            self.layer2swap = null;
                        }
                    }

                    swapBtn.update({
                        toggled: !toggled,
                    });
                },
            });

            let btns = [showBtn, settingsBtn];

            if (!(hips instanceof Image)) {
                btns.push(loadMOCBtn);
            }
            btns = btns.concat([swapBtn, deleteBtn]);

            let item = Layout.horizontal([layerSelect, Layout.horizontal(btns)]);
            layout.push(item);

            if (!(hips.layer in self.ui)) {
                self.ui[hips.layer] = {
                    //layerSelector: layerSelect,
                    settingsBox,
                    settingsBtn,
                    showBtn,
                };
            }
        }

        return layout;
    }

    _addOverlayIcon(overlay) {
        var tooltipText;
        var svg = "";
        if (overlay.type == "catalog" || overlay.type == "progressivecat") {
            var nbSources = overlay.getSources().length;
            tooltipText = nbSources + " source" + (nbSources > 1 ? "s" : "");

            svg = Icon.SVG_ICONS.CATALOG;
        } else if (overlay.type == "moc") {
            tooltipText =
                "Coverage: " +
                (100 * overlay.skyFraction()).toFixed(2) +
                " % of sky";

            svg = Icon.SVG_ICONS.MOC;
        } else if (overlay.type == "overlay") {
            svg = Icon.SVG_ICONS.OVERLAY;
        }

        let tooltip;
        if (tooltipText) {
            tooltip = {
                content: tooltipText,
                position: { direction: "bottom" },
            };
        }

        // retrieve SVG icon, and apply the layer color
        let color = overlay.color;
        if (overlay.colorFn) {
            color = "white"
        }

        return new Icon({
            size: "small",
            url: Icon.dataURLFromSVG({ svg, color }),
            tooltip,
        });
    }

    _show(options) {
        if (!this.aladin) {
            return;
        }

        this.position = (options && options.position) || this.position;

        if (!this.position) return;

        this.position.aladin = this.aladin;

        super._show({
            ...options,
            ...{ position: this.position },
        });
    }
}
