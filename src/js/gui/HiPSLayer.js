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
import { HiPSSelector } from "./HiPSSelector.js";

import $ from 'jquery';
import { ActionButton } from "./widgets/ActionButton.js";

export class HiPSLayer {

    // Constructor
    constructor(aladin, layer) {
        this.aladin = aladin;
        this.layer = layer;
        this.hidden = false;
        this.lastOpacity = 1.0;

        // HiPS header div
        this.headerDiv = $(
            '<div>' +
                '<div class="aladin-layer-header aladin-horizontal-list"></div>' +
            '</div>'
        );

        let layerHeaderEl = this.headerDiv[0].querySelector(".aladin-layer-header");

        let self = this;
    
        let clickOpenerBtn = new ActionButton({
            content: "▶",
            backgroundColor: '#eaeaea',
            color: 'black',
            info: 'Open the survey edition panel',
            action(e) {
                if (clickOpenerBtn.opt.content === '▶') {
                    clickOpenerBtn.attach({
                        info: 'Close the survey edition panel',
                        content: '▼',
                    });
                    self.mainDiv.slideDown(300);
                }
                else {
                    clickOpenerBtn.attach({
                        info: 'Open the survey edition panel',
                        content: '▶',
                    });
                    self.mainDiv.slideUp(300);
                }
            }
        }, layerHeaderEl);

        layerHeaderEl.appendChild((() => {
            let selector = $('<select class="aladin-input aladin-layerSelection"></select>');
            return selector[0];
        })());

        let hideLayerBtn = new ActionButton({
            content: "👁️",
            backgroundColor: '#eaeaea',
            info: 'Hide the layer',
            action(e) {
                self.hidden = !self.hidden;
                let opacitySlider = self.mainDiv.find('.opacity').eq(0);

                let newOpacity = 0.0;
                if (self.hidden) {
                    self.lastOpacity = self.layer.getOpacity();
                    hideLayerBtn.attach({content: ' '});
                } else {
                    newOpacity = self.lastOpacity;
                    hideLayerBtn.attach({content: '👁️'});
                }
                // Update the opacity slider
                opacitySlider.val(newOpacity);
                opacitySlider.get(0).disabled = self.hidden;

                self.layer.setOpacity(newOpacity);
            }
        }, layerHeaderEl);

        new ActionButton({
            content: "🔍",
            backgroundColor: '#eaeaea',
            info: 'Search for a survey (HiPS)',
            action(e) {
                if (!self.hipsSelector) {
                    self.hipsSelector = new HiPSSelector(self.aladin.aladinDiv, (IDOrURL) => {
                        const layerName = self.layer.layer;
                        self.aladin.setOverlayImageLayer(IDOrURL, layerName);
                    }, self.aladin);
                }
        
                self.hipsSelector.show();
            }
        }, layerHeaderEl);

        let deleteLayerBtn = new ActionButton({
            content: "❌",
            backgroundColor: '#eaeaea',
            info: 'Delete this layer',
            action(e) {
                self.aladin.aladinDiv.dispatchEvent(new CustomEvent('remove-layer', {
                    detail: self.layer.layer
                }));
            }
        }, layerHeaderEl);

        // Add a centered on button for images
        if (this.layer.subtype === "fits") {
            let layerSelector = this.headerDiv[0].querySelector(".aladin-layerSelection");
            new ActionButton({
                content: "🎯",
                backgroundColor: '#eaeaea',
                info: 'Focus on the FITS',
                action(e) {
                    self.layer.focusOn();
                }
            }, layerSelector, 'afterend');
        }

        if (this.layer.layer === "base") {
            deleteLayerBtn.attach({backgroundColor: 'lightgray', disable: true});
        }

        // HiPS main options div
        let cmListStr = '';
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
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, this.layerChangedListener);
    }

    destroy() {
        ALEvent.HIPS_LAYER_CHANGED.remove(this.aladin.aladinDiv, this.layerChangedListener);
    }

    _addListeners() {
        const self = this;
        // HEADER DIV listeners

        this.headerDiv.off("click");
        this.headerDiv.on("click", () => {
            self.aladin.aladinDiv.dispatchEvent(new CustomEvent('select-layer', {
                detail: self
            }));
        })

        // Click on aladin options should select the layer clicked
        // Update list of surveys
        self._updateLayersDropdownList();
        const layerSelector = this.headerDiv.find('.aladin-layerSelection');
        layerSelector.off("change");
        layerSelector.on("change", (e) => {
            let cfg = ImageLayer.LAYERS[layerSelector[0].selectedIndex];
            let layer;
            
            // Max order is specific for surveys
            if (cfg.subtype === "fits") {
                // FITS
                layer = self.aladin.createImageFITS(
                    cfg.url,
                    cfg.name,
                    cfg.options,
                );
            } else {
                // HiPS
                layer = self.aladin.createImageSurvey(
                    cfg.id,
                    cfg.name,
                    cfg.url,
                    undefined,
                    cfg.maxOrder,
                    cfg.options
                );
            }

            if (self.hidden) {
                layer.setAlpha(0.0);
            }

            self.aladin.setOverlayImageLayer(layer, self.layer.layer);
        });

        // MAIN DIV listeners
        // blending method
        const blendingSelector = this.mainDiv.find('.blending').eq(0);

        blendingSelector.off("change");
        blendingSelector.change(function () {
            let mode = blendingSelector.val()
            self.layer.setBlendingConfig( mode === "additive" );
        });
        
        // image format
        const format4ImgLayer = this.mainDiv.find('.format').eq(0);
        const minCut4ImgLayer = this.mainDiv.find('.min-cut').eq(0);
        const maxCut4ImgLayer = this.mainDiv.find('.max-cut').eq(0);
        format4ImgLayer.off("change");
        format4ImgLayer.on("change", function () {
            const imgFormat = format4ImgLayer.val();

            self.layer.setImageFormat(imgFormat);

            let minCut = 0;
            let maxCut = 1;

            if (imgFormat === "fits") {
                // FITS format
                minCut = self.layer.properties.minCutout;
                maxCut = self.layer.properties.maxCutout;
            }
            self.layer.setCuts(minCut, maxCut);

            // update the cuts only
            minCut4ImgLayer.val(parseFloat(minCut.toFixed(5)));
            maxCut4ImgLayer.val(parseFloat(maxCut.toFixed(5)));
        });
        // min/max cut
        minCut4ImgLayer.off("change blur");
        maxCut4ImgLayer.off("change blur");
        minCut4ImgLayer.add(maxCut4ImgLayer).on('change blur', function (e) {
            let minCutValue = parseFloat(minCut4ImgLayer.val());
            let maxCutValue = parseFloat(maxCut4ImgLayer.val());

            if (isNaN(minCutValue) || isNaN(maxCutValue)) {
                return;
            }
            self.layer.setCuts(minCutValue, maxCutValue);
        });

        // colormap
        const colorMapSelect4ImgLayer = this.mainDiv.find('.colormap-selector').eq(0);
        const stretchSelect4ImgLayer = this.mainDiv.find('.stretch').eq(0);
        const reverseCmCb = this.mainDiv.find('.reversed').eq(0);

        reverseCmCb.off("change");
        colorMapSelect4ImgLayer.off("change");
        stretchSelect4ImgLayer.off("change");
        colorMapSelect4ImgLayer.add(reverseCmCb).add(stretchSelect4ImgLayer).change(function () {
            const stretch = stretchSelect4ImgLayer.val();
            const reverse = reverseCmCb[0].checked;

            // Color map case
            const cmap = colorMapSelect4ImgLayer.val();
            self.layer.setColormap(cmap, { reversed: reverse, stretch: stretch });
        });

        // opacity
        const opacity4ImgLayer = self.mainDiv.find('.opacity').eq(0);
        opacity4ImgLayer.off("input");
        opacity4ImgLayer.on('input', function () {
            const opacity = +opacity4ImgLayer.val();
            self.layer.setOpacity(opacity);
        });

        // gamma
        const gamma4ImgLayer = self.mainDiv.find('.gamma').eq(0);
        gamma4ImgLayer.off("change blur");
        gamma4ImgLayer.on('change blur', function () {
            const gamma = parseFloat(gamma4ImgLayer.val()) || 1.0;

            self.layer.setGamma(gamma);

            const trueGamma = self.layer.getColorCfg().getGamma();
            if (gamma !== trueGamma) {
                gamma4ImgLayer.val(trueGamma);
            }
        });

        // saturation
        const sat4ImgLayer = self.mainDiv.find('.saturation').eq(0);
        sat4ImgLayer.off("input");
        sat4ImgLayer.on('input', function (e) {
            const saturation = parseFloat(sat4ImgLayer.val()) || 0.0;

            self.layer.setSaturation(saturation);

            const trueSaturation = self.layer.getColorCfg().getSaturation();
            if (saturation !== trueSaturation) {
                sat4ImgLayer.val(trueSaturation);
            }
        });

        // contrast
        const contrast4ImgLayer = self.mainDiv.find('.contrast').eq(0);
        contrast4ImgLayer.off("input");
        contrast4ImgLayer.on('input', function (e) {
            const contrast = parseFloat(contrast4ImgLayer.val()) || 0.0;

            self.layer.setContrast(contrast);

            const trueContrast = self.layer.getColorCfg().getContrast();
            if (contrast !== trueContrast) {
                contrast4ImgLayer.val(trueContrast);
            }
        });

        // brightness
        const brightness4ImgLayer = self.mainDiv.find('.brightness').eq(0);
        brightness4ImgLayer.off("input");
        brightness4ImgLayer.on('input', function (e) {
            const brightness = parseFloat(brightness4ImgLayer.val()) || 0.0;

            self.layer.setBrightness(brightness);

            const trueBrightness = self.layer.getColorCfg().getBrightness();
            if (brightness !== trueBrightness) {
                brightness4ImgLayer.val(trueBrightness);
            }
        });
    }

    _updateHiPSLayerOptions() {
        const reverseCmCb = this.mainDiv.find('.reversed').eq(0);
        const colorMapSelect4ImgLayer = this.mainDiv.find('.colormap-selector').eq(0);
        const stretchSelect4ImgLayer = this.mainDiv.find('.stretch').eq(0);
        const formatSelect4ImgLayer = this.mainDiv.find('.format').eq(0);
        const opacity4ImgLayer = this.mainDiv.find('.opacity').eq(0);
        const gamma4ImgLayer = this.mainDiv.find('.gamma').eq(0);
        const contrast4ImgLayer = this.mainDiv.find('.contrast').eq(0);
        const brightness4ImgLayer = this.mainDiv.find('.brightness').eq(0);
        const sat4ImgLayer = this.mainDiv.find('.saturation').eq(0);
        const blendingSelect4ImgLayer = this.mainDiv.find('.blending').eq(0);

        const minCut = this.mainDiv.find('.min-cut').eq(0);
        const maxCut = this.mainDiv.find('.max-cut').eq(0);

        formatSelect4ImgLayer.empty();
        
        this.layer.properties.formats.forEach(fmt => {
            formatSelect4ImgLayer.append($('<option>', {
                value: fmt,
                text: fmt
            }));
        });

        const colorCfg = this.layer.getColorCfg();
        const cmap = colorCfg.colormap;
        const reverse = colorCfg.reversed;
        const stretch = colorCfg.stretch;

        // Update radio color/colormap selection
        const imgFormat = this.layer.imgFormat;
        formatSelect4ImgLayer.val(imgFormat);

        // Update radio color/colormap selection
        const additive = colorCfg.getBlendingConfig();
        blendingSelect4ImgLayer.val(additive ? "additive" : "default");

        // cuts
        //colorMapTr[0].style.display = "flex";
        //reverseTr[0].style.display = "flex";
        //stretchTr[0].style.display = "flex";

        if (colorCfg.minCut) {
            if (parseFloat(minCut.val()) != colorCfg.minCut) {
                minCut.val(parseFloat(colorCfg.minCut.toFixed(5)));
            }
        }
        else {
            minCut.val(0.0);
        }

        //minCutTr[0].style.display = "flex";

        if (colorCfg.maxCut) {
            if (parseFloat(maxCut.val()) != colorCfg.maxCut) {
                maxCut.val(parseFloat(colorCfg.maxCut.toFixed(5)));
            }
        }
        else {
            maxCut.val(0.0);
        }
        //maxCutTr[0].style.display = "flex";
        // save opacity
        const opacity = colorCfg.getOpacity();
        opacity4ImgLayer.val(opacity);
        // save gamma
        const gamma = colorCfg.getGamma();
        gamma4ImgLayer.val(gamma);
        // save saturation
        const saturation = colorCfg.getSaturation();
        sat4ImgLayer.val(saturation);
        // save brightness
        const brightness = colorCfg.getBrightness();
        brightness4ImgLayer.val(brightness);
        // save contrast
        const contrast = colorCfg.getContrast();
        contrast4ImgLayer.val(contrast);
        // save cmap
        colorMapSelect4ImgLayer.val(cmap);
        this.cmap = cmap;
        // save reverse
        reverseCmCb.prop('checked', reverse);
        // save stretch
        stretchSelect4ImgLayer.val(stretch);
    }

    _updateLayersDropdownList() {
        let layerSelectDiv = this.headerDiv.find('.aladin-layerSelection');

        let layers = ImageLayer.LAYERS.sort(function (a, b) {
            if (!a.order) {
                return a.name > b.name ? 1 : -1;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });
        layerSelectDiv.empty();

        if (this.layer) {
            //let layerFound = false;
            layers.forEach(l => {
                const isCurLayer = this.layer.id.endsWith(l.id);
                layerSelectDiv.append($("<option />").attr("selected", isCurLayer).val(l.id).text(l.name));
                //layerFound |= isCurLayer;
            });

            /*// The survey has not been found among the ones cached
            if (!layerFound) {
                throw this.layer + " has not been found in the list of layers!"
            } else {
                // Update the ImageSurvey
                const idxSelectLayer = layerSelectDiv[0].selectedIndex;
                let layer = ImageLayer.LAYERS[idxSelectLayer];
                layer.options = this.layer.metadata;
            }*/
        }
    }

    attachTo(parentDiv) {
        this.headerDiv.append(this.mainDiv);
        parentDiv.append(this.headerDiv);

        this._addListeners();
    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.headerDiv.style.display = 'none';
        this.mainDiv.style.display = 'none';
    }
}