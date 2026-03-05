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
 * Author: Thomas Boch[CDS], Matthieu Baumann [CDS]
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
* @property {number} [lineWidth=2] - The line width in pixels (inherited from overlay, if any, where it defaults to 3)
* @property {number} [selectionLineWidth=lineWidth] - The line width in pixels when the shape is selected
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

    function _drawLine(l, ctx) {
        ctx.moveTo(l.x1, l.y1);
        ctx.lineTo(l.x2, l.y2);
    }

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
        this.selectionLineWidth = options["selectionLineWidth"] || undefined;
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
        this.setLineWidth(this.getLineWidth() + 2)
        this.setSelectionLineWidth(this.getSelectionLineWidth() + 2)
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Polyline.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;
        this.setLineWidth(this.getLineWidth() - 2)
        this.setSelectionLineWidth(this.getSelectionLineWidth() - 2)
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

    Polyline.prototype.getSelectionLineWidth = function() {
        return this.selectionLineWidth;
    };

    Polyline.prototype.setSelectionLineWidth = function(selectionLineWidth) {
        if (this.selectionLineWidth == selectionLineWidth) {
            return;
        }

        this.selectionLineWidth = selectionLineWidth;
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

        // Decide which line width to use.
        if (!this.lineWidth) {
            this.lineWidth = (this.overlay && this.overlay.lineWidth) || 2;
        }
        var drawingLineWidth = this.lineWidth;
        if (this.isSelected && this.selectionLineWidth) {
            drawingLineWidth = this.selectionLineWidth;
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

        let behind = true;
        for (var k=0; k<len; k++) {
            var xyview = view.aladin.world2pix(this.raDecArray[k][0], this.raDecArray[k][1]);

            if (!xyview) {
                xyView.push(undefined);
            } else {
                behind = false;
                let [x, y] =  xyview
                xyView.push({x, y});

                xmin = Math.min(xmin, x);
                ymin = Math.min(ymin, y);
                xmax = Math.max(xmax, x);
                ymax = Math.max(ymax, y);
            }
        }

        if (behind)
            return false;

        // 2. do not draw the polygon if it lies outside the view
        if (xmax < 0 || xmin > view.width || ymax < 0 || ymin > view.height) {
            return false;
        }

        // do not draw neither if the polygone does not lie inside lineWidth
        if (!noSmallCheck) {
            this.isTooSmall = (xmax - xmin) < drawingLineWidth && (ymax - ymin) < drawingLineWidth;

            if (this.isTooSmall) {
                return false;
            }
        }

        let drawLine;
        let fillPoly;

        if (view.projection === ProjectionEnum.SIN) {
            drawLine = (v0, v1) => {
                if (v0 === undefined || v1 === undefined) {
                    return false;
                }

                const l = {x1: v0.x, y1: v0.y, x2: v1.x, y2: v1.y};

                if (Polyline.isInsideView(l.x1, l.y1, l.x2, l.y2, view.width, view.height)) {
                    _drawLine(l, ctx);
                }
            };

            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    if (v0 === undefined || v1 === undefined)
                        return false;

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

        ctx.globalAlpha = this.opacity;
        ctx.lineWidth = drawingLineWidth;
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

            ctx.fillStyle = this.fillColor;
            ctx.fill();
        }

        return true;
    };

    Polyline.prototype.isInStroke = function(ctx, view, x, y) {
        ctx.beginPath()
        ctx.lineWidth = this.lineWidth;

        let pointXY = [];
        for (var j = 0; j < this.raDecArray.length; j++) {
            var xy = view.aladin.world2pix(this.raDecArray[j][0], this.raDecArray[j][1]);
            if (!xy) {
                pointXY.push(undefined)
            } else {
                pointXY.push({
                    x: xy[0],
                    y: xy[1]
                });
            }
        }

        const lastPointIdx = pointXY.length - 1;
        for (var l = 0; l < lastPointIdx; l++) {
            let v1 = pointXY[l];
            let v2 = pointXY[l + 1];

            if (v1 && v2) {
                const line = {x1: v1.x, y1: v1.y, x2: v2.x, y2: v2.y};                                   // new segment
                _drawLine(line, ctx);

                if (ctx.isPointInStroke(x, y)) {                    // x, y is on line?
                    return true;
                }
            }
        }

        if(this.closed) {
            let v1 = pointXY[lastPointIdx];
            let v2 = pointXY[0];

            if (v1 && v2) {
                const line = {x1: v1.x, y1: v1.y, x2: v2.x, y2: v2.y};                                   // new segment
                _drawLine(line, ctx);

                if (ctx.isPointInStroke(x, y)) {                    // x,y is on line?
                    return true;
                }
            }
        }

        return false;
    };

    Polyline.prototype.intersectsBBox = function (x, y, w, h, view) {
        let n = this.raDecArray.length;
        if (n < 2) return false;

        // 2️⃣ Edge intersection test
        let i = this.closed ? n - 1 : 0;
        let j = this.closed ? 0 : 1;

        let poly = [];
        while (j < n) {
            const p1 = this.raDecArray[i];
            const p2 = this.raDecArray[j];

            const xy1 = view.aladin.world2pix(p1[0], p1[1]);
            const xy2 = view.aladin.world2pix(p2[0], p2[1]);

            // Skip invalid segments, do NOT abort
            if (!xy1 || !xy2) {
                i = j++;
                continue;
            }

            const a = { x: xy1[0], y: xy1[1] };
            const b = { x: xy2[0], y: xy2[1] };

            poly.push(a);

            if (Polyline.segmentIntersectsBox(a, b, x, y, w, h)) {
                return true;
            }

            i = j++;
        }

        if (this.closed && poly.length === this.raDecArray.length) {
            const corners = [
                { x,  y },
                { x: x + w, y },
                { x: x + w, y: y + h },
                { x,  y: y + h }
            ];

            for (const c of corners) {
                if (Polyline.pointInPolygon(c, poly)) {
                    return true;
                }
            }
        }

        return false;
    };


    Polyline.segmentIntersectsBox = function (p1, p2, x, y, w, h) {
        const x2 = x + w;
        const y2 = y + h;

        // 1️⃣ Endpoint inside box
        if (Polyline.pointInBox(p1, x, y, x2, y2) ||
            Polyline.pointInBox(p2, x, y, x2, y2)) {
            return true;
        }

        // 2️⃣ Check intersection with 4 box edges
        return (
            Polyline.segmentsIntersect(p1, p2, { x, y }, { x: x2, y }) ||     // top
            Polyline.segmentsIntersect(p1, p2, { x: x2, y }, { x: x2, y: y2 }) || // right
            Polyline.segmentsIntersect(p1, p2, { x: x2, y: y2 }, { x, y: y2 }) || // bottom
            Polyline.segmentsIntersect(p1, p2, { x, y: y2 }, { x, y })           // left
        );
    };

    Polyline.pointInBox = function (p, x1, y1, x2, y2) {
        return p.x >= x1 && p.x <= x2 && p.y >= y1 && p.y <= y2;
    };

    Polyline.pointInPolygon = function (xy, poly) {
        let inside = false;

        for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
            const xi = poly[i].x, yi = poly[i].y;
            const xj = poly[j].x, yj = poly[j].y;

            const intersect =
                ((yi > xy.y) !== (yj > xy.y)) &&
                (xy.x < (xj - xi) * (xy.y - yi) / (yj - yi) + xi);

            if (intersect) inside = !inside;
        }

        return inside;
    };

    Polyline.segmentsIntersect = function (p1, p2, p3, p4) {
        function orient(a, b, c) {
            return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        }

        const o1 = orient(p1, p2, p3);
        const o2 = orient(p1, p2, p4);
        const o3 = orient(p3, p4, p1);
        const o4 = orient(p3, p4, p2);

        return o1 * o2 < 0 && o3 * o4 < 0;
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
        let left =   Polyline.segmentsIntersect({x: x1, y: y1}, {x: x2, y: y2}, {x: 0, y: 0}, {x: 0, y: rh});
        let right =  Polyline.segmentsIntersect({x: x1, y: y1}, {x: x2, y: y2}, {x: rw, y: 0}, {x: rw, y: rh});
        let top =    Polyline.segmentsIntersect({x: x1, y: y1}, {x: x2, y: y2}, {x: 0, y: 0}, {x: rw, y: 0});
        let bottom = Polyline.segmentsIntersect({x: x1, y: y1}, {x: x2, y: y2}, {x: 0, y: rh}, {x: rw, y: rh});

        // if ANY of the above are true, the line
        // has hit the rectangle
        if (left || right || top || bottom) {
            return true;
        }

        return false;
    };

    return Polyline;
})();
