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
 * File Footprint
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Footprint = (function() {
    // constructor
    Footprint = function(polygons) {
        this.polygons = polygons;
    	this.overlay = null;

        // TODO : all graphic overlays should have an id
        this.id = 'footprint-' + Utils.uuidv4();
    	
    	this.isShowing = true;
    	this.isSelected = false;
    };
    
    Footprint.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };
    
    Footprint.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Footprint.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Footprint.prototype.dispatchClickEvent = function() {
        if (this.overlay) {
            // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
            //window.dispatchEvent(new CustomEvent("footprintClicked", {
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
        }
    };
    
    Footprint.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
/*
            // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
            //window.dispatchEvent(new CustomEvent("footprintClicked", {
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
*/

            this.overlay.reportChange();
        }
    };

    Footprint.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    return Footprint;
})();
