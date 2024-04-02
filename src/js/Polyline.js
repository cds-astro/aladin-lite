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
import { Utils } from './Utils';
import { Overlay } from "./Overlay.js";
import { ProjectionEnum } from "./ProjectionEnum.js";


export let Polyline= (function() {
    function _calculateMag2ForNoSinProjections(line, view) {
        // check if the line is too big (in the clip space) to be drawn
        const [x1, y1] = view.wasm.screenToClip(line.x1, line.y1);
        const [x2, y2] = view.wasm.screenToClip(line.x2, line.y2);

        const mag2 = (x1 - x2)*(x1 - x2) + (y1 - y2)*(y1 - y2);
        return mag2;
    }

    function _isAcrossCollignonZoneForHpxProjection(line, view) {
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
    }

    // constructor
    let Polyline = function(radecArray, options) {
        options = options || {};
        this.color     = options['color']     || undefined;
        this.fill      = options['fill']      || false;
        this.fillColor = options['fillColor'] || undefined;
        this.opacity   = options['opacity']   || undefined;
        this.lineWidth = options["lineWidth"] || undefined;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;

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

        if (!this.lineWidth) {
            this.lineWidth = this.overlay.lineWidth || 2;
        }

        if (this.isSelected) {
            if(this.selectionColor) {
                ctx.strokeStyle = this.selectionColor;
            } else {
                ctx.strokeStyle = Overlay.increaseBrightness(baseColor, 50);
            }
        } else if (this.isHovered) {
            ctx.strokeStyle = this.hoverColor || Overlay.increaseBrightness(baseColor, 25);
        } else {
            ctx.strokeStyle = baseColor;
        }

        // 1. project the vertices into the screen
        //    and computes a BBox
        let xyView = [];
        let len = this.radecArray.length;

        let xmin = Number.POSITIVE_INFINITY
        let xmax = Number.NEGATIVE_INFINITY
        let ymin = Number.POSITIVE_INFINITY
        let ymax = Number.NEGATIVE_INFINITY;

        for (var k=0; k<len; k++) {
            var xyview = view.aladin.world2pix(this.radecArray[k][0], this.radecArray[k][1]);
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

        let drawLine;
        let fillPoly;

        if (view.projection === ProjectionEnum.SIN) {
            drawLine = (v0, v1) => {
                const line = new Line(v0.x, v0.y, v1.x, v1.y);

                if (line.isInsideView(view.width, view.height)) {
                    line.draw(ctx);
                }
            };

            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    const line = new Line(v0.x, v0.y, v1.x, v1.y);
                    if (index === 0) {
                        ctx.beginPath();
                        ctx.moveTo(line.x1, line.y1);
                    } else {
                        ctx.lineTo(line.x1, line.y1);
                    }

                    return true;
                };
            }
        /*} else if (view.projection === ProjectionEnum.HPX) {
            drawLine = (v0, v1) => {
                const line = new Line(v0.x, v0.y, v1.x, v1.y);

                if (_isAcrossCollignonZoneForHpxProjection(line, view)) {
                    return;
                } 

                if (line.isInsideView(view.width, view.height)) {
                    const mag2 = _calculateMag2ForNoSinProjections(line, view);

                    if (mag2 < 0.1) {
                        line.draw(ctx);
                    }
                }
            };

            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    const line = new Line(v0.x, v0.y, v1.x, v1.y);

                    if (_isAcrossCollignonZoneForHpxProjection(line, view)) {
                        return;
                    } 

                    const mag2 = _calculateMag2ForNoSinProjections(line, view);

                    if (mag2 < 0.1) {
                        if (index === 0) {
                            ctx.beginPath();
                            ctx.moveTo(line.x1, line.y1);
                        } else {
                            ctx.lineTo(line.x1, line.y1);
                        }

                        return true;
                    } else {
                        return false;
                    }
                };
            }*/
        } else {
            drawLine = (v0, v1) => {
                const line = new Line(v0.x, v0.y, v1.x, v1.y);

                if (line.isInsideView(view.width, view.height)) {
                    const mag2 = _calculateMag2ForNoSinProjections(line, view);

                    if (mag2 < 0.1) {
                        line.draw(ctx);
                    }
                }
            };
            if (this.closed && this.fill) {
                fillPoly = (v0, v1, index) => {
                    const line = new Line(v0.x, v0.y, v1.x, v1.y);

                    const mag2 = _calculateMag2ForNoSinProjections(line, view);

                    if (mag2 < 0.1) {
                        if (index === 0) {
                            ctx.beginPath();
                            ctx.moveTo(line.x1, line.y1);
                        } else {
                            ctx.lineTo(line.x1, line.y1);
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

            ctx.globalAlpha = 1;
            ctx.save();
            ctx.fillStyle = this.fillColor;
            ctx.globalAlpha = this.opacity;
            ctx.fill();
            ctx.restore();
        }
    };

    Polyline.prototype.isInStroke = function(ctx, view, x, y) {
        ctx.lineWidth = this.lineWidth;

        let pointXY = [];
        for (var j = 0; j < this.radecArray.length; j++) {
            var xy = view.aladin.world2pix(this.radecArray[j][0], this.radecArray[j][1]);
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
