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
 * Class Polyline
 *
 * A Polyline is a graphical overlay made of several connected points
 *
 * TODO: Polyline and Circle should derive from a common base class
 * TODO: index polyline, Circle in HEALPix pixels to avoid unneeded calls to draw
 *
 * Author: Thomas Boch[CDS], Matthieu Baumann[CDS]
 *
 *****************************************************************************/

import { Utils } from './Utils';

export let Footprint= (function() {
    // constructor
    //let Footprint = function(shapes, source) {
    let Footprint = function(shapes) {
        // All graphics overlay have an id
        this.id = 'footprint-' + Utils.uuidv4();

        /*this.source = source;
        if (this.source) {
            this.source.hasFootprint = true;
        }*/

        this.shapes = [].concat(shapes);

        this.isShowing = true;
    	this.isSelected = false;
        this.isHovered = false;

        this.overlay = null;
    };

    /*Footprint.prototype.setSource = function(source) {
        if (this.source) {
            this.source.hasFootprint = false;
        }

        this.source = source;

        if (this.source) {
            this.source.hasFootprint = true;
        }
    }*/

    /*Footprint.prototype.setCatalog = function(catalog) {
        if (this.source) {
            this.source.setCatalog(catalog);

        }
    };*/

    Footprint.prototype.show = function() {
        if (this.isShowing) {
            return;
        }

        this.isShowing = true;
        this.shapes.forEach((shape) => shape.show())
    };

    Footprint.prototype.hide = function() {
        if (!this.isShowing) {
            return;
        }

        this.isShowing = false;
        this.shapes.forEach((shape) => shape.hide())
    };

    Footprint.prototype.select = function() {
    	this.isSelected = true;
        this.shapes.forEach((shape) => shape.select())
    };

    Footprint.prototype.deselect = function() {
    	this.isSelected = false;
        this.shapes.forEach((shape) => shape.deselect())
    };

    Footprint.prototype.hover = function() {
        if (this.isHovered) {
            return;
        }

        this.isHovered = true;
        this.shapes.forEach((shape) => shape.hover())

        if (this.overlay) {
            this.overlay.reportChange();
            return;
        }
    };

    Footprint.prototype.unhover = function() {
        if (!this.isHovered) {
            return;
        }

        this.isHovered = false;
        this.shapes.forEach((shape) => shape.unhover())

        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Footprint.prototype.getLineWidth = function() {
        return this.shapes && this.shapes[0].getLineWidth();
    };


    Footprint.prototype.setLineWidth = function(lineWidth) {
        this.shapes.forEach((shape) => shape.setLineWidth(lineWidth))
    };

    Footprint.prototype.getSelectionLineWidth = function() {
        return this.shapes && this.shapes[0].getSelectionLineWidth();
    };

    Footprint.prototype.setSelectionLineWidth = function(selectionLineWidth) {
        this.shapes.forEach((shape) => shape.setSelectionLineWidth(selectionLineWidth))
    };

    Footprint.prototype.setColor = function(color) {
        if(!color) {
            return;
        }

        this.shapes.forEach((shape) => shape.setColor(color))
    };

    Footprint.prototype.setSelectionColor = function(color) {
        if (!color) {
            return;
        }

        this.shapes.forEach((shape) => shape.setSelectionColor(color))
    };

    Footprint.prototype.setHoverColor = function(color) {
        if (!color)
            return;

        this.shapes.forEach((shape) => shape.setHoverColor(color))
    };

    Footprint.prototype.isFootprint = function() {
        return true;
    }

    Footprint.prototype.draw = function(ctx, view, noStroke) {
        let hasBeenDrawn = false;
        for (let shape of this.shapes) {
            hasBeenDrawn |= shape.draw(ctx, view, noStroke)
        }

        return hasBeenDrawn;
    };

    /*Footprint.prototype.actionClicked = function() {
        if (this.source) {
            this.source.actionClicked(this);
        }
    };

    Footprint.prototype.actionOtherObjectClicked = function() {
        if (this.source) {
            this.source.actionOtherObjectClicked();
        }

        this.shapes.forEach((shape) => shape.deselect())
    };*/

    // If one shape is is stroke then the whole footprint is
    Footprint.prototype.isInStroke = function(ctx, view, x, y) {
        return this.shapes.some((shape) => shape.isInStroke(ctx, view, x, y));
    };

    Footprint.prototype.isTooSmall = function() {
        return this.shapes.every((shape) => shape.isTooSmall);
    };

    /*Footprint.prototype.getCatalog = function() {
        return this.source && this.source.catalog;
    };*/

    Footprint.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };

    Footprint.prototype.intersectsBBox = function(x, y, w, h, view) {
        return this.shapes.some((shape) => shape.intersectsBBox(x, y, w, h, view));
    };

    return Footprint;
})();
