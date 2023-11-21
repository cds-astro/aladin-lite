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
 * File gui/Stack.js
 *
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { ImageLayer } from "../ImageLayer.js";
import { ALEvent } from "../events/ALEvent.js";
import { HiPSSelectorBox } from "./HiPSSelectorBox.js";

import { ActionButton } from "./Widgets/ActionButton.js";
import { ContextMenu } from "./Widgets/ContextMenu.js";
import { Box } from "./Widgets/Box.js";
import { Form } from "./Widgets/Form.js";
import { Layout } from "./Layout.js";
import selectIconImg from '../../../assets/icons/select.svg';
import searchIconImg from '../../../assets/icons/search.svg';
import { DOMElement } from "./Widgets/Widget.js";

export class HiPSLayer extends DOMElement {

    // Constructor
    constructor(aladin, layer, parent) {
        //aladin = aladin;
        //layer = layer;
        let hipsSelector;

        let boxes = [];

        let hideBtn = ActionButton.createIconBtn({
            content: "👁️",
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                color: 'black',
            },
            tooltip: {content: 'Hide the layer', position: {direction: 'right'}}
        });
        
        let layoutImageSelection = {
            label: Layout.horizontal({
                layout: [
                    ActionButton.createIconBtn({
                        iconURL: selectIconImg,
                        tooltip: {content: 'Select an image layer among those already defined', position: { direction: 'bottom' }},
                        cssStyle: {
                            backgroundPosition: 'center center',
                            backgroundColor: '#bababa',
                            border: '1px solid rgb(72, 72, 72)',
                            cursor: 'help',
                        },
                    }),
                    'Choose a survey'
                ]
            }),
            subMenu: []
        };

        layoutImageSelection.subMenu.push({
            label: Layout.horizontal({layout: [
                ActionButton.createIconBtn({
                    iconURL: searchIconImg,
                    tooltip: {content: 'Search for a new survey in our database...', position: {direction: 'bottom'}},
                    cssStyle: {
                        backgroundPosition: 'center center',
                        backgroundColor: '#bababa',
                        border: '1px solid rgb(72, 72, 72)',
                        cursor: 'help',
                    },
                }),
                'Search for a new survey'
            ]}),
            action(o) {
                if (!hipsSelector) {
                    hipsSelector = new HiPSSelectorBox(aladin, layer);
                }
            }
        })

        let action = (e) => {
            let layerName = e.srcElement.innerText;
            let cfg = ImageLayer.LAYERS.find((layer) => layer.name === layerName);
            let newLayer;
            
            // Max order is specific for surveys
            if (cfg.subtype === "fits") {
                // FITS
                newLayer = aladin.createImageFITS(
                    cfg.url,
                    cfg.name,
                    cfg.options,
                );
            } else {
                // HiPS
                newLayer = aladin.createImageSurvey(
                    cfg.id,
                    cfg.name,
                    cfg.url,
                    undefined,
                    cfg.maxOrder,
                    cfg.options
                );
            }

            aladin.setOverlayImageLayer(newLayer, layer.layer);
        }

        // Sort the layers by name order
        let layers = ImageLayer.LAYERS.sort(function (a, b) {
            if (!a.order) {
                return a.name > b.name ? 1 : -1;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });

        for(let layer of layers) {
            layoutImageSelection.subMenu.push({label: layer.name, action})
        }

        let updateContextMenu = () => {
            ContextMenu.getInstance(aladin).attach([
                layoutImageSelection,
                {
                    label: "Edit",
                    subMenu: [
                        {
                            label: 'Color panel',
                            action(o) {            
                                let box = HiPSLayer.createColorBox(aladin, layer, parent);
                                boxes.push(box);
                            }
                        },
                        {
                            label: 'Tile format',
                            subMenu: [
                                {
                                    label: 'webp',
                                    disabled: !layer.getAvailableFormats().includes('webp'),
                                    selected: layer.imgFormat === 'webp',
                                    action(o) {
                                        layer.setImageFormat('webp');
                                    } 
                                },
                                {
                                    label: 'jpeg',
                                    disabled: !layer.getAvailableFormats().includes('jpeg'),
                                    selected: layer.imgFormat === 'jpeg',
                                    action(o) {
                                        layer.setImageFormat('jpeg');
                                    } 
                                },
                                {
                                    label: 'png',
                                    disabled: !layer.getAvailableFormats().includes('png'),
                                    selected: layer.imgFormat === 'png',
                                    action(o) {
                                        layer.setImageFormat('png');
                                    } 
                                },
                                {
                                    label: 'fits',
                                    disabled: !layer.getAvailableFormats().includes('fits'),
                                    selected: layer.imgFormat === 'fits',
                                    action(o) {
                                        layer.setImageFormat('fits');
                                    }
                                }
                            ]
                        },
                        {
                            label: 'Stretch',
                            subMenu: [
                                {
                                    label: 'pow',
                                    selected: layer.getColorCfg().stretch === 'pow2',
                                    action(o) {
                                        layer.setColormap(layer.getColorCfg().getColormap(), { stretch: 'pow2' });
                                    } 
                                },
                                {
                                    label: 'linear',
                                    selected: layer.getColorCfg().stretch === 'linear',
                                    action(o) {
                                        layer.setColormap(layer.getColorCfg().getColormap(), { stretch: 'linear' });
                                    } 
                                },
                                {
                                    label: 'sqrt',
                                    selected: layer.getColorCfg().stretch === 'sqrt',
                                    action(o) {
                                        layer.setColormap(layer.getColorCfg().getColormap(), { stretch: 'sqrt' });
                                        console.log(layer.getColorCfg().stretch === 'sqrt')
                                    } 
                                },
                                {
                                    label: 'asinh',
                                    selected: layer.getColorCfg().stretch === 'asinh',
                                    action(o) {
                                        layer.setColormap(layer.getColorCfg().getColormap(), { stretch: 'asinh' });
                                    } 
                                },
                                {
                                    label: 'log',
                                    selected: layer.getColorCfg().stretch === 'log',
                                    action(o) {
                                        layer.setColormap(layer.getColorCfg().getColormap(), { stretch: 'log' });
                                    } 
                                }
                            ]
                        }
                    ]
                },
                {
                    label: Layout.horizontal({layout: [hideBtn, (() => { if(layer.getOpacity() === 0.0) {return 'Show'} else {return 'Hide'}})()]}),
                    action(o) {
                        let opacity = layer.getOpacity();
                        if (opacity === 0.0) {
                            layer.setOpacity(1.0);
                        } else {
                            layer.setOpacity(0.0);
                        }
                    }
                },
                {
                    label: Layout.horizontal({layout: [
                        ActionButton.createIconBtn({
                            content: "❌",
                            cssStyle: {
                                backgroundColor: '#bababa',
                                borderColor: '#484848',
                                color: 'black',
                            },
                        }),
                        "Delete layer"
                    ]}),
                    disabled: layer.layer === aladin.getBaseImageLayer().layer,
                    action(o) {
                        aladin.removeImageLayer(layer.layer);
                    }
                },
            ]);
        };

        let settingsBtn = ActionButton.createIconBtn({
            content: "☰",
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
                color: 'black',
                fontSize: '1.5em',
                textAlign: 'center',
                lineHeight: '0px',
            },
            tooltip: {content: 'Layer settings', position: {direction: 'bottom'} },
            action(e) {
                ContextMenu.getInstance(aladin)._hide()
                updateContextMenu();
                ContextMenu.getInstance(aladin).show(e, {
                    position: {
                        anchor: settingsBtn.element(),
                        direction: 'bottom'
                    }
                })
            }
        });

        // HiPS header div
        let el = Layout.horizontal({layout: [settingsBtn, '<div style="max-width: 15em;word-wrap: break-word;">' + layer.name + '</div>']}).element();
        el.classList.add("aladin-stack-item");
        super(el)

        this.boxes = boxes;
        // Add a centered on button for images
        /*if (this.layer.subtype === "fits") {
            let layerSelector = this.headerDiv[0].querySelector(".aladin-layerSelection");
            ActionButton.createIconBtn({
                content: "🎯",
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'black',
                },
                info: 'Focus on the FITS',
                action(e) {
                    self.layer.focusOn();
                }
            }, layerSelector, 'afterend');
        }*/


        // HiPS main options div
        /*let cmListStr = '';
        for (const cm of this.aladin.wasm.getAvailableColormapList()) {
            cmListStr += '<option>' + cm + '</option>';
        }

        this.cmap = "native";
        this.color = "#ff0000";

        this.mainDiv = $('<div class="aladin-form-input-group" style="display:none; padding: 0px 4px">' +
            // colormap
            '  <div class="aladin-form-input"><label>Colormap</label><select class="aladin-input colormap-selector">' + cmListStr + '</select></div>' +
            '  <div class="aladin-form-input"><label>Reverse</label><input type="checkbox" class="reversed aladin-input" /></div>' +
            '  <div class="aladin-form-input"><label>Stretch</label><select class="aladin-input stretch"><option>pow2</option><option selected>linear</option><option>sqrt</option><option>asinh</option><option>log</option></select></div>' +
            '  <div class="aladin-form-input"><label>Format</label><select class="aladin-input format"></select></div>' +
            '  <div class="aladin-form-input"><label>Min cut</label><input type="number" class="aladin-input min-cut"></div>' +
            '  <div class="aladin-form-input"><label>Max cut</label><input type="number" class="aladin-input max-cut"></div>' +
            // tonal corrections
            '  <div class="aladin-form-input"><label>Gamma</label><input class="aladin-input gamma" type="number" value="1.0" min="0.1" max="10.0" step="0.01"></div>' +
            '  <div class="aladin-form-input"><label>Color Sat.</label><input class="aladin-input saturation" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div>' +
            '  <div class="aladin-form-input"><label>Contrast</label><input class="aladin-input contrast" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div>' +
            '  <div class="aladin-form-input"><label>Brightness</label><input class="aladin-input brightness" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div>' +
            // blending mode
            '  <div class="aladin-form-input"><label>Blending mode</label><select class="aladin-input blending"><option>additive</option><option selected>default</option></select></div>' +
            // opacity
            '  <div class="aladin-form-input"><label>Opacity</label><input class="aladin-input opacity" type="range" min="0" max="1" step="0.01"></div>' +
        '</div>');

        this._addListeners();
        this._updateHiPSLayerOptions();

        this.layerChangedListener = function(e) {
            const layer = e.detail.layer;
            if (layer.layer === self.layer.layer) {
                // Update the survey to the new one
                self.layer = layer;
                self._updateHiPSLayerOptions();
            }
            self._updateLayersDropdownList();
        };
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, this.layerChangedListener);*/
    }

    remove() {
        // remove the floating boxes first
        for (let box of this.boxes) {
            box.remove();
        }
        super.remove()
    }

    static createColorBox(aladin, layer, parent) {
        let box;
        let contentBox;
        let settingsSelector = new Form({
            label: "Settings",
            name: 'param',
            type: 'select',
            value: 'colormap',
            options: ['colormap', 'pixel cutouts', 'post-fx', 'blending'],
            change(e) {
                let val = e.target.value;

                let content;
                switch (val) {
                    case 'colormap':
                        content = colorSettings;
                        break;
                    case 'pixel cutouts':
                        content = pixelCutsSettings;
                        break;
                    case 'post-fx':
                        content = postFXSettings;
                        break;
                    case 'blending':
                        content = blendingSettings;
                        break;
                }
                box.update({
                    title: "Color settings",
                    content: Layout.vertical({
                        layout: ['For ' + layer.name, settingsSelector, content]
                    })
                })
            }
        })

        let cssForm = {
            border: '2px solid #d2d2d2',
            margin: '0px',
            padding: '4px'
        };
        // Color Settings form
        let colorSettings = new Form({
            cssStyle: cssForm,
            subInputs: [
                {
                    label: "Colormap",
                    name: 'cmap',
                    type: "select",
                    value: layer.getColorCfg().getColormap(),
                    options: aladin.wasm.getAvailableColormapList(),
                    change(e) {
                        let cmap = e.target.value;
                        layer.setColormap(cmap);
                    }
                },
                {
                    label: "Reverse",
                    name: 'reversed',
                    type: "checkbox",
                    checked: layer.getColorCfg().getReversed(),
                    change(e) {
                        let reversed = e.target.checked;
                        layer.setColormap(layer.getColorCfg().getColormap(), { reversed: reversed });
                    }
                }
            ]
        });

        ALEvent.HIPS_LAYER_CHANGED.listenedBy(aladin.aladinDiv, (e) => {
            const layer = e.detail.layer;
            if (layer.layer === layer.layer) {
                let colorCfg = layer.getColorCfg();

                let cmap = colorCfg.getColormap();
                let reversed = colorCfg.getReversed();

                colorSettings.set('cmap', cmap);
                colorSettings.set('reversed', reversed);
            }
        });

        // Pixel cutouts form
        let pixelCutsSettings = new Form({
            cssStyle: cssForm,
            subInputs: [
                {
                    label: "Mincut",
                    name: "mincut",
                    type: "number",
                    value: layer.getColorCfg().getCuts()[0],
                    change(e) {
                        let lowCut = +e.target.value;
                        layer.setCuts(lowCut, layer.getColorCfg().getCuts()[1]);
                    }
                },
                {
                    label: "Maxcut",
                    name: "maxcut",
                    type: "number",
                    value: layer.getColorCfg().getCuts()[1],
                    change(e) {
                        let highCut = +e.target.value;
                        layer.setCuts(layer.getColorCfg().getCuts()[0], highCut);
                    }
                }
            ]
        });

        ALEvent.HIPS_LAYER_CHANGED.listenedBy(aladin.aladinDiv, (e) => {
            const layer = e.detail.layer;
            if (layer.layer === layer.layer) {
                let [minCut, maxCut] = layer.getColorCfg().getCuts();
                pixelCutsSettings.set('mincut', minCut);
                pixelCutsSettings.set('maxcut', maxCut);
            }
        });

        // Post-fx settings
        let postFXSettings = new Form({
            cssStyle: cssForm,
            subInputs: [
                {
                    label: "Brightness",
                    name: "brightness",
                    type: "number",
                    value: parseFloat(layer.getColorCfg().getBrightness()),
                    change(e) {
                        let brightness = +e.target.value;
                        layer.setBrightness(brightness);
                    }
                },
                {
                    label: "Contrast",
                    name: "contrast",
                    type: "number",
                    value: layer.getColorCfg().getContrast(),
                    change(e) {
                        let contrast = +e.target.value;
                        layer.setContrast(contrast);
                    }
                },
                {
                    label: "Saturation",
                    name: "saturation",
                    type: "number",
                    value: layer.getColorCfg().kSaturation,
                    change(e) {
                        let sat = +e.target.value;
                        layer.setSaturation(sat);
                    }
                },
                {
                    label: "γ",
                    name: "Gamma",
                    type: "number",
                    value: layer.getColorCfg().getGamma(),
                    change(e) {
                        let gamma = +e.target.value;
                        layer.setGamma(gamma);
                    }
                }
            ]
        });

        box = new Box({
            title: "Color settings",
            content: Layout.vertical({layout: ['For ' + layer.name, settingsSelector, colorSettings]}),
            cssStyle: {
                width: 'object-contain',
            },
            position: {
                direction: "right",
                anchor: parent,
            },
            draggable: true,
        }, aladin.aladinDiv);

        return box;
    }
}