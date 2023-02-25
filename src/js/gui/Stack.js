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

import $ from 'jquery';

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

        this.backgroundColor = '#6699ff';
        let self = this;

        this.unselectAllLayers = () => {
            self.aladin.getImageOverlays()
                .forEach((layer) => {
                    let selectedHipsLayer = self.imgLayers.get(layer);

                    let headerSelectedLayerElement = selectedHipsLayer.headerDiv[0];
                    headerSelectedLayerElement.style.backgroundColor = "gainsboro";
                })
        };

        this.selectLayer = (hipsLayer) => {
            // Change the color currently selected layer
            const layer = hipsLayer.layer.layer;

            let headerLayerElement = hipsLayer.headerDiv[0];
            headerLayerElement.style.backgroundColor = "darkgray";

            // Set the active hips layer
            self.aladin.setActiveHiPSLayer(layer);
        };

        this.updateSelectedLayer = () => {
            self.unselectAllLayers();

            const selectedLayer = self.aladin.getActiveHiPSLayer();
            let selectedHipsLayer = self.imgLayers.get(selectedLayer);

            self.selectLayer(selectedHipsLayer);
        }

        this._createComponent();
        this._addListeners();
    }

    _createComponent() {
        let self = this;

        // first, update
        let layerBox = $(this.mainDiv);
        layerBox.empty();

        layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
        '<div class="aladin-box-title">Stack</div>'
        )

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Image layers</div>' +
            '<button class="aladin-btn add-layer-hips" type="button">Add image layer</button>'
        );
        $(this.mainDiv).find('.add-layer-hips').click(function () {
            let layerName = Utils.uuidv4();
            // A HIPS_LAYER_ADDED will be called after the hips is added to the view
            self.aladin.setOverlayImageLayer('CDS/P/DSS2/color', layerName);
        });

        if (this.imgLayers.size > 1) {
            layerBox.append('<div class="aladin-label">Overlay layers</div>')
            Array.from(self.aladin.getImageOverlays()).reverse().forEach((layer) => {
                let imgLayer = self.imgLayers.get(layer);

                if (imgLayer && imgLayer.layer.layer !== "base") {
                    imgLayer.attachTo(layerBox);
                }
            });
        }

        layerBox.append('<div class="aladin-label">Base layer</div>');
        if (this.imgLayers.has("base")) {
            this.imgLayers.get("base").attachTo(layerBox);
        }

        layerBox.append('<div class="aladin-label">Background color</div>');

        let backgroundColorInput = $('<input type="color">');
        layerBox.append(backgroundColorInput);

        // Set a default background color
        backgroundColorInput.val(this.backgroundColor);
        self.aladin.setBackgroundColor(Color.hexToRgb(this.backgroundColor));

        backgroundColorInput.on('input', () => {
            self.backgroundColor = backgroundColorInput.val();
            self.aladin.setBackgroundColor(Color.hexToRgb(self.backgroundColor));
        });

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Overlay layers</div>');

        // loop over all overlay layers
        var layers = this.aladin.getOverlays();
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
            str += '<input type="checkbox" ' + checked + ' id="aladin_lite_' + layer.uuid + '"></input><label for="aladin_lite_' + layer.uuid + '" class="aladin-layer-label" style="background: ' + layer.color + '; color:' + labelColor + ';" title="' + tooltipText + '">' + name + '</label>';
            str += ' <button class="aladin-btn-small aladin-delete-graphic-layer" type="button" title="Delete this layer" data-uuid="' + layer.uuid + '" style="font-size: 10px!important; vertical-align: text-bottom!important; background-color: unset!important;">❌</button>';
            str += '</li>';
        }
        str += '</ul>';

        str += '<button class="aladin-btn my-1 catalogue-selector" type="button">Add catalogue</button>';
        layerBox.append(str);

        layerBox.find('.aladin-delete-graphic-layer').click(function() {
            const layerToDelete = self.aladin.findLayerByUUID($(this).data('uuid'));
            self.aladin.removeLayer(layerToDelete);
        });

        let searchCatalogBtn = layerBox.find('.catalogue-selector');
        searchCatalogBtn.on("click", function () {
            if (!self.catalogSelector) {
                let fnIdSelected = function(type, params) {
                    if (type=='coneSearch') {
                        let catalogLayer = undefined;
                        if (params.baseURL.includes('/vizier.')) {
                            catalogLayer = A.catalogFromVizieR(params.id.replace('CDS/', ''), params.ra + ' ' + params.dec,
                                                               params.radiusDeg, {limit: params.limit, onClick: 'showTable'});
                        }
                        else {
                            let url = params.baseURL;
                            if (! url.endsWith('?')) {
                                url += '?';
                            }
                            url += 'RA=' + params.ra + '&DEC=' + params.dec + '&SR=' + params.radiusDeg;
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
        if (self.aladin.isReticleDisplayed()) {
            checked = 'checked="checked"';
        }
        var reticleCb = $('<input type="checkbox" ' + checked + ' id="displayReticle" />');
        layerBox.append(reticleCb).append('<label for="displayReticle">Reticle</label><br/>');
        reticleCb.change(function () {
            self.aladin.showReticle($(this).is(':checked'));
        });

        // Gestion grille Healpix
        checked = '';
        if (self.aladin.isHpxGridDisplayed()) {
            checked = 'checked="checked"';
        }
        var hpxGridCb = $('<input type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
        layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
        hpxGridCb.change(function () {
            self.aladin.showHealpixGrid($(this).is(':checked'));
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
            //var layer = self.aladin.layerByName(layerName);
            const layer = self.aladin.findLayerByUUID(layerName);

            if ($(this).is(':checked')) {
                layer.show();
            }
            else {
                layer.hide();
            }
        });
    }

    _addListeners() {
        let self = this;

        this.aladin.aladinDiv.addEventListener('remove-layer', e => {
            const layerName = e.detail;
            // Just call remove as it will send a HIPS_LAYER_REMOVED after
            self.aladin.removeImageLayer(layerName);
        });

        this.aladin.aladinDiv.addEventListener('select-layer', (e) => {
            let selectedHipsLayer = e.detail;

            self.unselectAllLayers();
            self.selectLayer(selectedHipsLayer);
        });

        // Events coming from the AL core
        ALEvent.HIPS_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            const layer = e.detail.layer;
            
            const hipsLayer = new HiPSLayer(self.aladin, layer);
            self.imgLayers.set(layer.layer, hipsLayer);

            self._createComponent();

            self.updateSelectedLayer();
        });

        ALEvent.HIPS_LAYER_RENAMED.listenedBy(this.aladin.aladinDiv, function (e) {
            const layer = e.detail.layer;
            const newLayer = e.detail.newLayer;

            const hipsLayer = self.imgLayers.get(layer);
            self.imgLayers.delete(layer);

            self.imgLayers.set(newLayer, new HiPSLayer(self.aladin, hipsLayer.layer));

            self._createComponent();

            self.updateSelectedLayer();
        });

        ALEvent.HIPS_LAYER_SWAP.listenedBy(this.aladin.aladinDiv, function (e) {
            const firstLayer = e.detail.firstLayer;
            const secondLayer = e.detail.secondLayer;

            const firstHiPSLayer = self.imgLayers.get(firstLayer);
            const secondHiPSLayer = self.imgLayers.get(secondLayer);

            self.imgLayers.set(secondLayer, firstHiPSLayer);
            self.imgLayers.set(firstLayer, secondHiPSLayer);

            self._createComponent();

            self.updateSelectedLayer();
        });

        ALEvent.HIPS_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            const layer = e.detail.layer;
            let hipsLayer = self.imgLayers.get(layer);
            // unbind the events
            hipsLayer.destroy();
            self.imgLayers.delete(layer);
    
            self._createComponent();

            self.updateSelectedLayer();
        });

        ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
            self._createComponent();
        });
        ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
            self._createComponent();
        });
    }

    show() {
        this.mainDiv.style.display = 'initial';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }
}
