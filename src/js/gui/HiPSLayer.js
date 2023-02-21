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

import { ImageSurvey } from "../ImageSurvey.js";
import { ALEvent } from "../events/ALEvent.js";
import { HiPSSelector } from "./HiPSSelector.js";

import $ from 'jquery';

export class HiPSLayer {

    // Constructor
    constructor(aladin, survey) {
        this.aladin = aladin;
        this.survey = survey;
        this.hidden = false;
        this.lastOpacity = 1.0;

        // HiPS header div
        this.headerDiv = $(
            '<div class="aladin-layer-header aladin-hips-layer">' +
            '<span class="indicator right-triangle">&nbsp;</span>' +
            '<select class="aladin-surveySelection"></select>' +
            '<button class="aladin-btn-small aladin-layer-hide" type="button" title="Hide this layer">👁️</button>' +
            '<button class="aladin-btn-small aladin-HiPSSelector" type="button" title="Search for a specific HiPS">🔍</button>' +
            '<button class="aladin-btn-small aladin-delete-layer" type="button" title="Delete this layer">❌</button>' +
            '</div>'
        );

        if (this.survey.layer === "base") {
            let deleteLayerBtn = this.headerDiv[0].getElementsByClassName("aladin-delete-layer")[0];
            deleteLayerBtn.disabled = true;
            deleteLayerBtn.style.backgroundColor = 'lightgray';
            deleteLayerBtn.style.borderColor = 'gray';

            // This is how to color emojis:
            // see: https://stackoverflow.com/questions/32413731/color-for-unicode-emoji
            deleteLayerBtn.style.color = 'transparent';
            deleteLayerBtn.style.textShadow = '0 0 0 gray';
        }

        // HiPS main options div
        let cmListStr = '';
        for (const cm of this.aladin.webglAPI.getAvailableColormapList()) {
            cmListStr += '<option>' + cm + '</option>';
        }

        this.cmap = "native";
        this.color = "#ff0000";

        this.mainDiv = $('<div class="aladin-frame" style="display:none">' +
            '<div class="aladin-options">' +
            // colormap
            '  <div class="row"><div class="col-label">Colormap</div><div class="col-input"><select class="colormap-selector">' + cmListStr + '</select></div></div>' +
            '  <label><div class="row"><div class="col-label">Reverse</div><div class="col-input"><input type="checkbox" class="reversed"></div></div></label>' +
            '  <div class="row"><div class="col-label"><label>Stretch</label></div><div class="col-input"><select class="stretch"><option>pow2</option><option selected>linear</option><option>sqrt</option><option>asinh</option><option>log</option></select></div></div>' +
            '  <div class="row"><div class="col-label"><label>Format</label></div><div class="col-input"><select class="format"></select></div></div>' +
            '  <div class="row"><div class="col-label"><label>Min cut</label></div><div class="col-input"><input type="number" class="min-cut"></div></div>' +
            '  <div class="row"><div class="col-label"><label>Max cut</label></div><div class="col-input"><input type="number" class="max-cut"></div></div>' +
            // tonal corrections
            '  <div class="row"><div class="col-label"><label>Gamma</label></div><div class="col-input"><input class="gamma" type="number" value="1.0" min="0.1" max="10.0" step="0.01"></div></div>' +
            '  <div class="row"><div class="col-label"><label>Color Sat.</label></div><div class="col-input"><input class="saturation" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div></div>' +
            '  <div class="row"><div class="col-label"><label>Contrast</label></div><div class="col-input"><input class="contrast" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div></div>' +
            '  <div class="row"><div class="col-label"><label>Brightness</label></div><div class="col-input"><input class="brightness" type="range" value="0.0" min="-1.0" max="1.0" step="0.01"></div></div>' +
            // blending mode
            '  <div class="row"><div class="col-label"><label>Blending mode</label></div><div class="col-input"><select class="blending"><option>additive</option><option selected>default</option></select></div></div>' +
            // opacity
            '  <div class="row"><div class="col-label"><label>Opacity</label></div><div class="col-input"><input class="opacity" type="range" min="0" max="1" step="0.01"></div></div>' +
            '</div> ' +
            '</div>');

        this._addListeners();
        this._updateHiPSLayerOptions();

        let self = this;
        this.layerChangedListener = function(e) {
            const survey = e.detail.survey;

            if (survey.layer === self.survey.layer) {
                // Update the survey to the new one
                self.survey = survey;
                self._updateHiPSLayerOptions();
            }
            self._updateSurveysDropdownList();
        };
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, this.layerChangedListener);
    }

    destroy() {
        ALEvent.HIPS_LAYER_CHANGED.remove(this.aladin.aladinDiv, this.layerChangedListener);
    }

    _addListeners() {
        const self = this;
        // HEADER DIV listeners
        // Click opener
        const clickOpener = this.headerDiv.find('.indicator');
        clickOpener.off("click");
        clickOpener.on("click", function () {
            if (clickOpener.hasClass('right-triangle')) {
                clickOpener.removeClass('right-triangle');
                clickOpener.addClass('down-triangle');
                self.mainDiv.slideDown(300);
            }
            else {
                clickOpener.removeClass('down-triangle');
                clickOpener.addClass('right-triangle');
                self.mainDiv.slideUp(300);
            }
        });

        this.headerDiv.off("click");
        this.headerDiv.on("click", () => {
            self.aladin.aladinDiv.dispatchEvent(new CustomEvent('select-layer', {
                detail: self
            }));
        })

        // Click on aladin options should select the layer clicked
        // Update list of surveys
        self._updateSurveysDropdownList();
        const surveySelector = this.headerDiv.find('.aladin-surveySelection');
        surveySelector.off("change");
        surveySelector.change(function () {
            let cfg = ImageSurvey.SURVEYS[$(this)[0].selectedIndex];
            if (self.hidden) {
                cfg.options.opacity = 0.0;
            }

            const survey = self.aladin.createImageSurvey(
                cfg.id,
                cfg.name,
                cfg.url,
                undefined,
                cfg.maxOrder,
                cfg.options
            );
            self.aladin.setOverlayImageLayer(survey, self.survey.layer);
        });

        // Search HiPS button
        const hipsSelector = this.headerDiv.find('.aladin-HiPSSelector');
        hipsSelector.off("click");
        hipsSelector.on("click", function () {
            if (!self.hipsSelector) {
                self.hipsSelector = new HiPSSelector(self.aladin.aladinDiv, (IDOrURL) => {
                    const layerName = self.survey.layer;
                    self.aladin.setOverlayImageLayer(IDOrURL, layerName);
                }, self.aladin);
            }

            self.hipsSelector.show();
        });

        // Delete HiPS button
        const deleteLayer = this.headerDiv.find('.aladin-delete-layer');
        deleteLayer.off("click");
        deleteLayer.on("click", function () {
            const removeLayerEvent = new CustomEvent('remove-layer', {
                detail: self.survey.layer
            });
            self.aladin.aladinDiv.dispatchEvent(removeLayerEvent);
        });

        // Hide HiPS button
        const hideLayer = this.headerDiv.find('.aladin-layer-hide');
        hideLayer.off("click");
        hideLayer.on("click", function () {
            self.hidden = !self.hidden;
            let opacitySlider = self.mainDiv.find('.opacity').eq(0);

            let newOpacity = 0.0;
            if (self.hidden) {
                self.lastOpacity = self.survey.getOpacity();
                hideLayer.text('');
            } else {
                newOpacity = self.lastOpacity;
                hideLayer.text('👁️');
            }
            // Update the opacity slider
            opacitySlider.val(newOpacity);
            opacitySlider.get(0).disabled = self.hidden;

            self.survey.setOpacity(newOpacity);
        });

        // MAIN DIV listeners
        // blending method
        const blendingSelector = this.mainDiv.find('.blending').eq(0);

        blendingSelector.off("change");
        blendingSelector.change(function () {
            let mode = blendingSelector.val()
            self.survey.setBlendingConfig( mode === "additive" );
        });
        
        // image format
        const format4ImgLayer = this.mainDiv.find('.format').eq(0);
        const minCut4ImgLayer = this.mainDiv.find('.min-cut').eq(0);
        const maxCut4ImgLayer = this.mainDiv.find('.max-cut').eq(0);
        format4ImgLayer.off("change");
        format4ImgLayer.on("change", function () {
            const imgFormat = format4ImgLayer.val();

            self.survey.setImageFormat(imgFormat);

            let minCut = 0;
            let maxCut = 1;
            if (imgFormat === "fits") {
                // FITS format
                minCut = self.survey.properties.minCutout;
                maxCut = self.survey.properties.maxCutout;
            }
            self.survey.setCuts(minCut, maxCut);

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
            self.survey.setCuts(minCutValue, maxCutValue);
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
            self.survey.setColormap(cmap, { reversed: reverse, stretch: stretch });
        });

        // opacity
        const opacity4ImgLayer = self.mainDiv.find('.opacity').eq(0);
        opacity4ImgLayer.off("input");
        opacity4ImgLayer.on('input', function () {
            const opacity = +opacity4ImgLayer.val();
            self.survey.setOpacity(opacity);
        });

        // gamma
        const gamma4ImgLayer = self.mainDiv.find('.gamma').eq(0);
        gamma4ImgLayer.off("change blur");
        gamma4ImgLayer.on('change blur', function () {
            const gamma = parseFloat(gamma4ImgLayer.val()) || 1.0;

            self.survey.setGamma(gamma);

            const trueGamma = self.survey.getColorCfg().getGamma();
            if (gamma !== trueGamma) {
                gamma4ImgLayer.val(trueGamma);
            }
        });

        // saturation
        const sat4ImgLayer = self.mainDiv.find('.saturation').eq(0);
        sat4ImgLayer.off("input");
        sat4ImgLayer.on('input', function (e) {
            const saturation = parseFloat(sat4ImgLayer.val()) || 0.0;

            self.survey.setSaturation(saturation);

            const trueSaturation = self.survey.getColorCfg().getSaturation();
            if (saturation !== trueSaturation) {
                sat4ImgLayer.val(trueSaturation);
            }
        });

        // contrast
        const contrast4ImgLayer = self.mainDiv.find('.contrast').eq(0);
        contrast4ImgLayer.off("input");
        contrast4ImgLayer.on('input', function (e) {
            const contrast = parseFloat(contrast4ImgLayer.val()) || 0.0;

            self.survey.setContrast(contrast);

            const trueContrast = self.survey.getColorCfg().getContrast();
            if (contrast !== trueContrast) {
                contrast4ImgLayer.val(trueContrast);
            }
        });

        // brightness
        const brightness4ImgLayer = self.mainDiv.find('.brightness').eq(0);
        brightness4ImgLayer.off("input");
        brightness4ImgLayer.on('input', function (e) {
            const brightness = parseFloat(brightness4ImgLayer.val()) || 0.0;

            self.survey.setBrightness(brightness);

            const trueBrightness = self.survey.getColorCfg().getBrightness();
            if (brightness !== trueBrightness) {
                brightness4ImgLayer.val(trueBrightness);
            }
        });
    }

    _updateHiPSLayerOptions() {
        const colorMapTr = this.mainDiv.find('.row').eq(0);
        const reverseTr = this.mainDiv.find('.row').eq(1);
        const stretchTr = this.mainDiv.find('.row').eq(2);
        const formatTr = this.mainDiv.find('.row').eq(3);
        const minCutTr = this.mainDiv.find('.row').eq(4);
        const maxCutTr = this.mainDiv.find('.row').eq(5);

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
        
        this.survey.formats.forEach(fmt => {
            formatSelect4ImgLayer.append($('<option>', {
                value: fmt,
                text: fmt
            }));
        });

        const colorCfg = this.survey.getColorCfg();
        const cmap = colorCfg.colormap;
        const reverse = colorCfg.reversed;
        const stretch = colorCfg.stretch;

        // Update radio color/colormap selection
        const imgFormat = this.survey.imgFormat;
        formatSelect4ImgLayer.val(imgFormat);

        // Update radio color/colormap selection
        const additive = colorCfg.getBlendingConfig();
        blendingSelect4ImgLayer.val(additive ? "additive" : "default");

        // cuts
        colorMapTr[0].style.display = "flex";
        reverseTr[0].style.display = "flex";
        stretchTr[0].style.display = "flex";

        if (colorCfg.minCut) {
            if (parseFloat(minCut.val()) != colorCfg.minCut) {
                minCut.val(parseFloat(colorCfg.minCut.toFixed(5)));
            }
        }
        else {
            minCut.val(0.0);
        }

        minCutTr[0].style.display = "flex";

        if (colorCfg.maxCut) {
            if (parseFloat(maxCut.val()) != colorCfg.maxCut) {
                maxCut.val(parseFloat(colorCfg.maxCut.toFixed(5)));
            }
        }
        else {
            maxCut.val(0.0);
        }
        maxCutTr[0].style.display = "flex";
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

    _updateSurveysDropdownList() {
        let surveySelectionDiv = this.headerDiv.find('.aladin-surveySelection');

        let surveys = ImageSurvey.SURVEYS.sort(function (a, b) {
            if (!a.order) {
                return a.name > b.name ? 1 : -1;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });
        surveySelectionDiv.empty();

        if (this.survey) {
            let surveyFound = false;
            surveys.forEach(s => {
                const isCurSurvey = this.survey.id.endsWith(s.id);
                surveySelectionDiv.append($("<option />").attr("selected", isCurSurvey).val(s.id).text(s.name));
                surveyFound |= isCurSurvey;
            });

            // The survey has not been found among the ones cached
            if (!surveyFound) {
                // Cache it
                console.warn(this.survey, " has not been found in SURVEYS!")
            } else {
                // Update the ImageSurvey
                const idxSelectedHiPS = surveySelectionDiv[0].selectedIndex;
                let surveyDef = ImageSurvey.SURVEYS[idxSelectedHiPS];
                surveyDef.options = this.survey.metadata;
            }
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