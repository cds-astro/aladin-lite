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

// TODO : Circle and Footprint should inherit from the same root object
Circle = (function() {
    // constructor
    Circle = function(centerRaDec, radiusDegrees, options) {
        options = options || {};
        
        this.color = options['color'] || undefined;

        // TODO : all graphic overlays should have an id
        this.id = 'circle-' + Utils.uuidv4();

        this.setCenter(centerRaDec);
        this.setRadius(radiusDegrees);
    	this.overlay = null;
    	
    	this.isShowing = true;
    	this.isSelected = false;
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
    
    Circle.prototype.dispatchClickEvent = function() {
        if (this.overlay) {
            // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
            //window.dispatchEvent(new CustomEvent("footprintClicked", {
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
        }
    };
    
    Circle.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
/*
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
*/

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
    Circle.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor, noStroke) {
        if (! this.isShowing) {
            return;
        }


        noStroke = noStroke===true || false;

        var centerXy;
        if (frame.system != CooFrameEnum.SYSTEMS.J2000) {
            var lonlat = CooConversion.J2000ToGalactic([this.centerRaDec[0], this.centerRaDec[1]]);
            centerXy = projection.project(lonlat[0], lonlat[1]);
        }
        else {
            centerXy = projection.project(this.centerRaDec[0], this.centerRaDec[1]);
        }
        if (!centerXy) {
            return;
        }
        var centerXyview = AladinUtils.xyToView(centerXy.X, centerXy.Y, width, height, largestDim, zoomFactor, false);

        // compute value of radius in pixels in current projection
        var circlePtXy;
        var ra = this.centerRaDec[0];
        var dec = this.centerRaDec[1] + (ra>0 ? - this.radiusDegrees : this.radiusDegrees);
        if (frame.system != CooFrameEnum.SYSTEMS.J2000) {
            var lonlat = CooConversion.J2000ToGalactic([ra, dec]);
            circlePtXy = projection.project(lonlat[0], lonlat[1]);
        }
        else {
            circlePtXy = projection.project(ra, dec);
        }
        if (!circlePtXy) {
            return;
        }
        var circlePtXyView = AladinUtils.xyToView(circlePtXy.X, circlePtXy.Y, width, height, largestDim, zoomFactor, false);
        var dx = circlePtXyView.vx - centerXyview.vx;
        var dy = circlePtXyView.vy - centerXyview.vy;
        var radiusInPix = Math.sqrt(dx*dx + dy*dy);

        // TODO : check each 4 point until show
        var baseColor = this.color;
        if (! baseColor && this.overlay) {
            baseColor = this.overlay.color;
        }
        if (! baseColor) {
            baseColor = '#ff0000';
        }
        
        if (this.isSelected) {
            ctx.strokeStyle= Overlay.increaseBrightness(baseColor, 50);
        }
        else {
            ctx.strokeStyle= baseColor;
        }

        ctx.beginPath();
        ctx.arc(centerXyview.vx, centerXyview.vy, radiusInPix, 0, 2*Math.PI, false);
        if (!noStroke) {
            ctx.stroke();
        }
    }; 
    
    return Circle;
})();
