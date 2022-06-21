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

import { AladinUtils } from "../AladinUtils.js";
import { Color } from "../Color.js";
import { ALEvent } from "../events/ALEvent.js";
import { CatalogSelector } from "./CatalogSelector.js";
import { HiPSLayer } from "./HiPSLayer.js";
import { Utils } from "../Utils.js";

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
        this.selectedLayer = undefined;

        this.#createComponent();
        this.#addListeners();
    }

    #createComponent() {
        let self = this;

        // first, update
        let layerBox = $(this.mainDiv);
        layerBox.empty();

        layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
        '<div class="aladin-box-title">Layer manager</div>'
        )

        if (this.imgLayers.size > 1) {
            layerBox.append('<div class="aladin-label">Overlay image layers</div>')

            Array.from(this.imgLayers.values()).reverse().forEach((imgLayer) => {
                if (imgLayer.survey.layer !== "base") {
                    imgLayer.attachTo(layerBox);
                }
            });
        }

        layerBox.append('<div class="aladin-label">Base image layer</div>');
        if (this.imgLayers.has("base")) {
            this.imgLayers.get("base").attachTo(layerBox);
        }

        layerBox.append(
            '<button class="aladin-btn add-layer-hips" type="button">Add image layer</button>'
        );

        $(this.mainDiv).find('.add-layer-hips').click(function () {
            const layerName = Utils.uuidv4();

            // A HIPS_LAYER_ADDED will be called after the hips is added to the view
            self.aladin.setOverlayImageLayer(
                'CDS/P/DSS2/color',
                null,
                layerName
            );
        });

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
            str += '<input type="checkbox" ' + checked + ' id="aladin_lite_' + name + '"></input><label for="aladin_lite_' + name + '" class="aladin-layer-label" style="background: ' + layer.color + '; color:' + labelColor + ';" title="' + tooltipText + '">' + name + '</label>';
            str += ' <button class="aladin-btn-small aladin-delete-layer" type="button" title="Delete this layer" data-uuid="' + layer.uuid + '" style="font-size: 10px!important; vertical-align: unset!important;">❌</button>';
            str += '</li>';
        }
        str += '</ul>';

        str += '<button class="aladin-btn my-1 catalogue-selector" type="button">Add catalogue</button>';
        layerBox.append(str);

        let searchCatalogBtn = layerBox.find('.catalogue-selector');
        searchCatalogBtn.click(function () {
            if (!self.catalogSelector) {
                let fnIdSelected = function(type, params) {
                    if (type=='coneSearch') {
                        let catalogLayer = undefined;
                        if (params.baseURL.includes('/vizier.')) {
                            catalogLayer = A.catalogFromVizieR(params.id.replace('CDS/', ''), params.ra + ' ' + params.dec,
                                                               params.radiusDeg, {limit: params.limit, onClick: 'showTable'});
                        }
                        else {
                            const url = params.baseURL + 'RA=' + params.ra + '&DEC=' + params.dec + '&SR=' + params.radiusDeg;
                            catalogLayer = A.catalogFromURL(url, {limit: params.limit, onClick: 'showTable'});
                        }
                        self.aladin.addCatalog(catalogLayer);
                    }
                    else if (type=='hips') {
                        const hips = A.catalogHiPS(params.hipsURL, {onClick: 'showTable', name: params.id});
                        self.aladin.addCatalog(hips);
                    }
                    else if(type=='votable') {
                        let catalogLayer = A.catalogFromURL(params.url, {onClick: 'showTable'});
                        console.log(catalogLayer)
                        self.aladin.addCatalog(catalogLayer);
                    }
                };

                self.catalogSelector = new CatalogSelector(self.aladinDiv, self.aladin, fnIdSelected);
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
        let cooGridOptions = $('<div class="layer-options" style="display: none;"><table><tbody><tr><td>Color</td><td><input type="color" value="#00ff00"></td></tr><tr><td>Opacity</td><td><input class="opacity" value="1.0" type="range" min="0" max="1" step="0.05"></td></tr><tr><td>Label size</td><td><input class="label-size" type="range" min="5" max="30" step="0.01"></td></tr></table></div>');
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
        this.aladin.aladinDiv.addEventListener('remove-layer', e => {
            const layerName = e.detail;
            // Just call remove as it will send a HIPS_LAYER_REMOVED after
            self.aladin.removeImageSurvey(layerName);

            if (self.selectedLayer === layerName) {
                self.selectedLayer = null;
            }
        });

        this.aladin.aladinDiv.addEventListener('select-layer', e => {
            const layerName = e.detail;
            
            // Update the color of the selected element
            if (self.selectedLayer) {
                const headerClassName = "aladin-layer-header-" + self.selectedLayer;
                let headerLayerElement = document.getElementsByClassName(headerClassName)[0];
                headerLayerElement.style.backgroundColor = "#eee";
            }

            const headerClassName = "aladin-layer-header-" + layerName;
            let headerLayerElement = document.getElementsByClassName(headerClassName)[0];
            headerLayerElement.style.backgroundColor = "#026baa";

            self.aladin.view.setActiveHiPSLayer(layerName);

            self.selectedLayer = layerName;
        });

        // Events coming from the AL core
        ALEvent.HIPS_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            const survey = e.detail.survey;
            self.imgLayers.set(survey.layer, new HiPSLayer(self.aladin, self.view, survey));

            self.#createComponent();
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            const layer = e.detail.layer;
            console.log("remove listener on:", layer)
            let hipsLayer = self.imgLayers.get(layer);
            // unbind the events
            hipsLayer.destroy();
            self.imgLayers.delete(layer);
    
            self.#createComponent();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            self.#createComponent();
        });
    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }
}
