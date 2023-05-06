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
 * File Source
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

export let Source = (function() {
    // constructor
    let Source = function(ra, dec, data, options) {
    	this.ra = ra;
    	this.dec = dec;
    	this.data = data;

    	this.catalog = null;

        this.marker = (options && options.marker) || false;
        if (this.marker) {
            this.popupTitle = (options && options.popupTitle) ? options.popupTitle : '';
            this.popupDesc = (options && options.popupDesc) ? options.popupDesc : '';
            this.useMarkerDefaultIcon = (options && options.useMarkerDefaultIcon!==undefined) ? options.useMarkerDefaultIcon : true;
        }

    	this.isShowing = true;
    	this.isSelected = false;
    };

    Source.prototype.setCatalog = function(catalog) {
        this.catalog = catalog;
    };

    Source.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    Source.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    Source.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    Source.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    // function called when a source is clicked. Called by the View object
    Source.prototype.actionClicked = function() {
        if (this.catalog && this.catalog.onClick) {
            var view = this.catalog.view;

            if (this.catalog.onClick=='showTable') {
                this.select();

                let singleSourceTable = {
                    'rows': [this],
                    'fields': this.catalog.fields,
                    'fieldsClickedActions': this.catalog.fieldsClickedActions,
                    'name': this.catalog.name,
                    'color': this.catalog.color
                };

                let options = {};
                //if (this.catalog.isObsCore && this.catalog.isObsCore()) {
                    // If the source is obscore, save the table state inside the measurement table
                    // This is used to go back from a possible datalink table to the obscore one
                    options["save"] = true;
                //}
                view.aladin.measurementTable.hide();
                view.aladin.measurementTable.showMeasurement([singleSourceTable], options);
            }
            else if (this.catalog.onClick=='showPopup') {

                view.popup.setTitle('<br><br>');
                var m = '<div class="aladin-marker-measurement">';
                m += '<table>';
                for (var key in this.data) {
                    m += '<tr><td>' + key + '</td><td>' + this.data[key] + '</td></tr>';
                }
                m += '</table>';
                m += '</div>';
                view.popup.setText(m);
                view.popup.setSource(this);
                view.popup.show();
            }
            else if (typeof this.catalog.onClick === 'function') {
                this.catalog.onClick(this);
                view.lastClickedObject = this;
            }

        }
    };

    Source.prototype.actionOtherObjectClicked = function() {
        if (this.catalog && this.catalog.onClick) {
            this.deselect();
        }
    };
    
    return Source;
})();
