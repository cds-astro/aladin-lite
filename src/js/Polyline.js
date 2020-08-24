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

Polyline= (function() {
    // constructor
    Polyline = function(radecArray, options) {
        options = options || {};
        this.color = options['color'] || "white";
        this.lineWidth = options["lineWidth"] || 2;
        
        this.radecArray = radecArray;
        this.overlay = null;
    	
    	this.isShowing = true;
    	this.isSelected = false;
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
        this.overlay.reportChange();
    };

    Polyline.prototype.setColor = function(color) {
        if (this.color == color) {
            return;
        }
        this.color = color;
        this.overlay.reportChange();
    };
    
    Polyline.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! this.isShowing) {
            return;
        }

        if (! this.radecArray || this.radecArray.length<2) {
            return;
        }
        
        if (this.color) {
            ctx.strokeStyle= this.color;
        }
        var start = AladinUtils.radecToViewXy(this.radecArray[0][0], this.radecArray[0][1], projection, frame, width, height, largestDim, zoomFactor);

        for (var k = 0; k < this.radecArray.length; k++) {
            start = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], projection, frame, width, height, largestDim, zoomFactor);
            if (start) {
                break;
            }
        }
        if (!start) {
            return;
        }
      
        ctx.moveTo(start.vx, start.vy);
        var pt;
        var newSeg = false;
        var drawingNewSeg = true;
        for (var k = 1; k < this.radecArray.length; k++) {
            pt = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], projection, frame, width, height, largestDim, zoomFactor);
            if (!pt) {
                if (drawingNewSeg) {
                    //console.log("closing segment");
                    ctx.stroke();
                }
                drawingNewSeg = false;
                newSeg = true;
            } else {
                if (newSeg) {
                    //console.log ("starting newSeg at "+pt.vx+" "+pt.vy);
                    drawingNewSeg = true;
                    ctx.beginPath();
                    ctx.lineWidth = this.lineWidth;
                    ctx.moveTo(pt.vx, pt.vy);
                    newSeg = false;
                } else {
                    ctx.lineTo(pt.vx, pt.vy);
                }
            }

        }
        ctx.stroke();
    };
    
    return Polyline;
})();
