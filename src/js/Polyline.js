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
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import { AladinUtils } from './AladinUtils.js';
import { Line } from './Line.js';
import { Utils } from './Utils.js';
import { Overlay } from "./Overlay.js";

export let Polyline= (function() {
    // constructor
    let Polyline = function(radecArray, options) {
        options = options || {};
        this.color     = options['color']     || undefined;
        this.lineWidth = options["lineWidth"] || 2;

        if (options["closed"]) {
            this.closed = options["closed"];
        } else {
            this.closed = false;
        }

        // All graphics overlay have an id
        this.id = 'polyline-' + Utils.uuidv4();

        this.radecArray = radecArray;
        this.overlay = null;

    	this.isShowing = true;
    	this.isSelected = false;

        this.selectionColor = '#00ff00';
    };

    Polyline.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };

    Polyline.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.setLineWidth = function(lineWidth) {
        if (this.lineWidth == lineWidth) {
            return;
        }
        this.lineWidth = lineWidth;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.setColor = function(color) {
        if (this.color == color) {
            return;
        }
        this.color = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.setSelectionColor = function(color) {
        if (this.selectionColor == color) {
            return;
        }
        this.selectionColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Polyline.prototype.isFootprint = function() {
        // The polyline is a footprint if it describes a polygon (i.e. a closed polyline)
        return this.closed;
    }

    Polyline.prototype.draw = function(ctx, view, noStroke) {
        if (! this.isShowing) {
            return;
        }

        if (! this.radecArray || this.radecArray.length<2) {
            return;
        }

        noStroke = noStroke===true || false;

        var baseColor = this.color;
        if (! baseColor && this.overlay) {
            baseColor = this.overlay.color;
        }
        if (! baseColor) {
            baseColor = '#ff0000';
        }

        if (this.isSelected) {
            if(this.selectionColor) {
                ctx.strokeStyle = this.selectionColor;
            } else {
                ctx.strokeStyle = Overlay.increaseBrightness(baseColor, 50);
            }
        }
        else {
            ctx.strokeStyle= baseColor;
        }

        // 0. Determine the clockwise order of the vertices given in the
        // space
        let ccwOrder = function(a, b, c) {
            return a.x*b.y + a.y*c.x + b.x*c.y - c.x*b.y - c.y*a.x - b.x*a.y > 0.0;
        };

        const ccwGoodOrder = ccwOrder(
            {x: this.radecArray[0][0], y: this.radecArray[0][1]},
            {x: this.radecArray[1][0], y: this.radecArray[1][1]},
            {x: this.radecArray[2][0], y: this.radecArray[2][1]},
        );

        // 1. project the vertices into the screen
        //    and computes a BBox
        let xyView = [];
        let len = this.radecArray.length;

        let xmin = Number.POSITIVE_INFINITY
        let xmax = Number.NEGATIVE_INFINITY
        let ymin = Number.POSITIVE_INFINITY
        let ymax = Number.NEGATIVE_INFINITY;

        for (var k=0; k<len; k++) {
            var xyview = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], view);
            if (!xyview) {
                return;
            }

            xyView.push({x: xyview[0], y: xyview[1]});

            xmin = Math.min(xmin, xyview[0]);
            ymin = Math.min(ymin, xyview[1]);
            xmax = Math.max(xmax, xyview[0]);
            ymax = Math.max(ymax, xyview[1]);
        }

        // 2. do not draw the polygon if it lies in less than 1 pixel
        if ((xmax - xmin) < 1 || (ymax - ymin) < 1) {
            return;
        }

        let drawLine = (v0, v1) => {
            const line = new Line(v0.x, v0.y, v1.x, v1.y);

            if (line.isInsideView(view.width, view.height)) {
                line.draw(ctx);
            }
        };

        // 3. Check whether the polygon do not cross the view
        let nSegment = this.closed ? len : len - 1;

        let v0 = this.closed ? len - 1 : 0;
        let v1 = this.closed ? 0 : 1;
        let v2 = this.closed ? 1 : 2;

        let drawPolygon = true;
        for (var k = 0; k < nSegment; k++) {
            let ccwTriOrder = ccwOrder(xyView[v0], xyView[v1], xyView[v2])

            if (ccwGoodOrder != ccwTriOrder) {
                // if it cross the view, we end up here
                drawPolygon = false;
                return;
            }

            v0 = v1;
            v1 = v2;
            v2 = (v2 + 1) % len;
        }

        if (!drawPolygon) {
            return;
        }

        // 4. Finally, draw all the polygon, segment by segment
        v0 = this.closed ? len - 1 : 0;
        v1 = this.closed ? 0 : 1;

        ctx.lineWidth = this.lineWidth;
        ctx.beginPath();
        
        for (var k = 0; k < nSegment; k++) {
            drawLine(xyView[v0], xyView[v1])

            v0 = v1;
            v1 = v1 + 1;
        }

        if (!noStroke) {
            ctx.stroke();
        }
    };

    Polyline.prototype.isInStroke = function(ctx, view, x, y) {
        let pointXY = [];
        for (var j = 0; j < this.radecArray.length; j++) {
            var xy = AladinUtils.radecToViewXy(this.radecArray[j][0], this.radecArray[j][1], view);
            if (!xy) {
                return false;
            }
            pointXY.push({
                x: xy[0],
                y: xy[1]
            });
        }

        const lastPointIdx = pointXY.length - 1;
        for (var l = 0; l < lastPointIdx; l++) {
            const line = new Line(pointXY[l].x, pointXY[l].y, pointXY[l + 1].x, pointXY[l + 1].y);                                   // new segment
            line.draw(ctx, true);

            if (ctx.isPointInStroke(x, y)) {                    // x,y is on line?
                return true;
            }
        }

        if(this.closed) {
            const line = new Line(pointXY[lastPointIdx].x, pointXY[lastPointIdx].y, pointXY[0].x, pointXY[0].y);                                   // new segment
            line.draw(ctx, true);
            
            if (ctx.isPointInStroke(x, y)) {                    // x,y is on line?
                return true;
            }
        }

        return false;
    };

    Polyline.prototype.intersectsBBox = function(x, y, w, h) {
        // todo
    };

    return Polyline;
})();
