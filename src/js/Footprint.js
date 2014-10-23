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
    
    Footprint.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
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
