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

 import { HpxImageSurvey } from "../HpxImageSurvey.js";
 import { ALEvent } from "../events/ALEvent.js";
 import { HiPSSelector } from "./HiPSSelector.js";

 export class HiPSLayer {

    // Constructor
    constructor(aladin, view, survey) {
        this.aladin = aladin;
        this.view = view;
        this.survey = survey;
        this.hidden = false;
        this.lastOpacity = 1.0;

        // HiPS header div
        if (this.survey.layer === "base") {
            this.headerDiv = $(
                '<div class=".aladin-layer-header">' +
                    '<span class="indicator right-triangle">&nbsp;</span>' +
                    '<select class="aladin-surveySelection"></select>' +
                    '<button class="aladin-btn-small aladin-HiPSSelector" type="button" title="Search for a specific HiPS">🔍</button>' +
                    '<button class="aladin-btn-small aladin-layer-hide" type="button" title="Hide this layer">👁️</button>' +
                '</div>'
            );
        } else {
            this.headerDiv = $(
                '<div class=".aladin-layer-header">' +
                    '<span class="indicator right-triangle">&nbsp;</span>' +
                    '<select class="aladin-surveySelection"></select>' +
                    '<button class="aladin-btn-small aladin-HiPSSelector" type="button" title="Search a specific HiPS">🔍</button>' +
                    '<button class="aladin-btn-small aladin-layer-hide" type="button" title="Hide this layer">👁️</button>' +
                    '<button class="aladin-btn-small aladin-delete-layer" type="button" title="Delete this layer">❌</button>' +
                '</div>'
            );
        }

        // HiPS main options div
        let cmListStr = '';
        for (const cm of this.aladin.webglAPI.getAvailableColormapList()) {
            cmListStr += '<option>' + cm + '</option>';
        }
        // Add the native which is special:
        // - for FITS hipses, it is changed to grayscale
        // - for JPG/PNG hipses, we do not use any colormap in the backend
        cmListStr += '<option>native</option>';
        this.mainDiv = $('<div class="aladin-layer-main" style="display: none;">' +
                '<table class="aladin-options"><tbody>' +
                '  <tr><td>Color map</td><td><select class="">' + cmListStr + '</select></td></tr>' +
                '  <tr><td></td><td><label><input type="checkbox"> Reverse</label></td></tr>' +
                '  <tr><td>Stretch</td><td><select class=""><option>Pow2</option><option selected>Linear</option><option>Sqrt</option><option>Asinh</option><option>Log</option></select></td></tr>' +
                '  <tr><td>Format</td><td><select class=""></select></td></tr>' +
                '  <tr><td>Min cut</td><td><input type="number" class="aladin-cuts"></td></tr>' +
                '  <tr><td>Max cut</td><td><input type="number" class="aladin-cuts"></td></tr>' +
                '  <tr><td>Opacity</td><td><input class="" type="range" min="0" max="1" step="0.01"></td></tr>' +
                '</table> ' +
            '</div>');

        this.#addListeners();
        this.#updateHiPSLayerOptions();

        let self = this;
        this.layerChangedListener = (e) => {
            const survey = e.detail.survey;

            if (survey.layer === self.survey.layer) {
                // Update the survey to the new one
                self.survey = survey;

                self.#updateHiPSLayerOptions();
                self.#updateSurveysDropdownList();

            }
        };
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, this.layerChangedListener);
    }

    destroy() {
        ALEvent.HIPS_LAYER_CHANGED.remove(this.aladin.aladinDiv, this.layerChangedListener);
    }

    #addListeners() {
        const self = this;

        // HEADER DIV listeners
        // Click opener
        const clickOpener = this.headerDiv.find('.indicator');
        clickOpener.unbind("click");
        clickOpener.click(function () {
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

        // Update list of surveys
        self.#updateSurveysDropdownList();
        const surveySelector = this.headerDiv.find('.aladin-surveySelection');
        surveySelector.unbind("change");
        surveySelector.change(function () {
            var survey = HpxImageSurvey.SURVEYS[$(this)[0].selectedIndex];
            const hpxImageSurvey = new HpxImageSurvey(
                survey.url,
                self.view,
                survey.options
            );
            self.aladin.setOverlayImageLayer(hpxImageSurvey, null, self.survey.layer);
        });

        // Search HiPS button
        const hipsSelector = this.headerDiv.find('.aladin-HiPSSelector');
        hipsSelector.unbind("click");
        hipsSelector.click(function () {
            if (!self.hipsSelector) {
                const layerName = self.survey.layer;
                let fnURLSelected = function(url) {
                    self.aladin.setOverlayImageLayer(url, null, layerName);
                };
                let fnIdSelected = function(id) {
                    self.aladin.setOverlayImageLayer(id, null, layerName);
                };
                self.hipsSelector = new HiPSSelector(self.aladin.aladinDiv, fnURLSelected, fnIdSelected);
            }
            self.hipsSelector.show();
        });

        // Delete HiPS button
        const deleteLayer = this.headerDiv.find('.aladin-delete-layer');
        deleteLayer.unbind('click');
        deleteLayer.click(function () {
            const removeLayerEvent = new CustomEvent('remove-layer', {
                detail: self.survey.layer 
            });
            self.aladin.aladinDiv.dispatchEvent(removeLayerEvent);
        });

        // Hide HiPS button
        const hideLayer = this.headerDiv.find('.aladin-layer-hide');
        hideLayer.unbind("click");
        hideLayer.click(function () {
            self.hidden = !self.hidden;
            let opacitySlider = self.mainDiv.find('input').eq(3);

            let newOpacity = 0.0;
            if (self.hidden) {
                self.lastOpacity = self.survey.getOpacity();
                hideLayer.html('<p>&emsp;&nbsp;</p>');
            } else {
                newOpacity = self.lastOpacity;
                hideLayer.text('👁️');
            }
            // Update the opacity slider
            opacitySlider.val(newOpacity);
            opacitySlider.get(0).disabled = self.hidden;

            self.survey.setOpacity(newOpacity);

            // Update HpxImageSurvey.SURVEYS definition
            /*const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.opacity = newOpacity;
            surveyDef.options = options;*/
        });

        // MAIN DIV listeners
        // image format
        const format4ImgLayer = this.mainDiv.find('select').eq(2);
        const minCut4ImgLayer = this.mainDiv.find('input').eq(1);
        const maxCut4ImgLayer = this.mainDiv.find('input').eq(2);
        format4ImgLayer.unbind("change");
        format4ImgLayer.change(function() {
            const imgFormat = format4ImgLayer.val();

            self.survey.changeImageFormat(imgFormat);

            let minCut = 0;
            let maxCut = 1;
            if ( imgFormat === "FITS" ) {
                // FITS format
                minCut = self.survey.properties.minCutout;
                maxCut = self.survey.properties.maxCutout;
            }
            self.survey.setCuts([minCut, maxCut]);
            // update the cuts only
            
            minCut4ImgLayer.val(minCut);
            maxCut4ImgLayer.val(maxCut);

            // update HpxImageSurvey.SURVEYS definition
            /*const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.minCut = minCut;
            options.maxCut = maxCut;
            options.imgFormat = imgFormat;
            surveyDef.options = options;*/
        });
        // min/max cut
        minCut4ImgLayer.unbind("input blur");
        maxCut4ImgLayer.unbind("input blur");
        minCut4ImgLayer.add(maxCut4ImgLayer).on('input blur', function (e) {
            let minCutValue = parseFloat(minCut4ImgLayer.val());
            let maxCutValue = parseFloat(maxCut4ImgLayer.val());

            if (isNaN(minCutValue) || isNaN(maxCutValue)) {
                return;
            }
            self.survey.setCuts([minCutValue, maxCutValue]);

            // update HpxImageSurvey.SURVEYS definition
            /*const idxSelectedHiPS = self.surveySelectionDiv[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.minCut = minCutValue;
            options.maxCut = maxCutValue;
            surveyDef.options = options;*/
        });

        // color
        const colorMapSelect4ImgLayer = this.mainDiv.find('select').eq(0);
        colorMapSelect4ImgLayer.val('grayscale');
        let stretchSelect4ImgLayer = this.mainDiv.find('select').eq(1);

        let reverseCmCb = this.mainDiv.find('input').eq(0);

        reverseCmCb.unbind("change");
        colorMapSelect4ImgLayer.unbind("change");
        stretchSelect4ImgLayer.unbind("change");
        colorMapSelect4ImgLayer.add(reverseCmCb).add(stretchSelect4ImgLayer).change(function () {
            const reverse = reverseCmCb[0].checked;
            const cmap = colorMapSelect4ImgLayer.val();
            const stretch = stretchSelect4ImgLayer.val();

            self.survey.setColormap(cmap, {reversed: reverse, stretch: stretch});

            // update HpxImageSurvey.SURVEYS definition
            /*const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.colormap = cmap;
            options.stretch = stretch;
            options.reversed = reverse;
            surveyDef.options = options;*/
        });

        // opacity
        const opacity4ImgLayer = self.mainDiv.find('input').eq(3);
        opacity4ImgLayer.unbind("input");
        opacity4ImgLayer.on('input', function() {
            const opacity = +opacity4ImgLayer.val();
            self.survey.setOpacity(opacity);

            // update HpxImageSurvey.SURVEYS definition
            /*const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.opacity = opacity;
            surveyDef.options = options;*/
        });
    }

    #updateHiPSLayerOptions() {
        const reverseCmCb                 = this.mainDiv.find('input').eq(0);
        const colorMapSelect4ImgLayer = this.mainDiv.find('select').eq(0);
        const colorMapTr = this.mainDiv.find('tr').eq(0);
        const reverseTr = this.mainDiv.find('tr').eq(1);
        const stretchTr = this.mainDiv.find('tr').eq(2);

        const stretchSelect4ImgLayer  = this.mainDiv.find('select').eq(1);
        const formatSelect4ImgLayer   = this.mainDiv.find('select').eq(2);
        const opacity4ImgLayer        = this.mainDiv.find('input').eq(3);
        const formatTr                    = this.mainDiv.find('tr').eq(3);
        const minCutTr                    = this.mainDiv.find('tr').eq(4);
        const maxCutTr                    = this.mainDiv.find('tr').eq(5);
        const minCut = this.mainDiv.find('input').eq(1);
        const maxCut = this.mainDiv.find('input').eq(2);

        const properties = this.survey.properties;
        const options    = this.survey.options;
        const colored    = this.survey.colored;

        // format
        formatSelect4ImgLayer.empty();
        $.each(properties.formats, function (i, format) {
            formatSelect4ImgLayer.append($('<option>', { 
                value: format,
                text : format
            }));
        });

        const imgFormat = options.imgFormat;
        formatSelect4ImgLayer.val(imgFormat);

        // cuts
        if (colored) {
            colorMapTr.hide();
            reverseTr.hide();
            stretchTr.hide();

            minCutTr.hide();
            maxCutTr.hide();
        }
        else {
            colorMapTr.show();
            reverseTr.show();
            stretchTr.show();

            minCut.val(options.minCut);
            minCutTr.show();
            maxCut.val(options.maxCut);
            maxCutTr.show();
        }

        const opacity = options.opacity;
        opacity4ImgLayer.val(opacity);

        // TODO: traiter ce cas
        if (this.survey.colored) {
            return;
        }
        const cmap = options.colormap;
        const reverse = options.reversed;
        const stretch = options.stretch;

        reverseCmCb.prop('checked', reverse);
        colorMapSelect4ImgLayer.val(cmap);
        stretchSelect4ImgLayer.val(stretch);
    }

    #updateSurveysDropdownList() {
        let surveySelectionDiv = this.headerDiv.find('.aladin-surveySelection');

        let surveys = HpxImageSurvey.SURVEYS.sort(function (a, b) {
            if (!a.order) {
                return a.id > b.id;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });
        surveySelectionDiv.empty();

        if (this.survey) {
            let surveyFound = false;
            surveys.forEach(s => {
                const isCurSurvey = this.survey.properties.url.endsWith(s.url);
                surveySelectionDiv.append($("<option />").attr("selected", isCurSurvey).val(s.id).text(s.name));
                surveyFound |= isCurSurvey;
            });

            // The survey has not been found among the ones cached
            if (!surveyFound) {
                // Cache it
                HpxImageSurvey.SURVEYS.push({
                    id: this.survey.properties.id,
                    name: this.survey.properties.name,
                    maxOrder: this.survey.properties.maxOrder,
                    url: this.survey.properties.url,
                    options: this.survey.options
                });
                surveySelectionDiv.append($("<option />").attr("selected", true).val(this.survey.properties.id).text(this.survey.properties.name));
            } else {
                // Update the HpxImageSurvey
                const idxSelectedHiPS = surveySelectionDiv[0].selectedIndex;
                let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
                surveyDef.options = this.survey.options;
            }
        }
    }

    attachTo(parentDiv) {
        parentDiv.append(this.headerDiv)
            .append(this.mainDiv);

        this.#addListeners();
    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.headerDiv.style.display = 'none';
        this.mainDiv.style.display = 'none';
    }
}
