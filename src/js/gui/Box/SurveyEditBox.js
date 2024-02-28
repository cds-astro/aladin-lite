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

import { ColorCfg } from "../../ColorCfg.js";
 import { Box } from "../Widgets/Box.js";
 import { ALEvent } from "../../events/ALEvent.js";
 import opacityIconUrl from '../../../../assets/icons/opacity.svg';
 import luminosityIconUrl from '../../../../assets/icons/brightness.svg';
 import colorIconUrl from '../../../../assets/icons/color.svg';
 import pixelHistIconUrl from '../../../../assets/icons/pixel_histogram.svg';
 import { SelectorButton } from "../Widgets/Selector";

 import { Layout } from "../Layout.js";
 import { Input } from "../Widgets/Input.js";
import { CmapSelector } from "../Selector/Colormap.js";

 export class LayerEditBox extends Box {
     // Constructor
     constructor(aladin, options) {
        super(
            aladin,
            {
                cssStyle: {
                    padding: '4px',
                    backgroundColor: 'black',
                }
            }
        )

        this.aladin = aladin;

        let self = this;
        this.selector = new SelectorButton({
            luminosity: {
                icon: {
                    size: 'small',
                    monochrome: true,
                    url: luminosityIconUrl
                },
                tooltip: {content: 'Luminosity sliders', position: {direction: 'right'}},
                change(e) {
                    const content = Layout.horizontal({
                        layout: [self.selector, self.luminositySettingsContent]
                    });
                    self.update({content})
                }
            },
            opacity: {
                icon: {
                    size: 'small',
                    monochrome: true,
                    url: opacityIconUrl
                },
                tooltip: {content: 'Opacity slider', position: {direction: 'right'}},
                change(e) {
                    const content = Layout.horizontal({layout: [self.selector, self.opacitySettingsContent]});
                    self.update({content})
                }
            },
            colors: {
                icon: {
                    size: 'small',
                    url: colorIconUrl
                },
                tooltip: {content: 'Colormap', position: {direction: 'right'}},
                change(e) {
                    const content = Layout.horizontal({layout: [self.selector, self.colorSettingsContent]});
                    self.update({content})
                }
            },
            pixel: {
                icon: {
                    size: 'small',
                    monochrome: true,
                    url: pixelHistIconUrl
                },
                tooltip: {content: 'Pixel cutouts', position: {direction: 'right'}},
                change(e) {
                    const content = Layout.horizontal({layout: [self.selector, self.pixelSettingsContent]});
                    self.update({content})
                }
            },
            selected: 'opacity'
        }, aladin);



        // content 
        this.minCutInput = Input.number({
            cssStyle: {
                padding: '0',
                width: '8ex',
                'font-family': 'monospace',
            },
            tooltip: {content: 'Min cut', position: {direction: 'bottom'}},
            name: 'mincut',
            change(e) {
                let layer = self.options.layer;
                layer.setCuts(+e.target.value, layer.getColorCfg().getCuts()[1])
            }
        })

        this.maxCutInput = Input.number({
            cssStyle: {
                padding: '0',
                width: '8ex',
                'font-family': 'monospace',
            },
            tooltip: {content: 'Max cut', position: {direction: 'bottom'}},
            name: 'maxcut',
            change(e) {
                let layer = self.options.layer;
                layer.setCuts(layer.getColorCfg().getCuts()[0], +e.target.value)
            }
        })
        self.stretchSelector = Input.select({
            name: 'stretch',
            value: self.options.layer && self.options.layer.getColorCfg().stretch || 'linear',
            options: ['sqrt', 'linear', 'asinh', 'pow2', 'log'],
            change() {
                let layer = self.options.layer;
                layer.setColormap(layer.getColorCfg().getColormap(), {stretch: this.value});
            },
            tooltip: {content: 'stretch function', position: {direction: 'right'}}
        });

        self._addListeners()
    }

    update(options) {
        let self = this;
        if (options && options.layer) {
            let layer = options.layer;
            // Define the contents

            let layerOpacity = layer.getOpacity()

            self.opacitySettingsContent = Layout.horizontal([
                Input.slider({
                    tooltip: {content: layerOpacity, position: {direction: 'bottom'}},
                    name: 'opacitySlider',
                    type: 'range',
                    min: 0.0,
                    max: 1.0,
                    value: layerOpacity,
                    change(e, slider) {
                        const opacity = +e.target.value;
                        layer.setOpacity(opacity)
                        slider.update({value: opacity, tooltip: {content: opacity.toFixed(2), position: {direction: 'bottom'}}})
                    }
                }),
            ]);

            let brightness = layer.getColorCfg().getBrightness()
            let saturation = layer.getColorCfg().getSaturation()
            let contrast = layer.getColorCfg().getContrast()

            self.luminositySettingsContent = Layout.vertical({
                layout: [
                    Input.slider({
                        cssStyle: {
                            marginBottom: '7px',
                        },
                        tooltip: {content: 'brightness', position: {direction: 'right'}},
                        name: 'brightness',
                        type: 'range',
                        min: -1,
                        max: 1,
                        ticks: [0.0],
                        value: brightness,
                        change(e, slider) {
                            const brightness = +e.target.value
                            layer.setBrightness(brightness)
                        }
                    }),
                    Input.slider({
                        cssStyle: {
                            marginBottom: '7px',
                        },
                        tooltip: {content: 'saturation', position: {direction: 'right'}},
                        name: 'saturation',
                        type: 'range',
                        min: -1,
                        max: 1,
                        ticks: [0.0],
                        value: saturation,
                        change(e, slider) {
                            const saturation = +e.target.value
                            layer.setSaturation(saturation)
                        }
                    }),
                    Input.slider({
                        tooltip: {content: 'contrast', position: {direction: 'right'}},
                        name: 'contrast',
                        type: 'range',
                        min: -1,
                        max: 1,
                        ticks: [0.0],
                        value: contrast,
                        change(e, slider) {
                            const contrast = +e.target.value
                            layer.setContrast(contrast)
                        }
                    }),
                ]
            });
            const [minCut, maxCut] = layer.getColorCfg().getCuts();
            self.minCutInput.set(minCut);
            self.maxCutInput.set(maxCut)
            self.stretchSelector.update({value: layer.getColorCfg().stretch})

            self.pixelSettingsContent = Layout.horizontal({
                layout: [
                    self.stretchSelector,
                    self.minCutInput,
                    self.maxCutInput
                ]
            });

            let cmap = layer.getColorCfg().getColormap();

            this.colorSettingsContent = Input.select({
                name: 'colormap',
                value: cmap,
                options: ColorCfg.COLORMAPS,
                change() {
                    let colormap = this.value;
                    layer.setColormap(colormap)
                },
            });

            //this.colorSettingsContent = new CmapSelector(optionsCmapSelector, this.aladin);
            let content = (() => {
                let selected = self.selector.options.selected;
                switch (selected) {
                    case 'colors':
                        return self.colorSettingsContent;
                    case 'pixel':
                        return self.pixelSettingsContent;
                    case 'opacity':
                        return self.opacitySettingsContent;
                    case 'luminosity':
                        return self.luminositySettingsContent;
                    default:
                        return self.opacitySettingsContent;
                }
            })();
            options.content = Layout.horizontal({layout: [self.selector, content]});
        }

        super.update(options)
    }

    _show(options) {
        this._hide();

        if (this.selector) {
            this.selector._show();
        }

        if (this.stretchSelector) {
            this.stretchSelector._show();
        }

        if (this.colorSettingsContent) {
            this.colorSettingsContent._show();
        }

        super._show(options)
    }

    _hide() {
        if (this.colorSettingsContent) {
            this.colorSettingsContent._hide();
        }

        if (this.stretchSelector) {
            this.stretchSelector._hide();
        }

        if (this.selector) {
            this.selector._hide();
        }

        super._hide()
    }

    _addListeners() {
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, (e) => {
            const layerChanged = e.detail.layer;
            let selectedLayer = this.options.layer;
            if (selectedLayer && layerChanged.layer === selectedLayer.layer) {
                let colorCfg = layerChanged.getColorCfg();

                let cmap = colorCfg.getColormap();
                let reversed = colorCfg.getReversed();
                let stretch = colorCfg.stretch;

                let [minCut, maxCut] = colorCfg.getCuts();
                this.minCutInput.set(+minCut.toFixed(2));
                this.maxCutInput.set(+maxCut.toFixed(2));
                this.stretchSelector.update({value: stretch})
            }
        });
    }
 
    static singleton;
 
    static getInstance(aladin) {
        if (!LayerEditBox.singleton) {
            LayerEditBox.singleton = new LayerEditBox(aladin);
        }

        return LayerEditBox.singleton;
    }
}
 