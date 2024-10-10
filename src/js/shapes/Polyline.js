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

import { Utils } from '../Utils';
import { GraphicOverlay } from "../Overlay.js";
import { ProjectionEnum } from "../ProjectionEnum.js";

/**
* @typedef {Object} ShapeOptions
* @description Options for describing a shape
*
* @property {string} [color] - The color of the shape
* @property {string} [fill=false] - Fill the shape with fillColor
* @property {string} [fillColor] - A filling color for the shape
* @property {number} [lineWidth=3] - The line width in pixels
* @property {number} [opacity=1] - The opacity, between 0 (totally transparent) and 1 (totally opaque)
* @property {string} [selectionColor='#00ff00'] - A selection color
* @property {string} [hoverColor] -  A hovered color
*/

export let Polyline = (function() {

    function _calculateMag2ForNoSinProjections(l, view) {
        // check if the line is too big (in the clip space) to be drawn
        const [x1, y1] = view.wasm.screenToClip(l.x1, l.y1);
        const [x2, y2] = view.wasm.screenToClip(l.x2, l.y2);

        const mag2 = (x1 - x2)*(x1 - x2) + (y1 - y2)*(y1 - y2);
        return mag2;
    }

    function _drawLine(l, ctx, noStroke) {
        noStroke = noStroke===true || false;

        ctx.beginPath();
        ctx.moveTo(l.x1, l.y1);
        ctx.lineTo(l.x2, l.y2);

        if (!noStroke) {
            ctx.stroke();
        }
    }

    /*function _isAcrossCollignonZoneForHpxProjection(line, view) {
        const [x1, y1] = view.wasm.screenToClip(line.x1, line.y1);
        const [x2, y2] = view.wasm.screenToClip(line.x2, line.y2);

        // x, y, between -1 and 1
        let triIdxCollignionZone = function(x, y) {
            let xZone = Math.floor((x * 0.5 + 0.5) * 4.0);
            return xZone + 4 * (y > 0.0);
        };

        let isInCollignionZone = function(x, y) {
            return Math.abs(y) > 0.5;
        };

        if (isInCollignionZone(x1, y1) && isInCollignionZone(x2, y2)) {
            if (triIdxCollignionZone(x1, y1) === triIdxCollignionZone(x2, y2)) {
                return false;
            } else {
                return true;
            }
        }

        return false;
    }*/

    /**
     * Represents a polyline shape
     *
     * @class
     * @constructs Polyline
     * @param {Array.<number[]>} raDecArray - right-ascension/declination 2-tuple array describing the polyline's vertices in degrees
     * @param {ShapeOptions} options - Configuration options for the polyline. Additional properties:
     * @param {boolean} [options.closed=false] - Close the polyline, default to false.
     * 
     * @returns {Polyline} - The polyline shape object
     */
    let Polyline = function(raDecArray, options) {
        options = options || {};
        this.color     = options['color']     || undefined;
        this.fill      = options['fill']      || false;
        this.fillColor = options['fillColor'] || undefined;
        this.opacity   = options['opacity']   || undefined;
        this.lineWidth = options["lineWidth"] || undefined;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;

        this.closed = (options["closed"] !== undefined) ? options["closed"] : false;

        // All graphics overlay have an id
        this.id = 'polyline-' + Utils.uuidv4();

        this.raDecArray = raDecArray;
        this.overlay = null;

    	this.isShowing = true;
    	this.isSelected = false;
        this.isHovered = false;
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

    Polyline.prototype.hover = function() {
        if (this.isHovered) {
            return;
        }
        this.isHovered = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.getLineWidth = function() {
        return this.lineWidth;
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
        if (!color || this.color == color) {
            return;
        }

        this.color = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.setSelectionColor = function(color) {
        if (!color || this.selectionColor == color) {
            return;
        }
        this.selectionColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.setHoverColor = function(color) {
        if (!color || this.hoverColor == color) {
            return;
        }
        this.hoverColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.isFootprint = function() {
        // The polyline is a footprint if it describes a polygon (i.e. a closed polyline)
        return this.closed;
    }

    Polyline.prototype.draw = function(ctx, view, noStroke, noSmallCheck) {
        if (! this.isShowing) {
            return false;
        }

        if (! this.raDecArray || this.raDecArray.length<2) {
            return false;
        }

        noSmallCheck = noSmallCheck===true || false;
        noStroke = noStroke===true || false;

        var baseColor = this.color;
        if (! baseColor && this.overlay) {
            baseColor = this.overlay.color;
        }
        if (! baseColor) {
            baseColor = '#ff0000';
        }

        if (!this.lineWidth) {
            this.lineWidth = (this.overlay && this.overlay.lineWidth) || 2;
        }

        if (this.isSelected) {
            if(this.selectionColor) {
                ctx.strokeStyle = this.selectionColor;
            } else {
                ctx.strokeStyle = GraphicOverlay.increaseBrightness(baseColor, 50);
            }
        } else if (this.isHovered) {
            ctx.strokeStyle = this.hoverColor || GraphicOverlay.increaseBrightness(baseColor, 25);
        } else {
            ctx.strokeStyle = baseColor;
        }

        // 1. project the vertices into the screen
        //    and computes a BBox
        let xyView = [];
        let len = this.raDecArray.length;

        let xmin = Number.POSITIVE_INFINITY
        let xmax = Number.NEGATIVE_INFINITY
        let ymin = Number.POSITIVE_INFINITY
        let ymax = Number.NEGATIVE_INFINITY;

        for (var k=0; k<len; k++) {
            var xyview = view.aladin.world2pix(this.raDecArray[k][0], this.raDecArray[k][1]);
            if (!xyview) {
                return false;
            }

            xyView.push({x: xyview[0], y: xyview[1]});

            xmin = Math.min(xmin, xyview[0]);
            ymin = Math.min(ymin, xyview[1]);
            xmax = Math.max(xmax, xyview[0]);
            ymax = Math.max(ymax, xyview[1]);
        }

        // 2. do not draw the polygon if it lies outside the view
        if (xmax < 0 || xmin > view.width || ymax < 0 || ymin > view.height) {
            return false;
        }

        // do not draw neither if the polygone does not lie inside lineWidth
        if (!noSmallCheck) {
            this.isTooSmall = (xmax - xmin) < this.lineWidth || (ymax - ymin) < this.lineWidth;

            if (this.isTooSmall) {
                return false;
            }
        }

        let drawLine;
        let fillPoly;

        if (view.projection === ProjectionEnum.SIN) {
            drawLine = (v0, v1) => {
                const l = {x1: v0.x, y1: v0.y, x2: v1.x, y2: v1.y};

                if (Polyline.isInsideView(l.x1, l.y1, l.x2, l.y2, view.width, view.height)) {
                    _drawLine(l, ctx);
                }
            };

            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    const l = {x1: v0.x, y1: v0.y, x2: v1.x, y2: v1.y};

                    if (index === 0) {
                        ctx.beginPath();
                        ctx.moveTo(l.x1, l.y1);
                    } else {
                        ctx.lineTo(l.x1, l.y1);
                    }

                    return true;
                };
            }
        } else {
            drawLine = (v0, v1) => {
                const l = {x1: v0.x, y1: v0.y, x2: v1.x, y2: v1.y};

                if (Polyline.isInsideView(l.x1, l.y1, l.x2, l.y2, view.width, view.height)) {
                    const mag2 = _calculateMag2ForNoSinProjections(l, view);

                    if (mag2 < 0.2) {
                        _drawLine(l, ctx);
                    }
                }
            };
            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    const l = {x1: v0.x, y1: v0.y, x2: v1.x, y2: v1.y};

                    const mag2 = _calculateMag2ForNoSinProjections(l, view);

                    if (mag2 < 0.2) {
                        if (index === 0) {
                            ctx.beginPath();
                            ctx.moveTo(l.x1, l.y1);
                        } else {
                            ctx.lineTo(l.x1, l.y1);
                        }

                        return true;
                    } else {
                        return false;
                    }
                };
            }
        }

        // 4. Finally, draw all the polygon, segment by segment
        let nSegment = this.closed ? len : len - 1;

        let v0 = this.closed ? len - 1 : 0;
        let v1 = this.closed ? 0 : 1;

        ctx.lineWidth = this.lineWidth;
        ctx.beginPath();

        for (var k = 0; k < nSegment; k++) {
            drawLine(xyView[v0], xyView[v1]);

            v0 = v1;
            v1 = v1 + 1;
        }

        if (!noStroke) {
            ctx.stroke();
        }

        if (this.fill && this.closed) {
            v0 = len - 1;
            v1 = 0;

            let index = 0;
            for (var k = 0; k < nSegment; k++) {
                if (fillPoly(xyView[v0], xyView[v1], index)) {
                    index++;
                }

                v0 = v1;
                v1 = v1 + 1;
            }

            //ctx.globalAlpha = 1;
            ctx.save();
            ctx.fillStyle = this.fillColor;
            ctx.globalAlpha = this.opacity;
            ctx.fill();
            ctx.restore();
        }

        return true;
    };

    Polyline.prototype.isInStroke = function(ctx, view, x, y) {
        ctx.lineWidth = this.lineWidth;

        let pointXY = [];
        for (var j = 0; j < this.raDecArray.length; j++) {
            var xy = view.aladin.world2pix(this.raDecArray[j][0], this.raDecArray[j][1]);
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
            const line = {x1: pointXY[l].x, y1: pointXY[l].y, x2: pointXY[l + 1].x, y2: pointXY[l + 1].y};                                   // new segment
            _drawLine(line, ctx, true);

            if (ctx.isPointInStroke(x, y)) {                    // x,y is on line?
                return true;
            }
        }

        if(this.closed) {
            const line = {x1: pointXY[lastPointIdx].x, y1: pointXY[lastPointIdx].y, x2: pointXY[0].x, y2: pointXY[0].y};                                   // new segment
            _drawLine(line, ctx, true);

            if (ctx.isPointInStroke(x, y)) {                    // x,y is on line?
                return true;
            }
        }

        return false;
    };

    Polyline.prototype.intersectsBBox = function(x, y, w, h, view) {
        for (let i = 0; i < this.raDecArray.length - 1; i++) {
            let p1 = this.raDecArray[i];
            let p2 = this.raDecArray[i + 1];

            let xy1 = view.aladin.world2pix(p1[0], p1[1]);
            let xy2 = view.aladin.world2pix(p2[0], p2[1]);

            if (!xy1 || !xy2) {
                return false;
            }

            xy1 = {x: xy1[0], y: xy1[1]};
            xy2 = {x: xy2[0], y: xy2[1]};
    
            // Check if line segment intersects with the bounding box
            if (this.lineIntersectsBox(xy1, xy2, x, y, w, h)) {
                return true;
            }
        }

        return false;
    };

    Polyline.prototype.lineIntersectsBox = function(p1, p2, x, y, w, h) {
        // Check if line segment is completely outside the box
        if ((p1.x < x && p2.x < x) || 
            (p1.y < y && p2.y < y) || 
            (p1.x > x + w && p2.x > x + w) || 
            (p1.y > y + h && p2.y > y + h)) {
            return false;
        }

        let m = (p2.y - p1.y) / (p2.x - p1.x);  // Slope of the line
        let c = p1.y - m * p1.x;  // y-intercept of the line

        // Check if line intersects with the sides of the box
        if ((p1.y >= y && p1.y <= y + h) || 
            (p2.y >= y && p2.y <= y + h) || 
            (m * x + c >= y && m * x + c <= y + h) || 
            (m * (x + w) + c >= y && m * (x + w) + c <= y + h)) {
            return true;
        }

        return false;       
    };

    // static methods
    // Method for testing whether a line is inside the view
    // http://www.jeffreythompson.org/collision-detection/line-rect.php
    Polyline.isInsideView = function(x1, y1, x2, y2, rw, rh) {
        if (x1 >= 0 && x1 <= rw && y1 >= 0 && y1 <= rh) {
            return true;
        }
        if (x2 >= 0 && x2 <= rw && y2 >= 0 && y2 <= rh) {
            return true;
        }

        // check if the line has hit any of the rectangle's sides
        // uses the Line/Line function below
        let left =   Polyline._intersectLine(x1, y1, x2, y2, 0, 0, 0, rh);
        let right =  Polyline._intersectLine(x1, y1, x2, y2, rw, 0, rw, rh);
        let top =    Polyline._intersectLine(x1, y1, x2, y2, 0, 0, rw, 0);
        let bottom = Polyline._intersectLine(x1, y1, x2, y2, 0, rh, rw, rh);
    
        // if ANY of the above are true, the line
        // has hit the rectangle
        if (left || right || top || bottom) {
            return true;
        }

        return false;
    };

    Polyline._intersectLine = function(x1, y1, x2, y2, x3, y3, x4, y4) {
        // Calculate the direction of the lines
        let uA = ((x4-x3)*(y1-y3) - (y4-y3)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
        let uB = ((x2-x1)*(y1-y3) - (y2-y1)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
    
        // If uA and uB are between 0-1, lines are colliding
        if (uA >= 0 && uA <= 1 && uB >= 0 && uB <= 1) {
            return true;
        }
        return false;
    };

    return Polyline;
})();
