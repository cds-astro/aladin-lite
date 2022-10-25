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
 * File gui/ProjectionSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 import { ALEvent } from "../events/ALEvent.js";
 
 export class ProjectionSelector {
 
     // Constructor
     constructor(parentDiv, aladin) {
         this.aladin = aladin;
 
         this.mainDiv = document.createElement('div');
         this.mainDiv.classList.add('aladin-projSelection');
 
         parentDiv.appendChild(this.mainDiv);
         
         this._createComponent();
         this._addListeners();
     }
 
     _createComponent() {
        $(this.mainDiv).append('<select title="Projection"></select>');

        this.selectProjection = $(this.mainDiv).find('select');

        this.selectProjection.empty();
        
        ["SIN", "AIT", "MOL", "MER", "ARC", "TAN", "HPX"].forEach(p => {
            this.selectProjection.append($("<option />").val(p).text(p));
        });
        let self = this;
        this.selectProjection.change(function () {
            self.aladin.setProjection($(this).val());
        });
     }
 
     _addListeners() {
         const self = this;
         ALEvent.PROJECTION_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
            self.selectProjection.val(e.detail.projection);
         });
 
     }
 
     show() {
         this.mainDiv.style.display = 'block';
     }
 
     hide() {
         this.mainDiv.style.display = 'none';
     }
 }
 
