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
    constructor(aladin, view, layer) {
        this.aladin = aladin;
        this.view = view;
        this.layer = layer;

        // HiPS header div
        if (this.layer === "base") {
            this.headerDiv = $(
                '<div class=".aladin-layer-header">' +
                    '<span class="indicator right-triangle">&nbsp;</span>' +
                    '<select class="aladin-surveySelection"></select>' +
                    '<button class="aladin-btn-small aladin-HiPSSelector" type="button" title="Search a specific HiPS">🔍</button>' +
                '</div>'
            );
        } else {
            this.headerDiv = $(
                '<div class=".aladin-layer-header">' +
                    '<span class="indicator right-triangle">&nbsp;</span>' +
                    '<select class="aladin-surveySelection"></select>' +
                    '<button class="aladin-btn-small aladin-HiPSSelector" type="button" title="Search a specific HiPS">🔍</button>' +
                    '<button class="aladin-btn-small aladin-delete-layer" type="button" title="Delete this layer">🗑️</button>' +
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
    }

    #addListeners() {
        const self = this;
        ALEvent.HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
            if (e.detail.layer === self.layer) {
                self.#updateHiPSLayerOptions();
                self.#updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
            }
        });

        // HEADER DIV listeners
        // Click opener
        this.headerDiv.find('.indicator').unbind("click");
        let indicator = this.headerDiv.find('.indicator');
        indicator.click(function () {
            if (indicator.hasClass('right-triangle')) {
                indicator.removeClass('right-triangle');
                indicator.addClass('down-triangle');
                self.mainDiv.slideDown(300);
            }
            else {
                indicator.removeClass('down-triangle');
                indicator.addClass('right-triangle');
                self.mainDiv.slideUp(300);
            }
        });

        // Update list of surveys
        self.#updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        this.headerDiv.find('.aladin-surveySelection').change(function () {
            var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
            //console.log("survey, chosen ", survey)
            const hpxImageSurvey = new HpxImageSurvey(
                survey.url,
                self.view,
                survey.options
            );
            self.aladin.setOverlayImageLayer(hpxImageSurvey, null, self.layer);
        });

        // Search HiPS button
        this.headerDiv.find('.aladin-HiPSSelector').unbind("click");
        this.headerDiv.find('.aladin-HiPSSelector').click(function () {
            if (!self.hipsSelector) {
                let fnURLSelected = function(url) {
                    self.aladin.setOverlayImageLayer(url, null, self.layer);
                };
                let fnIdSelected = function(id) {
                    self.aladin.setOverlayImageLayer(id, null, self.layer);
                };
                self.hipsSelector = new HiPSSelector(self.aladin.aladinDiv, fnURLSelected, fnIdSelected);
            }
            self.hipsSelector.show();
        });

        // Delete HiPS button
        this.headerDiv.find('.aladin-delete-layer').unbind('click');
        this.headerDiv.find('.aladin-delete-layer').click(function () {
            const removeLayerEvent = new CustomEvent('remove-layer', {
                detail: self.layer 
            });
            self.aladin.aladinDiv.dispatchEvent(removeLayerEvent);
        });

        // MAIN DIV listeners
        // image format
        const format4ImgLayer = this.mainDiv.find('select').eq(2);
        const minCut4ImgLayer = this.mainDiv.find('input').eq(1);
        const maxCut4ImgLayer = this.mainDiv.find('input').eq(2);

        format4ImgLayer.change(function() {
            const imgFormat = format4ImgLayer.val();
            const imgLayer = self.aladin.getOverlayImageLayer(self.layer);

            imgLayer.changeImageFormat(imgFormat);

            let minCut = 0;
            let maxCut = 1;
            if ( imgFormat === "FITS" ) {
                // FITS format
                minCut = imgLayer.properties.minCutout;
                maxCut = imgLayer.properties.maxCutout;
            }
            imgLayer.setCuts([minCut, maxCut]);
            // update the cuts only
            
            minCut4ImgLayer.val(minCut);
            maxCut4ImgLayer.val(maxCut);

            // update HpxImageSurvey.SURVEYS definition
            const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.minCut = minCut;
            options.maxCut = maxCut;
            options.imgFormat = imgFormat;
            surveyDef.options = options;
        });
        // min/max cut
        const minCut = this.mainDiv.find('input').eq(1);
        const maxCut = this.mainDiv.find('input').eq(2);
        minCut.add(maxCut).on('input blur', function (e) {
            let minCutValue = parseFloat(minCut.val());
            let maxCutValue = parseFloat(maxCut.val());

            if (isNaN(minCutValue) || isNaN(maxCutValue)) {
                return;
            }
            self.aladin.getOverlayImageLayer(self.layer).setCuts([minCutValue, maxCutValue]);

            // update HpxImageSurvey.SURVEYS definition
            const idxSelectedHiPS = self.surveySelectionDiv[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.minCut = minCutValue;
            options.maxCut = maxCutValue;
            surveyDef.options = options;
        });

        // color
        let colorMapSelect4ImgLayer = this.mainDiv.find('select').eq(0);
        colorMapSelect4ImgLayer.val('grayscale');
        let stretchSelect4ImgLayer = this.mainDiv.find('select').eq(1);

        let reverseCmCb = this.mainDiv.find('input').eq(0);

        colorMapSelect4ImgLayer.add(reverseCmCb).add(stretchSelect4ImgLayer).change(function () {
            const reverse = reverseCmCb[0].checked;
            const cmap = colorMapSelect4ImgLayer.val();
            const stretch = stretchSelect4ImgLayer.val();

            self.aladin.getOverlayImageLayer(self.layer).setColormap(cmap, {reversed: reverse, stretch: stretch});

            // update HpxImageSurvey.SURVEYS definition
            const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.colormap = cmap;
            options.stretch = stretch;
            options.reversed = reverse;
            surveyDef.options = options;
        });

        // opacity
        let opacity4ImgLayer = self.mainDiv.find('input').eq(3);
        opacity4ImgLayer.on('input', function() {
            const opacity = +opacity4ImgLayer.val();
            self.aladin.getOverlayImageLayer(self.layer).setOpacity(opacity);

            // update HpxImageSurvey.SURVEYS definition
            const idxSelectedHiPS = self.headerDiv.find('.aladin-surveySelection')[0].selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            let options = surveyDef.options || {};
            options.opacity = opacity;
            surveyDef.options = options;
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

        const survey = this.aladin.getOverlayImageLayer(this.layer);
        const properties = survey.properties;
        const options    = survey.options;
        const meta       = survey.meta;
        const colored    = survey.colored;

        // format
        formatSelect4ImgLayer.empty();
        $.each(properties.formats, function (i, format) {
            formatSelect4ImgLayer.append($('<option>', { 
                value: format,
                text : format
            }));
        });

        const imgFormat = survey.options.imgFormat;
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

        const opacity = meta.opacity;
        opacity4ImgLayer.val(opacity);

        // TODO: traiter ce cas
        if (!meta.color || !meta.color.grayscale) {
            return;
        }
        const cmap = meta.color.grayscale.color.colormap.name;
        const reverse = meta.color.grayscale.color.colormap.reversed;
        const stretch = meta.color.grayscale.stretch;

        reverseCmCb.prop('checked', reverse);
        colorMapSelect4ImgLayer.val(cmap);
        stretchSelect4ImgLayer.val(stretch);
    }

    #updateSurveysDropdownList(surveys) {
        const self = this;

        let surveySelectionDiv = this.headerDiv.find('.aladin-surveySelection');

        surveys = surveys.sort(function (a, b) {
            if (!a.order) {
                return a.id > b.id;
            }
            return a.maxOrder && a.maxOrder > b.maxOrder ? 1 : -1;
        });
        surveySelectionDiv.empty();
        const imgLayer = self.aladin.getOverlayImageLayer(self.layer);
        const imgNotLoaded = imgLayer.properties;

        if (imgNotLoaded) {
            let surveyFound = false;
            surveys.forEach(s => {
                const isCurSurvey = imgLayer.properties.url.endsWith(s.url);
                surveySelectionDiv.append($("<option />").attr("selected", isCurSurvey).val(s.id).text(s.name));
                surveyFound |= isCurSurvey;
            });

            // The survey has not been found among the ones cached
            if (!surveyFound) {
                // Cache it
                HpxImageSurvey.SURVEYS.push({
                    id: imgLayer.properties.id,
                    name: imgLayer.properties.name,
                    maxOrder: imgLayer.properties.maxOrder,
                    url: imgLayer.properties.url,
                });
                surveySelectionDiv.append($("<option />").attr("selected", true).val(imgLayer.properties.id).text(imgLayer.properties.name));
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
