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
 * Class Vector
 * 
 * A vector is a graphical overlay connecting 2 points with end or begin arrows on it
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/
import { Polyline } from "./Polyline.js";
import { Utils } from '../Utils';
import { GraphicOverlay } from "../Overlay.js";
import { Ellipse } from "./Ellipse.js";

export let Vector = (function() {
    /**
     * Represents an vector.
     * 
     * A vector is a graphical overlay connecting 2 sky positions with end or begin arrows on it
     *
     * @class
     * @constructs Vector
     * @param {number} ra1 - Right Ascension (RA) coordinate of the center in degrees.
     * @param {number} dec1 - Declination (Dec) coordinate of the center in degrees.
     * @param {number} ra2 - Right Ascension (RA) coordinate of the center in degrees.
     * @param {number} dec2 - Declination (Dec) coordinate of the center in degrees.
     * @param {ShapeOptions} options - Options for configuring the vector. Additional properties:
     * @param {boolean} [options.arrow=false] - Add an arrow pointing from (ra1, dec1) to (ra2, dec2)
     * 
     * @returns {Vector} - The vector shape object
     */
    let Vector = function(ra1, dec1, ra2, dec2, options) {
        options = options || {};
        this.color     = options['color']     || undefined;
        this.opacity   = options['opacity']   || undefined;
        this.lineWidth = options['lineWidth'] || undefined;
        this.selectionColor = options["selectionColor"] || '#00ff00';
        this.hoverColor = options["hoverColor"] || undefined;
        this.arrow = options["arrow"] === undefined ? false : options["arrow"];

        // All graphics overlay have an id
        this.id = ' vector-' + Utils.uuidv4();

        this.overlay = null;

    	this.isShowing = true;
    	this.isSelected = false;
        this.isHovered = false;

        this.ra1 = ra1;
        this.dec1 = dec1;
        this.ra2 = ra2;
        this.dec2 = dec2;
    };

    Vector.prototype = {
        setOverlay: Polyline.prototype.setOverlay,
        isFootprint: Polyline.prototype.isFootprint,
        show: Polyline.prototype.show,
        hide: Polyline.prototype.hide,
        
        select: Polyline.prototype.select,
        deselect: Polyline.prototype.deselect,
        
        hover: Polyline.prototype.hover,
        unhover: Polyline.prototype.unhover,
        
        getLineWidth: Polyline.prototype.getLineWidth,
        setLineWidth: Polyline.prototype.setLineWidth,

        setColor: Polyline.prototype.setColor,
        setSelectionColor: Polyline.prototype.setSelectionColor,
        setHoverColor: Polyline.prototype.setHoverColor,

        draw: function(ctx, view, noStroke, noSmallCheck) {
            noStroke = noStroke===true || false;
            noSmallCheck = noSmallCheck===true || false;
            // project
            const v1 = view.aladin.world2pix(this.ra1, this.dec1);
            if (!v1)
                return false;
            const v2 = view.aladin.world2pix(this.ra2, this.dec2);
            if (!v2)
                return false;
            
            const xmin = Math.min(v1[0], v2[0]);
            const xmax = Math.max(v1[0], v2[0]);
            const ymin = Math.min(v1[1], v2[1]);
            const ymax = Math.max(v1[1], v2[1]);

            // out of bbox
            if (xmax < 0 || xmin > view.width || ymax < 0 || ymin > view.height) {
                return false;
            }

            let baseColor = this.color || (this.overlay && this.overlay.color) || '#ff0000';
            if (!this.lineWidth) {
                this.lineWidth = (this.overlay && this.overlay.lineWidth) || 2;
            }

            // too small
            if(!noSmallCheck) {
                this.isTooSmall = (xmax - xmin) < 1 && (ymax - ymin) < 1;
                if (this.isTooSmall) {
                    return false;
                }
            }

            if (this.isSelected) {
                ctx.strokeStyle = this.selectionColor || GraphicOverlay.increaseBrightness(baseColor, 50);
            } else if (this.isHovered) {
                ctx.strokeStyle = this.hoverColor || GraphicOverlay.increaseBrightness(baseColor, 25);
            } else {
                ctx.strokeStyle = baseColor;
            }

            ctx.lineWidth = this.lineWidth;
            ctx.globalAlpha = this.opacity;

            ctx.beginPath();
            ctx.moveTo(v1[0], v1[1]);
            ctx.lineTo(v2[0], v2[1]);

            if (this.arrow) {
                // draw the arrow
                var angle, x, y, xh, yh;
                var arrowRad = this.lineWidth * 3;

                angle = Math.atan2(v2[1] - v1[1], v2[0] - v1[0])
                xh = v2[0];
                yh = v2[1];

                var t = angle + Math.PI * 3 / 4;
                x = arrowRad * Math.cos(t) + v2[0];
                y = arrowRad * Math.sin(t) + v2[1];

                ctx.moveTo(x, y);
                ctx.lineTo(xh, yh);

                var t = angle - Math.PI * 3 / 4;
                x = arrowRad *Math.cos(t) + v2[0];
                y = arrowRad *Math.sin(t) + v2[1];

                ctx.lineTo(x, y);
            }

            if (!noStroke) {
                ctx.stroke();
            }

            return true;
        },

        isInStroke: Ellipse.prototype.isInStroke,

        /*Line.prototype.intersectsBBox = function(x, y, w, h) {
            // todo
        };*/
    };

    return Vector;
})();