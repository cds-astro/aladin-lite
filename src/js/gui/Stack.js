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
        this.view   = view;


        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'none';
        this.mainDiv.classList.add('aladin-box', 'aladin-layerBox', 'aladin-cb-list');

        parentDiv.appendChild(this.mainDiv);
        this.aladinDiv = parentDiv;
    };

    // TODO: do not recreate all DOM objects at each show() call
    show() {
        let self = this;

        // first, update
            let layerBox = $(this.mainDiv);

            layerBox.empty();
            layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
            '<div style="clear: both;"></div>' +
            '<div class="aladin-label">Base image layer</div>' +
            '<select class="aladin-surveySelection"></select>' +
            '<br>' +
            '<button class="aladin-btn" type="button">Search HiPS</button>' +
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
            searchHiPS4BaseLayerBtn.click(function() {
                if (! self.hipsSelector) {
                    self.hipsSelector = new HiPSSelector(self.aladinDiv);
                }
                console.log('SHOW');
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
        let cooGridOptions = $('<div class="layer-options" style="display: none;"><table><tbody><tr><td>Color</td><td><input type="color"></td></tr><tr><td>Opacity</td><td><input type="range" min="0" max="1" step="0.05"></td></tr></table></div>');
        labelCoordinatesGridCb.prepend(coordinatesGridCb);
        layerBox.append(optionsOpenerForCoordinatesGrid).append(labelCoordinatesGridCb).append(cooGridOptions);
        coordinatesGridCb.change(function () {
            console.log('cb change');
            let isChecked = $(this).is(':checked');
            if (isChecked) {
                self.view.setGridConfig({
                    enabled: true,
                    color: [0.0, 1.0, 0.0, 0.5],
                });
            } else {
                self.view.setGridConfig({
                    enabled: false,
                });
            }
        });
        optionsOpenerForCoordinatesGrid.click(function() {
            var $this = $(this);
            if ($this.hasClass('right-triangle')) {
                $this.removeClass('right-triangle');
                $this.addClass('down-triangle');
                $this.parent().find('.layer-options').slideDown(300);
            }
            else {
                $this.removeClass('down-triangle');
                $this.addClass('right-triangle');
                $this.parent().find('.layer-options').slideUp(300);
            }
        });

        let gridColorInput = cooGridOptions.find('input[type="color"]');
        let gridOpacityInput = cooGridOptions.find('input[type="range"]');
        let updateGridcolor = function() {
            let rgb = Color.hexToRgb(gridColorInput.val());
            let opacity = gridOpacityInput.val();
            self.view.setGridConfig({
                enabled: coordinatesGridCb.is(':checked'),
                color: [rgb.r / 255.0, rgb.g / 255.0, rgb.b / 255.0, parseFloat(gridOpacityInput.val())]
            });
        };
        gridColorInput.on('input', updateGridcolor);
        gridOpacityInput.on('input', updateGridcolor);
        // coordinates grid - add event listeners
        ALEvent.COO_GRID_ENABLED.listenedBy(this.aladinDiv, function() {
            if (! coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', true);
            }
        });
        ALEvent.COO_GRID_DISABLED.listenedBy(this.aladinDiv, function() {
            if (coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', false);
            }
        });
        ALEvent.COO_GRID_UPDATED.listenedBy(this.aladinDiv, function(e) {
            let c = e.detail.color;
            let opacity = c[3].toFixed(2);
            if (gridOpacityInput.val() != opacity) {
                gridOpacityInput.val(opacity);
            }

            let hexColor = Color.rgbToHex(Math.round(255*c[0]), Math.round(255*c[1]), Math.round(255*c[2]));
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

        // finally show
        this.mainDiv.style.display = 'block';
        }


    hide() {
        this.mainDiv.style.display = 'none';
    }
}