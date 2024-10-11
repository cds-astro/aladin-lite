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
 * File Circle
 *
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import { Utils } from "./../Utils";
import { GraphicOverlay } from "./../Overlay.js";

export let Circle = (function() {
    /**
     * Constructor function for creating a new circle.
     *
     * @class
     * @constructs Circle
     * @param {number[]} centerRaDec - right-ascension/declination 2-tuple of the circle's center in degrees
     * @param {number} radius - radius in degrees
     * @param {ShapeOptions} options - Configuration options for the circle
     * 
     * @returns {Circle} - The circle shape object
     */
    let Circle = function(centerRaDec, radius, options) {
        options = options || {};

        this.color     = options['color']     || undefined;
        this.fillColor = options['fillColor'] || undefined;
        this.lineWidth = options["lineWidth"] || 2;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;

        // TODO : all graphic overlays should have an id
        this.id = 'circle-' + Utils.uuidv4();

        this.setCenter(centerRaDec);
        this.setRadius(radius);
    	this.overlay = null;

    	this.isShowing = true;
    	this.isSelected = false;
        this.isHovered = false;
        this.frame = options.frame || "icrs";
    };

    Circle.prototype.setColor = function(color) {
        if (!color || this.color == color) {
            return;
        }
        this.color = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.setSelectionColor = function(color) {
        if (!color || this.selectionColor == color) {
            return;
        }
        this.selectionColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.setHoverColor = function(color) {
        if (!color || this.hoverColor == color) {
            return;
        }
        this.hoverColor = color;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.setLineWidth = function(lineWidth) {
        if (this.lineWidth == lineWidth) {
            return;
        }
        this.lineWidth = lineWidth;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.getLineWidth = function() {
        return this.lineWidth;
    };

    Circle.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };

    Circle.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.hover = function() {
        if (this.isHovered) {
            return;
        }
        this.isHovered = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    }

    Circle.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    }

    Circle.prototype.isFootprint = function() {
        return true;
    }

    Circle.prototype.setCenter = function(centerRaDec) {
        this.centerRaDec = centerRaDec;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Circle.prototype.setRadius = function(radiusDegrees) {
        this.radiusDegrees = radiusDegrees;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    // TODO
    Circle.prototype.draw = function(ctx, view, noStroke, noSmallCheck) {
        if (! this.isShowing) {
            return false;
        }

        noSmallCheck = noSmallCheck===true || false;
        if (!noSmallCheck) {
            const px_per_deg = view.width / view.fov;
            this.isTooSmall = this.radiusDegrees * 2 * px_per_deg < this.lineWidth;
            if (this.isTooSmall) {
                return false;
            }
        }

        noStroke = noStroke===true || false;

        var centerXyview = view.aladin.world2pix(this.centerRaDec[0], this.centerRaDec[1]);
        if (!centerXyview) {
            // the center goes out of the projection
            // we do not draw it
            return false;
        }
        this.center = {
            x: centerXyview[0],
            y: centerXyview[1],
        };

        let hidden = true;

        var ra, dec, vertOnCircle, dx, dy;
        this.radius = Number.NEGATIVE_INFINITY;
        
        // Project 4 points lying on the circle and take the minimal dist with the center as radius
        [[-1, 0], [1, 0], [0, -1], [0, 1]].forEach(([cardDirRa, cardDirDec]) => {
            ra = this.centerRaDec[0] + cardDirRa * this.radiusDegrees;
            dec = this.centerRaDec[1] + cardDirDec * this.radiusDegrees;

            vertOnCircle = view.aladin.world2pix(ra, dec);

            if (vertOnCircle) {
                dx = vertOnCircle[0] - this.center.x;
                dy = vertOnCircle[1] - this.center.y;

                this.radius = Math.max(Math.sqrt(dx*dx + dy*dy), this.radius);

                hidden = false;
            }            
        });

        if (hidden) {
            return false;
        }
        // Then we can draw

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

        ctx.lineWidth = this.lineWidth;
        ctx.beginPath();
        ctx.arc(this.center.x, this.center.y, this.radius, 0, 2*Math.PI, false);
        if (!noStroke) {
            if (this.fillColor) {
                ctx.fillStyle = this.fillColor;
                ctx.fill();
            }
            ctx.stroke();
        }

        return true;
    };

    Circle.prototype.isInStroke = function(ctx, view, x, y) {
        this.draw(ctx, view, true);
        return ctx.isPointInStroke(x, y);
    };

    // From StackOverflow: https://stackoverflow.com/questions/401847/circle-rectangle-collision-detection-intersection
    Circle.prototype.intersectsBBox = function(x, y, w, h) {
        const circleDistance = {
            x: Math.abs(this.center.x - x),
            y: Math.abs(this.center.y - y)
        };

        if (circleDistance.x > (w/2 + this.radius)) { return false; }
        if (circleDistance.y > (h/2 + this.radius)) { return false; }

        if (circleDistance.x <= (w/2)) { return true; } 
        if (circleDistance.y <= (h/2)) { return true; }

        const dx = circleDistance.x - w/2;
        const dy = circleDistance.y - h/2;

        const cornerDistanceSquared = dx*dx + dy*dy;
        return (cornerDistanceSquared <= (this.radius*this.radius));
    }

    return Circle;
})();
