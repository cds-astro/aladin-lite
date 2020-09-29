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

        var xyviewArray = [];
        for (var k=0, len=this.radecArray.length; k<len; k++) {
            var xyview = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], view);
            if (!xyview) {
                return;
            }

            xyviewArray.push(xyview);
        }

        ctx.lineWidth = this.lineWidth;
        ctx.beginPath();

        const lastVertexIdx = xyviewArray.length-1;
        ctx.moveTo(xyviewArray[0][0], xyviewArray[0][1]);

        for (var k=0, len=lastVertexIdx; k<len; k++) {
            const line = new Line(xyviewArray[k][0], xyviewArray[k][1], xyviewArray[k+1][0], xyviewArray[k+1][1]);
            if (line.isInsideView(view.width, view.height)) {
                line.draw(ctx);
            }
        }

        if (this.closed) {
            const line = new Line(
                xyviewArray[lastVertexIdx][0],
                xyviewArray[lastVertexIdx][1],
                xyviewArray[0][0],
                xyviewArray[0][1]
            );

            if (line.isInsideView(view.width, view.height)) {
                line.draw(ctx);
            }
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
