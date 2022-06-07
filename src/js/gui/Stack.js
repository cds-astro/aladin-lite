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
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

import { HpxImageSurvey } from "../HpxImageSurvey.js";
import { AladinUtils } from "../AladinUtils.js";
import { Color } from "../Color.js";
import { ALEvent } from "../events/ALEvent.js";
import { HiPSSelector } from "./HiPSSelector.js";

export class Stack {

    // Constructor
    constructor(parentDiv, aladin, view) {
        this.aladin = aladin;
        this.view = view;


        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'none';
        this.mainDiv.classList.add('aladin-box', 'aladin-layerBox', 'aladin-cb-list');

        parentDiv.appendChild(this.mainDiv);
        this.aladinDiv = parentDiv;

        this.#createComponent();
        this.#addListeners();
    }

    #createComponent() {
        let self = this;

        // first, update
        let layerBox = $(this.mainDiv);

        layerBox.empty();
        let optionsOpenerForBaseImageLayer = $('<span class="indicator right-triangle">&nbsp;</span>');
        let cmListStr = '';
        for (const cm of aladin.webglAPI.getAvailableColormapList()) {
            cmListStr += '<option>' + cm + '</option>';
        }
        // Add the native which is special:
        // - for FITS hipses, it is changed to grayscale
        // - for JPG/PNG hipses, we do not use any colormap in the backend
        cmListStr += '<option>native</option>';

        this.baseImageLayerOptions = $('<div class="layer-options" style="display: none;">' +
                                        '<table class="aladin-options"><tbody>' +
                                        '  <tr><td>Color map</td><td><select class="">' + cmListStr + '</select></td></tr>' +
                                        '  <tr><td></td><td><label><input type="checkbox"> Reverse</label></td></tr>' +
                                        '  <tr><td>Stretch</td><td><select class=""><option>Pow2</option><option selected>Linear</option><option>Sqrt</option><option>Asinh</option><option>Log</option></select></td></tr>' +
                                        '</table> ' +
                                      '</div>');

        let colorMapSelect4BaseImgLayer = this.baseImageLayerOptions.find('select').eq(0);
        colorMapSelect4BaseImgLayer.val('grayscale');
        let stretchSelect4BaseImgLayer = this.baseImageLayerOptions.find('select').eq(1);

        let reverseCmCb = this.baseImageLayerOptions.find('input').eq(0);
        colorMapSelect4BaseImgLayer.add(reverseCmCb).add(stretchSelect4BaseImgLayer).change(function () {
            const reverse = reverseCmCb[0].checked;
            const cmap = colorMapSelect4BaseImgLayer.val();
            const stretch = stretchSelect4BaseImgLayer.val();

            aladin.getBaseImageLayer().setColormap(cmap, {reversed: reverse, stretch: stretch});
            // update HpxImageSurvey.SURVEYS definition
            const idxSelectedBaseHiPS = self.mainDiv.querySelector('.aladin-surveySelection').selectedIndex;
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedBaseHiPS];
            let options = surveyDef.options || {};
            options.colormap = cmap;
            options.stretch = stretch;
            surveyDef.options = options;
        });

        optionsOpenerForBaseImageLayer.click(function () {
            var $this = $(this);
            if ($this.hasClass('right-triangle')) {
                $this.removeClass('right-triangle');
                $this.addClass('down-triangle');
                self.baseImageLayerOptions.slideDown(300);
            }
            else {
                $this.removeClass('down-triangle');
                $this.addClass('right-triangle');
                self.baseImageLayerOptions.slideUp(300);
            }
        });

        layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
                        '<div style="clear: both;"></div>' +
                        '<div class="aladin-label">Base image layer</div>')
                .append(optionsOpenerForBaseImageLayer) 
                .append('<select class="aladin-surveySelection"></select>')
                .append(this.baseImageLayerOptions)
                .append('<br>' +
                        '<button class="aladin-btn my-1" type="button">Search HiPS</button>' +
                        '<div class="aladin-label">Projection</div>' +
                        '<select class="aladin-projSelection"></select>' +
                        '</div>');

        this.aladin.updateProjectionCombobox(this.aladin.projection);
        var projectionSelection = $(this.aladin.aladinDiv).find('.aladin-projSelection');
        projectionSelection.change(function () {
            self.aladin.projection = $(this).val();
            self.aladin.setProjection(self.aladin.projection);
        });

        layerBox.append(projectionSelection)
            .append('<br />');

        let searchHiPS4BaseLayerBtn = layerBox.find('button');
        searchHiPS4BaseLayerBtn.click(function () {
            if (!self.hipsSelector) {
                let fnURLSelected = function(url) {
                    aladin.setBaseImageLayer(url);
                };
                let fnIdSelected = function(id) {
                    aladin.setBaseImageLayer(id);
                };
                self.hipsSelector = new HiPSSelector(self.aladinDiv, fnURLSelected, fnIdSelected);
            }
            self.hipsSelector.show();
        });

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Overlay layers</div>');

        //var cmDiv = layerBox.find('.aladin-cmap');

        // fill color maps options
        /*var cmSelect = layerBox.find('.aladin-cmSelection');
        for (var k = 0; k < ColorMap.MAPS_NAMES.length; k++) {
            cmSelect.append($("<option />").text(ColorMap.MAPS_NAMES[k]));
        }
        console.log(self.getBaseImageLayer())
        console.log(self.getBaseImageLayer().getColorMap())
        cmSelect.val(self.getBaseImageLayer().getColorMap().mapName);*/


        // loop over all overlay layers
        var layers = this.view.allOverlayLayers;
        var str = '<ul>';
        for (var k = layers.length - 1; k >= 0; k--) {
            var layer = layers[k];
            var name = layer.name;
            var checked = '';
            if (layer.isShowing) {
                checked = 'checked="checked"';
            }

            var tooltipText = '';
            var iconSvg = '';
            if (layer.type == 'catalog' || layer.type == 'progressivecat') {
                var nbSources = layer.getSources().length;
                tooltipText = nbSources + ' source' + (nbSources > 1 ? 's' : '');

                iconSvg = AladinUtils.SVG_ICONS.CATALOG;
            }
            else if (layer.type == 'moc') {
                tooltipText = 'Coverage: ' + (100 * layer.skyFraction()).toFixed(3) + ' % of sky';

                iconSvg = AladinUtils.SVG_ICONS.MOC;
            }
            else if (layer.type == 'overlay') {
                iconSvg = AladinUtils.SVG_ICONS.OVERLAY;
            }

            var rgbColor = $('<div></div>').css('color', layer.color).css('color'); // trick to retrieve the color as 'rgb(,,)' - does not work for named colors :(
            var labelColor = Color.getLabelColorForBackground(rgbColor);

            // retrieve SVG icon, and apply the layer color
            var svgBase64 = window.btoa(iconSvg.replace(/FILLCOLOR/g, layer.color));
            str += '<li><div class="aladin-stack-icon" style=\'background-image: url("data:image/svg+xml;base64,' + svgBase64 + '");\'></div>';
            str += '<input type="checkbox" ' + checked + ' id="aladin_lite_' + name + '"></input><label for="aladin_lite_' + name + '" class="aladin-layer-label" style="background: ' + layer.color + '; color:' + labelColor + ';" title="' + tooltipText + '">' + name + '</label></li>';
        }
        str += '</ul>';
        layerBox.append(str);

        layerBox.append('<div class="aladin-blank-separator"></div>');

        // gestion du réticule
        var checked = '';
        if (this.view.displayReticle) {
            checked = 'checked="checked"';
        }
        var reticleCb = $('<input type="checkbox" ' + checked + ' id="displayReticle" />');
        layerBox.append(reticleCb).append('<label for="displayReticle">Reticle</label><br/>');
        reticleCb.change(function () {
            self.aladin.showReticle($(this).is(':checked'));
        });

        // Gestion grille Healpix
        checked = '';
        if (this.view.displayHpxGrid) {
            checked = 'checked="checked"';
        }
        var hpxGridCb = $('<input type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
        layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
        hpxGridCb.change(function () {
            self.aladin.showHealpixGrid($(this).is(':checked'));
        });

        // Coordinates grid plot
        checked = '';
        if (this.view.showCooGrid) {
            checked = 'checked="checked"';
        }
        let optionsOpenerForCoordinatesGrid = $('<span class="indicator right-triangle"> </span>');
        let coordinatesGridCb = $('<input type="checkbox" ' + checked + ' id="displayCoordinatesGrid"/>');
        let labelCoordinatesGridCb = $('<label>Coordinates grid</label>');
        let cooGridOptions = $('<div class="layer-options" style="display: none;"><table><tbody><tr><td>Color</td><td><input type="color" value="#00ff00"></td></tr><tr><td>Opacity</td><td><input class="opacity" type="range" min="0" max="1" step="0.05"></td></tr><tr><td>Label size</td><td><input class="label-size" type="range" min="5" max="30" step="0.01"></td></tr></table></div>');
        labelCoordinatesGridCb.prepend(coordinatesGridCb);
        layerBox.append(optionsOpenerForCoordinatesGrid).append(labelCoordinatesGridCb).append(cooGridOptions);
        coordinatesGridCb.change(function () {
            let isChecked = $(this).is(':checked');
            if (isChecked) {
                console.log("ENABLE grid")
                self.view.setGridConfig({
                    enabled: true,
                });
            } else {
                self.view.setGridConfig({
                    enabled: false,
                });
            }
        });
        optionsOpenerForCoordinatesGrid.click(function () {
            var $this = $(this);
            if ($this.hasClass('right-triangle')) {
                $this.removeClass('right-triangle');
                $this.addClass('down-triangle');
                cooGridOptions.slideDown(300);
            }
            else {
                $this.removeClass('down-triangle');
                $this.addClass('right-triangle');
                cooGridOptions.slideUp(300);
            }
        });

        let gridColorInput = cooGridOptions.find('input[type="color"]');
        let gridOpacityInput = cooGridOptions.find('.opacity');
        let updateGridcolor = function () {
            let rgb = Color.hexToRgb(gridColorInput.val());
            let opacity = gridOpacityInput.val();
            self.view.setGridConfig({
                color: [rgb.r / 255.0, rgb.g / 255.0, rgb.b / 255.0, parseFloat(gridOpacityInput.val())]
            });
        };
        gridColorInput.on('input', updateGridcolor);
        gridOpacityInput.on('input', updateGridcolor);
        let gridLabelSizeInput = cooGridOptions.find('.label-size');
        gridLabelSizeInput.on('input', function () {
            const size = +gridLabelSizeInput.val();
            self.view.setGridConfig({
                labelSize: size
            });
        });

        // coordinates grid - add event listeners
        ALEvent.COO_GRID_ENABLED.listenedBy(this.aladinDiv, function () {
            if (!coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', true);
            }
        });
        ALEvent.COO_GRID_DISABLED.listenedBy(this.aladinDiv, function () {
            if (coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', false);
            }
        });
        ALEvent.COO_GRID_UPDATED.listenedBy(this.aladinDiv, function (e) {
            let c = e.detail.color;
            let opacity = c[3].toFixed(2);
            if (gridOpacityInput.val() != opacity) {
                gridOpacityInput.val(opacity);
            }

            let hexColor = Color.rgbToHex(Math.round(255 * c[0]), Math.round(255 * c[1]), Math.round(255 * c[2]));
            if (gridColorInput.val() != hexColor) {
                gridColorInput.val(hexColor);
            }
        });

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Tools</div>');
        var exportBtn = $('<button class="aladin-btn" type="button">Export view as PNG</button>');
        layerBox.append(exportBtn);
        exportBtn.click(function () {
            self.aladin.exportAsPNG();
        });

        layerBox.find('.aladin-closeBtn').click(function () { self.aladin.hideBoxes(); return false; });

        // update list of surveys
        this.aladin.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        var surveySelection = $(this.mainDiv).find('.aladin-surveySelection');
        surveySelection.change(function () {
            var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
            console.log("survey, chosen ", survey)

            const hpxImageSurvey = new HpxImageSurvey(
                survey.url,
                self.view,
                survey.options
            );
            self.aladin.setImageSurvey(hpxImageSurvey, function () {
                var baseImgLayer = self.aladin.getBaseImageLayer();

                // !TODO
                /*
                if (baseImgLayer.useCors) {
                    // update color map list with current value color map
                    cmSelect.val(baseImgLayer.getColorMap().mapName);
                    cmDiv.show();

                    exportBtn.show();
                }
                else {
                    cmDiv.hide();

                    exportBtn.hide();
                }*/
            });
        });

        //// COLOR MAP management ////////////////////////////////////////////
        // update color map
        /*cmDiv.find('.aladin-cmSelection').change(function () {
            var cmName = $(this).find(':selected').val();
            self.getBaseImageLayer().getColorMap().update(cmName);
        });

        // reverse color map
        cmDiv.find('.aladin-reverseCm').click(function () {
            self.getBaseImageLayer().getColorMap().reverse();
        });
        if (this.getBaseImageLayer().useCors) {
            cmDiv.show();
            exportBtn.show();
        }
        else {
            cmDiv.hide();
            exportBtn.hide();
        }
        layerBox.find('.aladin-reverseCm').parent().attr('disabled', true);
        */
        //////////////////////////////////////////////////////////////////////


        // handler to hide/show overlays
        $(this.mainDiv).find('ul input').change(function () {
            var layerName = ($(this).attr('id').substr(12));
            var layer = self.aladin.layerByName(layerName);
            if ($(this).is(':checked')) {
                layer.show();
            }
            else {
                layer.hide();
            }
        });
    }

    #addListeners() {
        const self = this;
        ALEvent.BASE_HIPS_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, function () {
            self.#updateBaseHiPSLayerOptions();
            console.log('hello');
        });

    }

    #updateBaseHiPSLayerOptions() {
        const reverseCmCb                 = this.baseImageLayerOptions.find('input').eq(0);
        const colorMapSelect4BaseImgLayer = this.baseImageLayerOptions.find('select').eq(0);
        const stretchSelect4BaseImgLayer  = this.baseImageLayerOptions.find('select').eq(1);

        const meta = this.aladin.getBaseImageLayer().meta;
        console.log('cmap',  meta);
        const cmap = meta.color.grayscale.color.colormap.name;
        const reverse = meta.color.grayscale.color.colormap.reversed;
        const stretch = meta.color.grayscale.stretch;

        console.log('reverse', reverse);
        console.log('stretch', stretch);
        reverseCmCb.prop('checked', reverse);
        colorMapSelect4BaseImgLayer.val(cmap);
        stretchSelect4BaseImgLayer.val(stretch);

    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }
}
