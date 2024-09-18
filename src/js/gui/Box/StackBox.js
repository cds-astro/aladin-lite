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
import showIconUrl from "../../../../assets/icons/show.svg";
import addIconUrl from "../../../../assets/icons/plus.svg";
import hideIconUrl from "../../../../assets/icons/hide.svg";
import removeIconUrl from "../../../../assets/icons/remove.svg";
import settingsIconUrl from "../../../../assets/icons/settings.svg";
import searchIconImg from "../../../../assets/icons/search.svg";
import downloadIconUrl from '../../../../assets/icons/download.svg';


import { TogglerActionButton } from "../Button/Toggler.js";
import { Icon } from "../Widgets/Icon.js";
import { Box } from "../Widgets/Box.js";
import { CtxMenuActionButtonOpener } from "../Button/CtxMenuOpener.js";
import { Input } from "../Widgets/Input.js";
import { Image } from "../../Image.js";
import { HiPSBrowserBox } from "./HiPSBrowserBox.js";

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
            url: "https://axel.u-strasbg.fr/HiPSCatService/SIMBAD",
            options: {
                id: "simbad",
                name: "SIMBAD",
                shape: "circle",
                sourceSize: 8,
                color: "#318d80",
                hoverColor: 'red',
                onClick: "showTable",
                shape: (s) => {
                    let galaxy = ["Seyfert","Seyfert_1", "Seyfert_2","LSB_G","PartofG","RadioG","Gin","GinPair","HII_G","LensedG","BClG","BlueCompG","EmG","GinCl","GinGroup","StarburstG","LINER","AGN","Galaxy"].some((n) => s.data.main_type.indexOf(n) >= 0);
                    if (!galaxy) return;

                    let a = +s.data.size_maj;
                    let b = +s.data.size_min;

                    let theta = +s.data.size_angle || 0.0;
                    return A.ellipse(s.ra, s.dec, a / 60, b / 60, theta, { color: "cyan" });
                }
            },
        },
        gaia: {
            url: "https://axel.u-strasbg.fr/HiPSCatService/I/355/gaiadr3",
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
            url: "https://axel.u-strasbg.fr/HiPSCatService/II/246/out",
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
    constructor(aladin, stackBtn) {
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
        this.stackBtn = stackBtn;
        this.cachedHiPS = {};

        this.aladin = aladin;

        this.mode = "stack";

        this._addListeners();

        this.mocHiPSUrls = {};

        this.HiPSui = {};
        let self = this;
        // Add overlay button
        this.addOverlayBtn = new CtxMenuActionButtonOpener(
            {
                icon: {
                    url: addIconUrl,
                    size: "small",
                    monochrome: true,
                },
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

                                    //self._hide();

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

                                    //self._hide();

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
                                        lineWidth: 3.0,
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
                                                                "j2000"
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
                                                                lineWidth: 3.0,
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
                                                            "j2000"
                                                        );
                                                    let [ra2, dec2] =
                                                        self.aladin.pix2world(
                                                            r.x + r.w,
                                                            r.y,
                                                            "j2000"
                                                        );
                                                    let [ra3, dec3] =
                                                        self.aladin.pix2world(
                                                            r.x + r.w,
                                                            r.y + r.h,
                                                            "j2000"
                                                        );
                                                    let [ra4, dec4] =
                                                        self.aladin.pix2world(
                                                            r.x,
                                                            r.y + r.h,
                                                            "j2000"
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
                                                            lineWidth: 3.0,
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
                                                                "j2000"
                                                            );
                                                        ra.push(lon);
                                                        dec.push(lat);
                                                    }

                                                    let moc = A.MOCFromPolygon(
                                                        { ra, dec },
                                                        {
                                                            name: "poly",
                                                            lineWidth: 3.0,
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
                            content: "Add new survey",
                        },
                        action: (e) => {
                            e.stopPropagation();
                            e.preventDefault();

                            /*self._hide();

                            self.hipsSelectorBox = new HiPSSelectorBox(self.aladin);
                            // attach a callback
                            self.hipsSelectorBox.attach( 
                                (HiPSId) => {
                                    let name = Utils.uuidv4()
                                    self.aladin.setOverlayImageLayer(HiPSId, name)

                                    self.show();
                                }
                            );

                            self.hipsSelectorBox._show({
                                position: self.position,
                            });*/
                            self.aladin.addNewImageLayer(
                                A.imageHiPS('P/DSS2/color', {
                                    errorCallback: (e) => {
                                        aladin.addStatusBarMessage({
                                            duration: 2000,
                                            type: 'info',
                                            message: 'DSS2 colored HiPS could not plot',
                                        })
                                    }
                                })
                            );
                        },
                    },
                    {
                        label: {
                            icon: {
                                url: hipsIconUrl,
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

                            if (!self.hipsBrowser)
                                self.hipsBrowser = new HiPSBrowserBox(aladin);

                            self.hipsBrowser._show({position: {
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
                            let id = files[0].webkitRelativePath.split("/")[0];
                            let name = id;

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

        ALEvent.GRAPHIC_OVERLAY_LAYER_CHANGED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.HIPS_LAYER_ADDED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.HIPS_LAYER_SWAP.listenedBy(this.aladin.aladinDiv, function (e) {
            updateOverlayList();
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                updateOverlayList();
            }
        );

        ALEvent.HIPS_LAYER_CHANGED.listenedBy(
            this.aladin.aladinDiv,
            function (e) {
                const hips = e.detail.layer;
                let ui = self.HiPSui[hips.layer];

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
        let hipsCache = this.aladin.hipsCache;

        // Add a listener for HiPS list changes
        ALEvent.HIPS_CACHE_UPDATED.listenedBy(document.body, () => {
            self.cachedHiPS = {};

            for (var key in hipsCache.cache) {
                let HiPS = hipsCache.cache[key];

                if (HiPS.name) {
                    self.cachedHiPS[HiPS.name.toString()] = HiPS;
                }
            }

            // Update the options of the selector
            const favorites = Object.keys(self.cachedHiPS);

            // one must add the current HiPS too!
            favorites.sort();

            for (var key in self.HiPSui) {
                let hips = self.HiPSui[key];
                let currentHiPS = hips.HiPSSelector.options.value

                let favoritesCopy = [...favorites];
                if (!(currentHiPS in favoritesCopy)) {
                    favoritesCopy.push(currentHiPS)
                }

                hips.HiPSSelector.update({value: currentHiPS, options: favoritesCopy});
            }
        });
    }

    _hide() {
        for (var key in this.HiPSui) {
            let hips = this.HiPSui[key];
            if (hips.settingsBtn.toggled) {
                // toggle off
                hips.settingsBtn.toggle();
            }
        }

        /*if (this.hipsBrowser) {
            this.hipsBrowser._hide();
        }*/

        /*if (this.catBox) {
            this.catBox._hide();
        }*/

        if (this.addOverlayBtn) this.addOverlayBtn.hideMenu();

        if (this.addHiPSBtn) this.addHiPSBtn.hideMenu();

        // toggle the button because the window is closed
        this.stackBtn.update({toggled: false});

        super._hide();
    }

    createLayout() {
        this.HiPSui = {};

        let layout = [Layout.horizontal([this.addOverlayBtn, "Overlays"])];

        layout = layout.concat(this._createOverlaysList());

        layout.push(
            Layout.horizontal({
                layout: [
                    this.addHiPSBtn,
                    "Surveys",
                    this.filterEnabler,
                    this.filterBtn,
                ],
            })
        );
        layout = layout.concat(this._createSurveysList());

        return Layout.vertical({ layout });
    }

    _createOverlaysList() {
        let self = this;

        let layout = [];
        const overlays = Array.from(this.aladin.getOverlays())
            .reverse()
            .map((overlay) => {
                return overlay;
            });
        // list of overlays
        for (const overlay of overlays) {
            const name = overlay.name;
            let optBtn = [];
            optBtn.push(new ActionButton({
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
            }));

            optBtn.push(new ActionButton({
                icon: {
                    url: removeIconUrl,
                    monochrome: true,
                },
                size: "small",
                /*cssStyle: {
                    visibility: Utils.hasTouchScreen() ? 'visible' : 'hidden',
                },*/
                tooltip: {
                    content: "Remove",
                    position: { direction: "top" },
                },
                action(e) {
                    self.aladin.removeLayer(overlay);
                },
            }));

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
            

            let item = Layout.horizontal({
                layout: [
                    this._addOverlayIcon(overlay),
                    '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;">' +
                        name +
                        "</div>",
                    Layout.horizontal({ layout: optBtn }),
                ],
                cssStyle: {
                    textAlign: "center",
                    display: "flex",
                    alignItems: "center",
                    listStyle: "none",
                    justifyContent: "space-between",
                    width: "100%",
                },
            });

            /*if(!Utils.hasTouchScreen()) {
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
            }*/
            layout.push(item);
        }

        return layout;
    }

    _createSurveysList() {
        let self = this;

        const layers = Array.from(self.aladin.getStackLayers())
            .reverse()
            .map((name) => {
                let overlay = self.aladin.getOverlayImageLayer(name);
                return overlay;
            });

        // survey list
        let layout = [];

        let hipsOptions = Object.keys(self.cachedHiPS);
        hipsOptions.sort()

        for (const layer of layers) {

            let HiPSSelector = Input.select({
                value: layer.name,
                options: hipsOptions,
                title: layer.name,
                change: (e) => {
                    let name = e.target.value;
                    // search for the 
                    let HiPS = self.cachedHiPS[name];

                    let image;
                    if (HiPS instanceof Image) {
                        image = HiPS;
                    } else {
                        // HiPS
                        image = HiPS.id || HiPS.url || undefined;
                    }

                    self.aladin.setOverlayImageLayer(image, layer.layer);
                }
            });

            let deleteBtn = ActionButton.createSmallSizedIconBtn({
                icon: { url: removeIconUrl, monochrome: true },

                disable: layer.layer === "base",
                tooltip: { content: "Remove", position: { direction: "top" } },
                action(e) {
                    self.aladin.removeImageLayer(layer.layer);
                },
            });

            let showBtn = ActionButton.createSmallSizedIconBtn({
                icon: {
                    url: layer.getOpacity() === 0.0 ? hideIconUrl : showIconUrl,
                    monochrome: true,
                },
                tooltip: {
                    content: layer.getOpacity() === 0.0 ? "Show" : "Hide",
                    position: { direction: "top" },
                },
                action(e, btn) {
                    e.preventDefault();
                    e.stopPropagation();

                    let opacity = layer.getOpacity();
                    if (opacity === 0.0) {
                        layer.setOpacity(1.0);
                        btn.update({
                            icon: { monochrome: true, url: showIconUrl },
                            tooltip: { content: "Hide" },
                        });
                    } else {
                        layer.setOpacity(0.0);
                        btn.update({
                            icon: { monochrome: true, url: hideIconUrl },
                            tooltip: { content: "Show" },
                        });
                    }
                },
            });

            let settingsBox = new HiPSSettingsBox(self.aladin);

            settingsBox.update({ layer });
            settingsBox._hide();


            let settingsBtn = new TogglerActionButton({
                icon: { url: settingsIconUrl, monochrome: true },
                size: "small",
                tooltip: {
                    content: "Settings",
                    position: { direction: "top" },
                },
                toggled: false,
                actionOn: (e) => {
                    // toggle off the other settings if opened
                    for (var l in self.HiPSui) {
                        let ui = self.HiPSui[l]

                        if (l != layer.layer) {
                            ui.settingsBtn.close();
                        }
                    }

                    settingsBox._show({
                        position: {
                            nextTo: settingsBtn,
                            direction: "right",
                            aladin: self.aladin,
                        },
                    });
                },
                actionOff: (e) => {
                    settingsBox._hide();
                },
            });

            let loadMOCBtn = new ActionButton({
                size: "small",

                icon: {
                    url: Icon.dataURLFromSVG({ svg: Icon.SVG_ICONS.MOC }),
                    size: "small",
                    monochrome: true,
                },
                tooltip: {
                    content: "Add coverage",
                    position: { direction: "top" },
                },
                toggled: (() => {
                    let overlays = self.aladin.getOverlays();
                    let found = overlays.find(
                        (o) => o.type === "moc" && o.name === layer.name
                    );
                    return found !== undefined;
                })(),
                action: (e) => {
                    if (!loadMOCBtn.options.toggled) {
                        // load the moc
                        let moc = A.MOCFromURL(
                            layer.url + "/Moc.fits",
                            { name: layer.name },
                            () => {
                                self.mocHiPSUrls[layer.url] = moc;

                                if (self.aladin.statusBar) {
                                    self.aladin.statusBar.appendMessage({
                                        message:
                                            "Coverage of " +
                                            layer.name +
                                            " loaded",
                                        duration: 2000,
                                        type: "info",
                                    });
                                }

                                loadMOCBtn.update({
                                    toggled: true,
                                    tooltip: {
                                        content: "Remove coverage",
                                        position: { direction: "top" },
                                    },
                                });
                            }
                        );
                        self.aladin.addMOC(moc);
                    } else {
                        // unload the moc
                        let moc = self.mocHiPSUrls[layer.url];
                        self.aladin.removeLayer(moc);

                        delete self.mocHiPSUrls[layer.url];

                        if (self.aladin.statusBar) {
                            self.aladin.statusBar.appendMessage({
                                message:
                                    "Coverage of " + layer.name + " removed",
                                duration: 2000,
                                type: "info",
                            });
                        }

                        loadMOCBtn.update({
                            toggled: false,
                            tooltip: {
                                content: "Add coverage",
                                position: { direction: "top" },
                            },
                        });
                    }
                },
            });

            let btns = [showBtn, settingsBtn];

            if (!(layer instanceof Image)) {
                btns.push(loadMOCBtn);
            }
            btns.push(deleteBtn);

            let item = Layout.horizontal({
                layout: [HiPSSelector, Layout.horizontal(btns)],
            });

            layout.push(item);

            if (!(layer.layer in self.HiPSui)) {
                self.HiPSui[layer.layer] = {
                    HiPSSelector,
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
        return new Icon({
            size: "small",
            url: Icon.dataURLFromSVG({ svg, color: overlay.color }),
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

        this.stackBtn.update({toggled: true});
    }
}
