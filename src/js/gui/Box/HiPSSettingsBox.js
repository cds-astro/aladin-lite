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
import { Form } from "../Widgets/Form.js";
import { ColorCfg } from "../../ColorCfg.js";
 import { Box } from "../Widgets/Box.js";
 import { ALEvent } from "../../events/ALEvent.js";
 import opacityIconUrl from '../../../../assets/icons/opacity.svg';
 import luminosityIconUrl from '../../../../assets/icons/brightness.svg';
 import colorIconUrl from '../../../../assets/icons/color.svg';
 import pixelHistIconUrl from '../../../../assets/icons/pixel_histogram.svg';
 import { RadioButton } from "../Widgets/Radio.js";

 import { Layout } from "../Layout.js";

 export class HiPSSettingsBox extends Box {
     // Constructor
     constructor(aladin, options) {
        let self;
        let selector = new RadioButton({
            luminosity: {
                icon: {
                    size: 'small',
                    monochrome: true,
                    url: luminosityIconUrl
                },
                tooltip: {content: 'Luminosity sliders', position: {direction: 'bottom'}},
                action: (e) => {
                    const content = Layout.vertical({
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
                tooltip: {content: 'Opacity slider', position: {direction: 'bottom'}},
                action: (e) => {
                    const content = Layout.vertical({layout: [self.selector, self.opacitySettingsContent]});
                    self.update({content})
                }
            },
            colors: {
                icon: {
                    size: 'small',
                    url: colorIconUrl
                },
                tooltip: {content: 'Colormap', position: {direction: 'bottom'}},
                action: (e) => {
                    const content = Layout.vertical({layout: [self.selector, self.colorSettingsContent]});
                    self.update({content})
                }
            },
            pixel: {
                icon: {
                    size: 'small',
                    monochrome: true,
                    url: pixelHistIconUrl
                },
                tooltip: {content: 'Pixel cutouts', position: {direction: 'bottom'}},
                action: (e) => {
                    const content = Layout.vertical({layout: [self.selector, self.pixelSettingsContent]});
                    self.update({content})
                }
            },
            selected: 'opacity'
        }, aladin);

        // Define the contents

        let opacitySettingsContent = new Form({
            type: 'group',
            subInputs: [
                {
                    label: 'opacity:',
                    tooltip: {content: 1.0, position: {direction: 'bottom'}},
                    name: 'opacity',
                    type: 'range',
                    min: 0.0,
                    max: 1.0,
                    value: 1.0,
                    change: (e, slider) => {
                        const opacity = +e.target.value;
                        self.options.layer.setOpacity(opacity)
                        slider.update({value: opacity, tooltip: {content: opacity.toFixed(2), position: {direction: 'bottom'}}})
                    }
                }
            ]
        })

        let luminositySettingsContent = new Form({
            subInputs: [
                {
                    label: 'brightness:',
                    tooltip: {content: 'brightness', position: {direction: 'right'}},
                    name: 'brightness',
                    type: 'range',
                    min: -1,
                    max: 1,
                    ticks: [0.0],
                    value: 0.0,
                    change: (e, slider) => {
                        const brightness = +e.target.value;
                        self.options.layer.setBrightness(brightness)
                        slider.update({value: brightness, tooltip: {content: `${brightness.toFixed(3)}`, position: {direction: 'right'}}})
                    }
                },
                {
                    label: 'saturation:',
                    tooltip: {content: 'saturation', position: {direction: 'right'}},
                    name: 'saturation',
                    type: 'range',
                    min: -1,
                    max: 1,
                    ticks: [0.0],
                    value: 0.0,
                    change: (e, slider) => {
                        const saturation = +e.target.value
                        self.options.layer.setSaturation(saturation)
                        slider.update({value: saturation, tooltip: {content: `${saturation.toFixed(3)}`, position: {direction: 'right'}}})
                    }
                },
                {
                    label: 'contrast:',
                    tooltip: {content: 'contrast', position: {direction: 'right'}},
                    name: 'contrast',
                    type: 'range',
                    min: -1,
                    max: 1,
                    ticks: [0.0],
                    value: 0.0,
                    change: (e, slider) => {
                        const contrast = +e.target.value
                        self.options.layer.setContrast(contrast)
                        slider.update({value: contrast, tooltip: {content: `${contrast.toFixed(3)}`, position: {direction: 'right'}}})
                    }
                },
            ]
        });
        let pixelSettingsContent = new Form({
            type: 'group',
            subInputs: [{
                label: 'format:',
                type: 'select',
                name: 'fmt',
                value: 'jpeg',
                options: ['jpeg'],
                change(e) {
                    if (self.options.layer.imgFormat !== e.target.value)
                        self.options.layer.setImageFormat(e.target.value);
                },
                tooltip: {content: 'Formats availables', position: {direction: 'bottom'}}
            },
            {
                label: 'stretch:',
                type: 'select',
                name: 'stretch',
                value: 'linear',
                options: ['sqrt', 'linear', 'asinh', 'pow2', 'log'],
                change(e) {
                    self.options.layer.setColormap(self.options.layer.getColorCfg().getColormap(), {stretch: e.target.value});
                },
                tooltip: {content: 'stretch function', position: {direction: 'bottom'}}
            },
            {
                label: 'min cut:',
                type: 'number',
                cssStyle: {
                    width: '6rem',
                },
                tooltip: {content: 'Min cut', position: {direction: 'bottom'}},
                name: 'mincut',
                value: 0.0,
                change: (e) => {
                    self.options.layer.setCuts(+e.target.value, self.options.layer.getColorCfg().getCuts()[1])
                }
            },
            {
                type: 'number',
                label: 'max cut:',
                cssStyle: {
                    width: '6rem',
                },
                tooltip: {content: 'Max cut', position: {direction: 'bottom'}},
                name: 'maxcut',
                value: 1.0,
                change: (e) => {
                    self.options.layer.setCuts(self.options.layer.getColorCfg().getCuts()[0], +e.target.value)
                }
            }]
        }); 

        let colorSettingsContent = new Form({
            subInputs: [{
                label: 'colormap:',
                type: 'select',
                name: 'cmap',
                value: 'native',
                options: aladin.getListOfColormaps(),
                change: (e, cmapSelector) => {
                    self.options.layer.setColormap(e.target.value)
                },
            }]
        });

        super({
            close: false,
            ...options,
            content: Layout.vertical([selector, opacitySettingsContent]),
        },
        aladin.aladinDiv)
        self = this;

        this.aladin = aladin;
        this._addListeners()

        this.selector = selector;
        this.opacitySettingsContent = opacitySettingsContent;
        this.colorSettingsContent = colorSettingsContent;
        this.pixelSettingsContent = pixelSettingsContent;
        this.luminositySettingsContent = luminositySettingsContent;
    }

    update(options) {
        if (options.layer) {
            let hips = options.layer;
            let colorCfg = hips.getColorCfg();
            let stretch = colorCfg.stretch;
            let colormap = colorCfg.getColormap();

            let [minCut, maxCut] = colorCfg.getCuts();
            this.pixelSettingsContent.set('mincut', +minCut.toFixed(4))
            this.pixelSettingsContent.set('maxcut', +maxCut.toFixed(4))
            this.pixelSettingsContent.set('stretch', stretch)
            let fmtInput = this.pixelSettingsContent.getInput('fmt')

            fmtInput.innerHTML = '';
            for (const option of hips.formats) {
                fmtInput.innerHTML += "<option>" + option + "</option>";
            }
            fmtInput.value = hips.imgFormat;

            this.colorSettingsContent.set('cmap', colormap);
            this.opacitySettingsContent.set('opacity', hips.getOpacity());
            this.luminositySettingsContent.set('brightness', colorCfg.getBrightness());
            this.luminositySettingsContent.set('contrast', colorCfg.getContrast());
            this.luminositySettingsContent.set('saturation', colorCfg.getSaturation());
        }

        super.update(options)
    }

    _addListeners() {
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, (e) => {
            const hips = e.detail.layer;
            let selectedLayer = this.options.layer;
            if (selectedLayer && hips.layer === selectedLayer.layer) {
                let colorCfg = hips.getColorCfg();
                let stretch = colorCfg.stretch;
                let colormap = colorCfg.getColormap();
    
                let [minCut, maxCut] = colorCfg.getCuts();
                this.pixelSettingsContent.set('mincut', +minCut.toFixed(4))
                this.pixelSettingsContent.set('maxcut', +maxCut.toFixed(4))
                this.pixelSettingsContent.set('stretch', stretch)
                let fmtInput = this.pixelSettingsContent.getInput('fmt')
    
                fmtInput.innerHTML = '';
                for (const option of hips.formats) {
                    fmtInput.innerHTML += "<option>" + option + "</option>";
                }
                fmtInput.value = hips.imgFormat;
    
                this.colorSettingsContent.set('cmap', colormap);
                this.opacitySettingsContent.set('opacity', hips.getOpacity());
                this.luminositySettingsContent.set('brightness', colorCfg.getBrightness());
                this.luminositySettingsContent.set('contrast', colorCfg.getContrast());
                this.luminositySettingsContent.set('saturation', colorCfg.getSaturation());
            }
        });

        ALEvent.UPDATE_CMAP_LIST.listenedBy(this.aladin.aladinDiv, (e) => {
            let cmapSelector = this.colorSettingsContent.getInput('cmap');
            cmapSelector.update({options: this.aladin.getListOfColormaps()})
        });
    }
}
 