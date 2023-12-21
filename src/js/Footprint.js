// Copyright 2015 - UDS/CNRS
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

import { AladinUtils } from './AladinUtils.js';
import { Utils } from './Utils';

export let Footprint= (function() {
    // constructor
    let Footprint = function(shapes, source) {
        // All graphics overlay have an id
        this.id = 'footprint-' + Utils.uuidv4();

        this.source = source;
        this.shapes = shapes;

        this.isShowing = true;

        this.overlay = null;
    };

    Footprint.prototype.setCatalog = function(catalog) {
        if (this.source) {
            this.source.setCatalog(catalog);
        }
    };

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
        this.shapes.forEach((shape) => shape.select())
    };

    Footprint.prototype.deselect = function() {
        this.shapes.forEach((shape) => shape.deselect())
    };

    Footprint.prototype.setLineWidth = function(lineWidth) {
        this.shapes.forEach((shape) => shape.setLineWidth(lineWidth))
    };

    Footprint.prototype.getLineWidth = function() {
        if (this.shapes && this.shapes.length > 0) {
            return this.shapes[0].getLineWidth();
        }
    };

    Footprint.prototype.setColor = function(color) {
        this.shapes.forEach((shape) => shape.setColor(color))
    };

    Footprint.prototype.setSelectionColor = function(color) {
        this.shapes.forEach((shape) => shape.setSelectionColor(color))
    };

    Footprint.prototype.isFootprint = function() {
        return true;
    }

    Footprint.prototype.draw = function(ctx, view, noStroke) {
        this.shapes.forEach((shape) => shape.draw(ctx, view, noStroke))
    };

    Footprint.prototype.actionClicked = function() {
        if (this.source) {
            this.source.actionClicked(this);
        }
    };

    Footprint.prototype.actionOtherObjectClicked = function() {
        if (this.source) {
            this.source.actionOtherObjectClicked();
        }

        this.shapes.forEach((shape) => shape.deselect())
    };

    // If one shape is is stroke then the whole footprint is
    Footprint.prototype.isInStroke = function(ctx, view, x, y) {
        return this.shapes.some((shape) => shape.isInStroke(ctx, view, x, y));
    };

    Footprint.prototype.getCatalog = function() {
        return this.source && this.source.catalog;
    };

    Footprint.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };

    Footprint.prototype.intersectsBBox = function(x, y, w, h, view) {
        if(this.source) {
            let s = this.source;

            if (!s.isShowing) {
                return false;
            }

            let c = null;
            if (s.x && s.y) {
                c = {
                    x: s.x,
                    y: s.y,
                };
            } else {
                var xy = AladinUtils.radecToViewXy(s.ra, s.dec, view.aladin);
                if (!xy) {
                    return false;
                }

                c = {
                    x: xy[0],
                    y: xy[1],
                };
            }

            if (c.x >= x && c.x <= x + w && c.y >= y && c.y <= y + h) {
                return true;
            }
        }
        return false;
    };

    return Footprint;
})();
