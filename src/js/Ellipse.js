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
 * File Ellipse
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { Utils } from "./Utils";
import { Overlay } from "./Overlay.js";

/**
* @typedef {Object} ShapeOptions
* @description Options for describing a shape
*
* @property {Object} options - Configuration options for the shape.
* @property {string} [options.color] - The color of the shape
* @property {string} [options.fill=false] - Fill the shape with fillColor
* @property {string} [options.fillColor] - A filling color for the shape
* @property {number} [options.lineWidth=2] - The line width in pixels
* @property {number} [options.opacity=1] - The opacity, between 0 (totally transparent) and 1 (totally opaque)
* @property {string} [options.selectionColor='#00ff00'] - A selection color
* @property {string} [options.hoverColor] -  A hovered color
*/

/**
 * Represents an ellipse shape
 *
 * @namespace
 * @typedef {Object} Ellipse
 */
export let Ellipse = (function() {
    /**
     * Constructor function for creating a new ellipse.
     *
     * @constructor
     * @memberof Ellipse
     * @param {number[]} center - right-ascension/declination 2-tuple of the ellipse's center in degrees
     * @param {number} a - semi-major axis length in degrees
     * @param {number} b - semi-minor axis length in degrees
     * @param {number} theta - angle of the ellipse in degrees
     * @param {ShapeOptions} options - Configuration options for the ellipse
     * 
     * @returns {Ellipse} - The ellipse shape object
     */
    let Ellipse = function(center, a, b, theta, options) {
        options = options || {};

        this.color = options['color'] || undefined;
        this.fillColor = options['fillColor'] || undefined;
        this.lineWidth = options["lineWidth"] || 2;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;
        this.opacity   = options['opacity']   || 1;

        // TODO : all graphic overlays should have an id
        this.id = 'ellipse-' + Utils.uuidv4();

        this.setCenter(center);
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
        if (this.overlay) {
            this.overlay.reportChange();
        }
    }

    Ellipse.prototype.unhover = function() {
        if (! this.isHovered) {
            return;
        }
        this.isHovered = false;
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

    // TODO
    Ellipse.prototype.draw = function(ctx, view, noStroke) {
        if (! this.isShowing) {
            return;
        }

        let px_per_deg = view.width / view.fov;

        /*if (this.a * 2 * px_per_deg < this.lineWidth || this.b * 2 * px_per_deg < this.lineWidth) {
            return;
        }*/

        var originScreen = view.aladin.world2pix(this.centerRaDec[0], this.centerRaDec[1]);
        if (!originScreen) {
            // the center goes out of the projection
            // we do not draw it
            return;
        }

        // 1. Find the spherical tangent vector going to the north
        let toNorth = [this.centerRaDec[0], this.centerRaDec[1] + 1e-3];

        // 2. Project it to the screen
        let toNorthScreen = view.aladin.world2pix(toNorth[0], toNorth[1]);

        if(!toNorthScreen) {
            return;
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
        //let ct = Math.cos(theta);
        //let st = Math.sin(theta);

        /*let circlePtXyViewRa = view.aladin.world2pix(view.viewCenter.lon + 1.0, view.viewCenter.lat);
        let circlePtXyViewDec = view.aladin.world2pix(view.viewCenter.lon, view.viewCenter.lat + 1.0);

        if (!circlePtXyViewRa || !circlePtXyViewDec) {
            // the circle border goes out of the projection
            // we do not draw it
            return;
        }

        var dxRa = circlePtXyViewRa[0] - centerXyview[0];
        var dyRa = circlePtXyViewRa[1] - centerXyview[1];
        var dRa = Math.sqrt(dxRa*dxRa + dyRa*dyRa);

        var dxDec = circlePtXyViewDec[0] - centerXyview[0];
        var dyDec = circlePtXyViewDec[1] - centerXyview[1];
        var dDec = Math.sqrt(dxDec*dxDec + dyDec*dyDec);*/

        //var radiusInPixX = Math.abs(this.a * ct * dRa) + Math.abs(this.a * st * dDec);
        //var radiusInPixY = Math.abs(this.b * st * dRa) + Math.abs(this.b * ct * dDec);

        // Ellipse crossing the projection
        /*if ((dxRa*dyDec - dxDec*dyRa) <= 0.0) {
            // We do not draw it
            return;
        }*/
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
                ctx.strokeStyle = Overlay.increaseBrightness(baseColor, 50);
            }
        } else if (this.isHovered) {
            ctx.strokeStyle = this.hoverColor || Overlay.increaseBrightness(baseColor, 25);
        } else {
            ctx.strokeStyle = baseColor;
        }

        ctx.lineWidth = this.lineWidth;
        ctx.globalAlpha = this.opacity;
        ctx.beginPath();

        ctx.ellipse(originScreen[0], originScreen[1], px_per_deg * this.a, px_per_deg * this.b, theta, 0, 2*Math.PI, false);
        if (!noStroke) {
            if (this.fillColor) {
                ctx.fillStyle = this.fillColor;
                ctx.fill();
            }
            ctx.stroke();
        }
    };

    Ellipse.prototype.isInStroke = function(ctx, view, x, y) {
        this.draw(ctx, view, true);
        return ctx.isPointInStroke(x, y);
    };

    Ellipse.prototype.intersectsBBox = function(x, y, w, h) {
        // todo
    };
    
    return Ellipse;
})();
