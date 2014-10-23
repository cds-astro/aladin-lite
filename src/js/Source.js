/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Source
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

cds.Source = (function() {
    // constructor
    cds.Source = function(ra, dec, data, options) {
    	this.ra = ra;
    	this.dec = dec;
    	this.data = data;
    	this.catalog = null;
    	
        this.marker = (options && options.marker) || false;
        if (this.marker) {
            this.popupTitle = (options && options.popupTitle) ? options.popupTitle : '';
            this.popupDesc = (options && options.popupDesc) ? options.popupDesc : '';
        }
    	this.isShowing = true;
    	this.isSelected = false;
    };
    
    cds.Source.prototype.setCatalog = function(catalog) {
        this.catalog = catalog;
    };
    
    cds.Source.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    return cds.Source;
})();
