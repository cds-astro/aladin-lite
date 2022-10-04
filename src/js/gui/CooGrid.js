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

 import { Color } from "../Color.js";
 import { ALEvent } from "../events/ALEvent.js";
 
 export class CooGrid {
 
    // Constructor
    constructor(parentDiv, aladin, view) {
        this.aladin = aladin;
        this.view = view;

        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'none';
        this.mainDiv.classList.add('aladin-box', 'aladin-layerBox', 'aladin-cb-list');

        this.aladinDiv = parentDiv;
        parentDiv.appendChild(this.mainDiv);

        this._createComponent();
        this._addListeners();
    }
 
    _createComponent() {
        let self = this;

        // first, update
        let layerBox = $(this.mainDiv);
        layerBox.empty();

        layerBox.append(
            '<a class="aladin-closeBtn">&times;</a>'
        )

        // Coordinates grid plot
        var checked = '';
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
                color: { r: rgb.r / 255.0, g: rgb.g / 255.0, b: rgb.b / 255.0 },
                opacity: parseFloat(opacity)
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
        ALEvent.COO_GRID_ENABLED.listenedBy(self.aladinDiv, function () {
            if (!coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', true);
            }
        });
        ALEvent.COO_GRID_DISABLED.listenedBy(self.aladinDiv, function () {
            if (coordinatesGridCb.prop('checked')) {
                coordinatesGridCb.prop('checked', false);
            }
        });
        ALEvent.COO_GRID_UPDATED.listenedBy(self.aladinDiv, function (e) {
            let c = e.detail.color;
            let opacity = e.detail.opacity;

            if (gridOpacityInput.val() != opacity) {
                gridOpacityInput.val(opacity);
            }

            let hexColor = Color.rgbToHex(Math.round(255 * c[0]), Math.round(255 * c[1]), Math.round(255 * c[2]));
            if (gridColorInput.val() != hexColor) {
                gridColorInput.val(hexColor);
            }
        });

        layerBox.find('.aladin-closeBtn').click(function () { self.aladin.hideBoxes(); return false; });
    }

    _addListeners() {
    }

    show() {
        this.mainDiv.style.display = 'block';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }
 }
 