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
 
 import $ from 'jquery';
 
 export class CooGrid {
 
    // Constructor
    constructor(parentDiv, aladin, view) {
        this.aladin = aladin;
        this.view = view;
        this.isChecked = false;

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
        let labelCoordinatesGridCb = $('<div class="aladin-label">Coo grid options</div>');
        let cooGridOptions = $('<div class="layer-options"><table><tbody><tr><td>Color</td><td><input type="color" value="#00ff00"></td></tr><tr><td>Opacity</td><td><input class="aladin-input opacity" value="0.5" type="range" min="0.0" max="1.0" step="0.05"></td></tr><tr><td>Thickness</td><td><input class="aladin-input thickness" value="3.0" type="range" min="0.5" max="10.0" step="0.01"></td></tr><tr><td>Label size</td><td><input class="aladin-input label-size" type="range" value="30" min="10" max="60" step="0.01"></td></tr></table></div>');
        layerBox.append(labelCoordinatesGridCb).append(cooGridOptions);

        let gridColorInput = cooGridOptions.find('input[type="color"]');
        let gridOpacityInput = cooGridOptions.find('.opacity');
        let gridThicknessInput = cooGridOptions.find('.thickness');
        gridThicknessInput.on('input', () => {
            const thickness = +gridThicknessInput.val();
            self.view.setGridConfig({
                thickness: thickness
            });
        });

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
            self.isChecked = !self.isChecked;
        });

        ALEvent.COO_GRID_DISABLED.listenedBy(self.aladinDiv, function () {
            self.isChecked = !self.isChecked;
        });

        ALEvent.COO_GRID_UPDATED.listenedBy(self.aladinDiv, function (e) {
            let opacity = e.detail.opacity;

            if (gridOpacityInput.val() != opacity) {
                gridOpacityInput.val(opacity);
            }

            let color = e.detail.color;
            let hexColor = Color.rgbToHex(Math.round(255 * color.r), Math.round(255 * color.g), Math.round(255 * color.b));
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
 