import { ImageLayer } from "../../ImageLayer.js";
import searchIconImg from '../../../../assets/icons/search.svg';

import { ContextMenu } from "../Widgets/ContextMenu";
import { HiPSSelectorBox } from "../Box/HiPSSelectorBox";

export class SurveyCtxMenu extends ContextMenu {
    // Constructor
    constructor(aladin) {
        super(aladin)

        let layout = [{
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
                if (!self.hipsBox) {
                    self.hipsBox = new HiPSSelectorBox({
                            layer: 'base',
                            position: self.position,
                        },
                        aladin
                    );
                }
                
                self.hipsBox._show()
            }
        }];

        let layers = ImageLayer.LAYERS.sort(function (a, b) {
            if (!a.order) {
                return a.name > b.name ? 1 : -1;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });

        for(let layer of layers) {
            let backgroundUrl = SurveyCtxMenu.previewImagesUrl[layer.name];
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

            layout.push({
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

                    if (self.hipsBox) {
                        self.hipsBox._hide();
                    }
                }
            })
        }

        let self = this;
        self.hipsSelector = null;
        this.attach(layout)
    }

    _show(options) {
        // set the position when we want to show
        if (options && options.position) {
            this.position = options.position;
        }

        console.log(this.aladin.aladinDiv.offsetHeight)
        let maxHeight = Math.min(this.aladin.aladinDiv.offsetHeight, 500);
        super.show({
            position: this.position,
            cssStyle: {
                width: '20em',
                overflowY: 'scroll',
                maxHeight: maxHeight + 'px',
                color: 'white',
                backgroundColor: 'black',
            }
        })
    }

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
}