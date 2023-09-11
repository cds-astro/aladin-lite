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
import A from "../A.js";

import $ from 'jquery';

export class Stack {

    // Constructor
    constructor(parentDiv, aladin, view) {
        this.aladin = aladin;
        this.view = view;

        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'none';
        this.mainDiv.classList.add('aladin-box', 'aladin-layerBox');
        this.backgroundColorInput = $('<input type="color">');

        this.aladinDiv = parentDiv;
        parentDiv.appendChild(this.mainDiv);

        this.imgLayers = new Map();

        this.backgroundColor = this.aladin.getBackgroundColor();
        let self = this;

        this.unselectAllLayers = () => {
            self.aladin.getImageOverlays()
                .forEach((layer) => {
                    let selectedHipsLayer = self.imgLayers.get(layer);

                    let layerElement = selectedHipsLayer.headerDiv[0];
                    layerElement.style.backgroundColor = "#f2f2f2";

                    let headerLayerElement = layerElement.querySelector(".aladin-layer-header")
                    headerLayerElement.style.backgroundColor = "#f2f2f2";
                })
        };

        this.selectLayer = (hipsLayer) => {
            // Change the color currently selected layer
            const layer = hipsLayer.layer.layer;

            let layerElement = hipsLayer.headerDiv[0];
            layerElement.style.backgroundColor = "lightgray";

            let headerLayerElement = layerElement.querySelector(".aladin-layer-header")
            headerLayerElement.style.backgroundColor = "lightgray";

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

    _onAddCatalogue() {
        if (!this.catalogSelector) {
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
                    this.aladin.addCatalog(catalogLayer);
                }
                else if (type=='hips') {
                    const hips = A.catalogHiPS(params.hipsURL, {onClick: 'showTable', name: params.id});
                    this.aladin.addCatalog(hips);
                }
                else if(type=='votable') {
                    let catalogLayer = A.catalogFromURL(params.url, {onClick: 'showTable'});
                    this.aladin.addCatalog(catalogLayer);
                }
            };
             this.catalogSelector = new CatalogSelector(this.aladinDiv, this.aladin, fnIdSelected);
        }
        this.catalogSelector.show();
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
            '<div class="aladin-label">Image layers</div>');


        if (this.imgLayers.size > 1) {
            layerBox.append(
                '<div class="aladin-label" style="font-size: 12px">Overlays</div>'
            );

            Array.from(self.aladin.getImageOverlays()).reverse().forEach((layer) => {
                let imgLayer = self.imgLayers.get(layer);

                if (imgLayer && imgLayer.layer.layer !== "base") {
                    imgLayer.attachTo(layerBox);
                }
            });
        }
        layerBox.append(
            '<div class="aladin-label" style="font-size: 12px">Base</div>'
        );

        if (this.imgLayers.has("base")) {
            this.imgLayers.get("base").attachTo(layerBox);
        }

        layerBox.append(
            '<div class="aladin-horizontal-list">' +
            '<button class="aladin-btn add-layer-hips" type="button" title="Add a full survey (i.e. a HiPS)">Add survey</button>' +
            '<button class="aladin-btn add-layer-image" type="button" title="Add a single image (only FITS file supported)">Open image 📂</button>' +
            '</div>'
        );

        $(this.mainDiv).find('.add-layer-hips').on('click', function () {
            self.aladin.addNewImageLayer();
        });
        $(this.mainDiv).find('.add-layer-image').on('click', function () {
            let input = document.createElement('input');
            input.type = 'file';
            input.onchange = _ => {
                let files = Array.from(input.files);

                files.forEach(file => {
                    const url = URL.createObjectURL(file);
                    const name = file.name;

                    // Consider other cases
                    const image = self.aladin.createImageFITS(
                        url,
                        name,
                        undefined,
                        (ra, dec, fov, _) => {
                            // Center the view around the new fits object
                            self.aladin.gotoRaDec(ra, dec);
                            self.aladin.setFoV(fov * 1.1);
                        },
                        undefined
                    );

                    self.aladin.setOverlayImageLayer(image, name)
                });
            };
            input.click();
        });

        layerBox.append('<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label">Overlay layers</div>');

        // loop over all overlay layers
        var layers = this.aladin.getOverlays();
        var str = '<ul class="aladin-list">';
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
            str += '<li class="aladin-horizontal-list"><div class="aladin-stack-icon" style=\'background-image: url("data:image/svg+xml;base64,' + svgBase64 + '");\'></div>';
            str += '<input class="aladin-input" type="checkbox" ' + checked + ' id="aladin_lite_' + layer.uuid + '"></input><label for="aladin_lite_' + layer.uuid + '" class="aladin-layer-label" style="background: ' + layer.color + '; color:' + labelColor + ';" title="' + tooltipText + '">' + name + '</label>';
            str += ' <button class="aladin-btn aladin-24px-icon aladin-delete-graphic-layer" style="background-color: #eaeaea" type="button" title="Delete this layer">❌</button>';
            str += '</li>';
        }
        str += '</ul>';

        str += '<button class="aladin-btn catalogue-selector" type="button">Add catalogue</button>';
        layerBox.append(str);

        layerBox.find('.aladin-delete-graphic-layer').on('click', () => {
            const layerToDelete = this.aladin.findLayerByUUID(layer.uuid);
            this.aladin.removeLayer(layerToDelete);
        });

        let addCatalogBtn = layerBox.find('.catalogue-selector');
        addCatalogBtn.on("click", () => self._onAddCatalogue());

        layerBox.append('<div class="aladin-blank-separator"></div>');

        // gestion du réticule
        var checked = '';
        if (self.aladin.isReticleDisplayed()) {
            checked = 'checked="checked"';
        }
        var reticleCb = $('<input class="aladin-input" type="checkbox" ' + checked + ' id="displayReticle" />');
        layerBox.append(reticleCb).append('<label for="displayReticle">Reticle</label><br/>');
        reticleCb.change(function () {
            self.aladin.showReticle($(this).is(':checked'));
        });

        // Gestion grille Healpix
        checked = '';
        if (self.aladin.isHpxGridDisplayed()) {
            checked = 'checked="checked"';
        }
        var hpxGridCb = $('<input class="aladin-input" type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
        layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
        hpxGridCb.change(function () {
            self.aladin.showHealpixGrid($(this).is(':checked'));
        });

        layerBox.append('<div class="aladin-box-separator"></div>' +
        '<div class="aladin-label">Background color</div>');

        layerBox.append(this.backgroundColorInput);

        this.backgroundColorInput.on('input', () => {
            self.backgroundColor = this.backgroundColorInput.val();
            self.aladin.setBackgroundColor(Color.hexToRgb(self.backgroundColor));
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
        ALEvent.BACKGROUND_COLOR_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
            const color = e.detail.color;

            let inputColor = self.mainDiv.querySelector('input[type="color"]');
            let hexColor = Color.rgbToHex(color.r, color.g, color.b);
            inputColor.value = hexColor;

            self.backgroundColor = color;
        });

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

            if(hipsLayer.children) {
                hipsLayer.children.forEach((child) => {
                // unbind the events
                    child.destroy();
                    self.imgLayers.delete(child.layer);
                });
            } else {
                // unbind the events
                hipsLayer.destroy();
                self.imgLayers.delete(layer);
            }

            self._createComponent();

            if (self.imgLayers.length > 0) {
                self.updateSelectedLayer();
            }
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
