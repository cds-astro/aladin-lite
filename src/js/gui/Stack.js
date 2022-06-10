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
import { CatalogSelector } from "./CatalogSelector.js";
import { HiPSLayer } from "./HiPSLayer.js";

export class Stack {

    // Constructor
    constructor(parentDiv, aladin, view) {
        this.aladin = aladin;
        this.view = view;

        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'none';
        this.mainDiv.classList.add('aladin-box', 'aladin-layerBox', 'aladin-cb-list');

        this.aladinDiv = parentDiv;
        parentDiv.appendChild(this.mainDiv);

        this.imgLayers = new Map();
        this.imgLayers.set("base", new HiPSLayer(this.aladin, this.view, "base"));

        this.#createComponent();
        this.#addListeners();
    }

    #createComponent() {
        let self = this;

        // first, update
        let layerBox = $(this.mainDiv);
        layerBox.empty();

        layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
            '<div style="clear: both;"></div>' +
            '<div class="aladin-label">Base image layer</div>')

        this.imgLayers.forEach((imgLayer) => {
            imgLayer.attachTo(layerBox);
        });


        layerBox.append('<div class="aladin-label">Add Layer</div>' +
            '<input type="text" class="layer-name" maxlength="16" size="16">' +
            '<button class="aladin-btn" type="button">+</button>'
        );

        layerBox.append('<br>');

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Overlay layers</div>');

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

        str += '<button class="aladin-btn my-1" type="button">Add catalogue</button>';
        layerBox.append(str);

        let searchCatalogBtn = layerBox.find('button').eq(1);
        searchCatalogBtn.click(function () {
            if (!self.catalogSelector) {
                let fnURLSelected = function(url) {
                    alert(url);
                };
                let fnIdSelected = function(id, item, params) {
                    alert(id);
                    console.log(item);
                };
                self.catalogSelector = new CatalogSelector(self.aladinDiv, fnURLSelected, fnIdSelected);
            }
            self.catalogSelector.show();
        });

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
        //this.aladin.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        /*var surveySelection = $(this.mainDiv).find('.aladin-surveySelection');
        surveySelection.change(function () {
            var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
            //console.log("survey, chosen ", survey)
            const hpxImageSurvey = new HpxImageSurvey(
                survey.url,
                self.view,
                survey.options
            );
            self.aladin.setImageSurvey(hpxImageSurvey);
        });*/

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
        let self = this;
        const layerBox = $(this.mainDiv);

        layerBox.find('button').click(function () {
            const layerName = $(layerBox.find('.layer-name')[0]).val();

            /*if (self.imgLayers.has(layerName)) {
                throw 'Layer ' + layerName + ' already exist.';
            }*/

            self.aladin.setOverlayImageLayer(
                'CDS/P/DSS2/color',
                (survey) => {
                    console.log("loaded")
                    self.imgLayers.set(layerName, new HiPSLayer(self.aladin, self.view, layerName));
                    survey.setOpacity(0.5)

                    self.#createComponent();
                },
                layerName
            );
        });
    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }
}
