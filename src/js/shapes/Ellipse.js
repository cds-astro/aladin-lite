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
 * File Ellipse
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

import { Utils } from "./../Utils";
import { GraphicOverlay } from "./../Overlay.js";


export let Ellipse = (function() {
    /**
     * Constructor function for creating a new ellipse.
     *
     * @class
     * @constructs Ellipse
     * @param {number[]} centerRaDec - right-ascension/declination 2-tuple of the ellipse's center in degrees
     * @param {number} a - semi-major axis length in degrees
     * @param {number} b - semi-minor axis length in degrees
     * @param {number} theta - angle of the ellipse in degrees. Origin aligns the ellipsis' major axis with the north pole. Positive angle points towards the east.
     * @param {ShapeOptions} [options] - Configuration options for the ellipse
     * @param {boolean} [options.drawAxes] - Whether to show the semi-major and semi-minor axes in dashed
     * @returns {Ellipse} - The ellipse shape object
     */
    let Ellipse = function(centerRaDec, a, b, theta, options) {
        options = options || {};

        this.color = options['color'] || undefined;
        this.fillColor = options['fillColor'] || undefined;
        this.lineWidth = options["lineWidth"] || undefined;
        this.selectionLineWidth = options["selectionLineWidth"] || undefined;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;
        this.opacity   = options['opacity']   || 1;
        this.drawAxes   = options['drawAxes'] || undefined;

        // TODO : all graphic overlays should have an id
        this.id = 'ellipse-' + Utils.uuidv4();

        this.setCenter(centerRaDec);
        this.setAxisLength(a, b);
        this.setRotation(theta);
    	this.overlay = null;

    	this.isShowing = true;
        this.isSelected = false;
        this.isHovered = false;
    };

    Ellipse.prototype.setColor = function(color) {
        if (!color || this.color == color) {
            return;
        }
        this.color = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setSelectionColor = function(color) {
        if (!color || this.selectionColor == color) {
            return;
        }
        this.selectionColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setHoverColor = function(color) {
        if (!color || this.hoverColor == color) {
            return;
        }
        this.hoverColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setLineWidth = function(lineWidth) {
        if (this.lineWidth == lineWidth) {
            return;
        }
        this.lineWidth = lineWidth;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.getLineWidth = function() {
        return this.lineWidth;
    };

    Ellipse.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };

    Ellipse.prototype.setSelectionLineWidth = function(selectionLineWidth) {
        if (this.selectionLineWidth == selectionLineWidth) {
            return;
        }
        this.selectionLineWidth = selectionLineWidth;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.getSelectionLineWidth = function() {
        return this.selectionLineWidth;
    };

    Ellipse.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };


    Ellipse.prototype.hover = function() {
        if (this.isHovered) {
            return;
        }
        this.isHovered = true;
        this.setLineWidth(this.getLineWidth() + 2)
        this.setSelectionLineWidth(this.getSelectionLineWidth() + 2)
        if (this.overlay) {
            this.overlay.reportChange();
        }
    }

    Ellipse.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;
        this.setLineWidth(this.getLineWidth() - 2)
        this.setSelectionLineWidth(this.getSelectionLineWidth() - 2)
        if (this.overlay) {
            this.overlay.reportChange();
        }
    }

    Ellipse.prototype.setCenter = function(centerRaDec) {
        this.centerRaDec = centerRaDec;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setRotation = function(rotationDegrees) {
        // radians
        let theta = rotationDegrees * Math.PI / 180;
        this.rotation = theta;
        // rotation in clockwise in the 2d canvas
        // we must transform it so that it is a north to east rotation
        //this.rotation = -theta - Math.PI/2;

        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setAxisLength = function(a, b) {
        this.a = a;
        this.b = b;

        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.isFootprint = function() {
        return true;
    }

    Ellipse.prototype.draw = function(ctx, view, noStroke, noSmallCheck) {
        if (! this.isShowing) {
            return false;
        }

        // Decide which line width to use.
        if (!this.lineWidth) {
            this.lineWidth = (this.overlay && this.overlay.lineWidth) || 2;
        }
        let drawingLineWidth = this.lineWidth;
        if (this.isSelected && this.selectionLineWidth) {
            drawingLineWidth = this.selectionLineWidth;
        }

        const px_per_deg = view.width / view.fov;
        noSmallCheck = noSmallCheck===true || false;
        if (!noSmallCheck) {
            this.isTooSmall = this.b * 2 * px_per_deg < drawingLineWidth;
            if (this.isTooSmall) {
                return false;
            }
        }

        var originScreen = view.aladin.world2pix(this.centerRaDec[0], this.centerRaDec[1]);
        if (!originScreen) {
            // the center goes out of the projection
            // we do not draw it
            return false;
        }

        // 1. Find the spherical tangent vector going to the north
        let toNorth = [this.centerRaDec[0], this.centerRaDec[1] + 1e-3];

        // 2. Project it to the screen
        let toNorthScreen = view.aladin.world2pix(toNorth[0], toNorth[1]);

        if(!toNorthScreen) {
            return false;
        }

        // 3. normalize this vector
        let toNorthVec = [toNorthScreen[0] - originScreen[0], toNorthScreen[1] - originScreen[1]];
        let norm = Math.sqrt(toNorthVec[0]*toNorthVec[0] + toNorthVec[1]*toNorthVec[1]);

        toNorthVec = [toNorthVec[0] / norm, toNorthVec[1] / norm];
        let toWestVec = [1.0, 0.0];

        let x1 = toWestVec[0];
        let y1 = toWestVec[1];
        let x2 = toNorthVec[0];
        let y2 = toNorthVec[1];
        // 4. Compute the west to north angle
        let westToNorthAngle = Math.atan2(x1*y2-y1*x2, x1*x2+y1*y2);

        // 5. Get the correct ellipse angle
        let theta = -this.rotation + westToNorthAngle;

        noStroke = noStroke===true || false;

        // TODO : check each 4 point until show
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
                ctx.strokeStyle = GraphicOverlay.increaseBrightness(baseColor, 50);
            }
        } else if (this.isHovered) {
            ctx.strokeStyle = this.hoverColor || GraphicOverlay.increaseBrightness(baseColor, 25);
        } else {
            ctx.strokeStyle = baseColor;
        }

        ctx.lineWidth = drawingLineWidth;
        ctx.globalAlpha = this.opacity;
        ctx.beginPath();

        this.aPixels = px_per_deg * this.a;
        ctx.ellipse(originScreen[0], originScreen[1], px_per_deg * this.a, px_per_deg * this.b, theta, 0, 2*Math.PI, false);
        if (!noStroke) {
            if (this.fillColor) {
                ctx.fillStyle = this.fillColor;
                ctx.fill();
            }

            ctx.stroke();

            if (this.drawAxes === true) {
                let getVertexOnEllipse = (t) => {
                    let ax = px_per_deg * this.a * Math.cos(theta);
                    let ay = px_per_deg * this.a * Math.sin(theta);
                    let bx = -px_per_deg * this.b * Math.sin(theta);
                    let by = px_per_deg * this.b * Math.cos(theta);

                    let X = originScreen[0] + ax * Math.cos(t) + bx * Math.sin(t);
                    let Y = originScreen[1] + ay * Math.cos(t) + by * Math.sin(t);

                    return [X, Y]
                }

                let [xa, ya] = getVertexOnEllipse(Math.PI * 0.5)
                let [xb, yb] = getVertexOnEllipse(3 * Math.PI * 0.5)
                let [xc, yc] = getVertexOnEllipse(Math.PI)
                let [xd, yd] = getVertexOnEllipse(0)
                ctx.lineWidth = Math.max(this.lineWidth * 0.5, 1.0);
                ctx.setLineDash([this.lineWidth, this.lineWidth]);

                ctx.moveTo(xa, ya);
                ctx.lineTo(xb, yb);
                ctx.moveTo(xc, yc);
                ctx.lineTo(xd, yd);

                ctx.stroke();
            }
        }

        return true;
    };

    Ellipse.prototype.isInStroke = function(ctx, view, x, y) {
        if (!this.draw(ctx, view, true)) {
            return false;
        }

        return ctx.isPointInStroke(x, y);
    };

    Ellipse.prototype.intersectsBBox = function(x, y, w, h, view) {
        // TODO: currently the same as Circle where radius = a.
        var centerXyview = view.aladin.world2pix(this.centerRaDec[0], this.centerRaDec[1]);
        if (!centerXyview) {
            return false;
        }

        // compute the absolute distance between the middle of the bbox
        // and the center of the circle
        const circleDistance = {
            x: Math.abs(centerXyview[0] - (x + w/2)),
            y: Math.abs(centerXyview[1] - (y + h/2))
        };

        if (circleDistance.x > (w/2 + this.aPixels)) { return false; }
        if (circleDistance.y > (h/2 + this.aPixels)) { return false; }

        if (circleDistance.x <= (w/2)) { return true; }
        if (circleDistance.y <= (h/2)) { return true; }

        const dx = circleDistance.x - w/2;
        const dy = circleDistance.y - h/2;

        const cornerDistanceSquared = dx*dx + dy*dy;
        return (cornerDistanceSquared <= (this.aPixels*this.aPixels));
    };

    return Ellipse;
})();
