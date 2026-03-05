// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//

/******************************************************************************
 * Aladin Lite project
 * 
 * File Source
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


/**
* @typedef {Object} SourceOptions
* @description Options for describing a source
*
* @property {boolean} [marker=false] - If the source is a marker. A source marker is associated to a popup that is shown when clicking on it.
* @property {string} [popupTitle=''] - Only for marker. The title of the popup.
* @property {string} [popupDesc=''] - Only for marker. The content of the popup.
* @property {number} [useMarkerDefaultIcon=true] - Only for marker. Set to false to use the regular shape option from the catalog and not the specific marker shape.
*/

/**
* @typedef {Object} MarkerOptions
* @description Options for describing a source marker
*
* @property {string} [popupTitle=''] - Only for marker. The title of the popup.
* @property {string} [popupDesc=''] - Only for marker. The content of the popup.
* @property {number} [useMarkerDefaultIcon=true] - Only for marker. Set to false to use the regular shape option from the catalog and not the specific marker shape.
*/
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
        this.isHovered = false;
    };

    Source.prototype.setFootprint = function(footprint) {
        this.footprint = footprint;
    }

    Source.prototype.setCatalog = function(catalog) {
        this.catalog = catalog;
    };

    Source.prototype.getCatalog = function() {
        return this.catalog;
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

        if (this.footprint) {
            this.footprint.select();
        }

        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    Source.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }

        this.isSelected = false;

        if (this.footprint) {
            this.footprint.deselect();
        }

        if (this.catalog) {
            this.catalog.reportChange();
        }
    };

    Source.prototype.hover = function() {
        if (this.isHovered) {
            return;
        }

        this.isHovered = true;

        if (this.footprint) {
            this.footprint.hover();
        }

        if (this.catalog) {
            this.catalog.reportChange();
        }
    }

    Source.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;

        if (this.footprint) {
            this.footprint.unhover();
        }

        if (this.catalog) {
            this.catalog.reportChange();
        }
    }

    Source.prototype.setImage = function(image) {
        this.image = image;
    }

    Source.prototype.setShape = function(shape) {
        this.shape = shape;
    }

    Source.prototype.setColor = function(color) {
        this.color = color;

        if (this.footprint) {
            this.footprint.setColor(color);
        }
    }

    Source.prototype.setSize = function(size) {
        this.size = Math.max(size, 1.0);
    }

    /**
     * Simulates a click on the source
     *
     * @memberof Source 
     * @param {Footprint|Source} [obj] - If not given, the source is taken as the object to be selected 
     */
    Source.prototype.actionClicked = function(obj) {
        if (this.catalog) {
            var view = this.catalog.view;

            if (this.marker) {
                view.aladin.popup.setTitle(this.popupTitle);
                view.aladin.popup.setText(this.popupDesc);
                view.aladin.popup.setSource(this);

                if (this.popupDesc || this.popupTitle)
                    view.aladin.popup.show();
    
                return;
            }

            if (this.catalog.onClick) {
                if (this.catalog.onClick == 'showTable') {
                    if (!obj) {
                        obj = this;
                    }
                    view.selectObjects([[obj]]);
                }
                else if (this.catalog.onClick == 'showPopup') {
                    var title = '<br><br>';
                    var desc;
                    desc = '<div class="aladin-marker-measurement">';
                    desc += '<table>';
                    for (var key in this.data) {
                        desc += '<tr><td>' + key + '</td><td>' + this.data[key] + '</td></tr>';
                    }
                    desc += '</table>';
                    desc += '</div>';
    
                    view.aladin.popup.setTitle(title);
                    view.aladin.popup.setText(desc);
                    view.aladin.popup.setSource(this);
                    view.aladin.popup.show();
                }
                else if (typeof this.catalog.onClick === 'function') {
                    this.catalog.onClick(this);
                    view.lastClickedObject = this;
                }
            }
        }
    };

    Source.prototype.isFootprint = function() {
        return this.footprint !== undefined && this.footprint !== null;
    }

    Source.prototype.actionOtherObjectClicked = function() {
        if (this.catalog && this.catalog.onClick) {
            this.deselect();
        }

        if (this.footprint) {
            this.footprint.deselect()
        }
    };

    Source.prototype.draw = function(ctx, width, height) {
        if (this.catalog) {
            this.catalog.drawSource(this, ctx, width, height);
        }
    }

    return Source;
})();
