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

export let Polyline= (function() {
    // constructor
    let Polyline = function(radecArray, options) {
        options = options || {};
        this.color = options['color'] || undefined;
        
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
    
    Polyline.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor, view) {
        if (! this.isShowing) {
            return;
        }

        if (! this.radecArray || this.radecArray.length<2) {
            return;
        }
        
        if (this.color) {
            ctx.strokeStyle= this.color;
        }
        /*var start = AladinUtils.radecToViewXy(this.radecArray[0][0], this.radecArray[0][1], projection, frame, width, height, largestDim, zoomFactor);
        if (! start) {
            return;
        }
       
        ctx.beginPath();
        ctx.moveTo(start.vx, start.vy);
        var pt;
        for (var k=1; k<this.radecArray.length; k++) {
            pt = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], projection, frame, width, height, largestDim, zoomFactor);
            if (!pt) {
                break;
            }
            ctx.lineTo(pt.vx, pt.vy);
        }
        
        
        ctx.stroke();*/
        ctx.beginPath();
        for(var l=0; l<this.radecArray.length-1; l++) {
            let pts = view.aladin.webglAPI.projectLine(this.radecArray[l][0], this.radecArray[l][1], this.radecArray[l+1][0], this.radecArray[l+1][1]);
            for(var k=0; k<pts.length; k+=4) {
                let line = new Line(pts[k], pts[k+1], pts[k+2], pts[k+3]);
                if (line.isInsideView(width, height)) {
                    line.draw(ctx);
                }    
            }
        }

        ctx.stroke();
    };

    return Polyline;
})();

let Line = (function() {
    // constructor
    let Line = function(x1, y1, x2, y2) {
        this.x1 = x1;
        this.y1 = y1;
        this.x2 = x2;
        this.y2 = y2;
    };

    // Method for testing whether a line is inside the view
    // http://www.jeffreythompson.org/collision-detection/line-rect.php
    Line.prototype.isInsideView = function(rw, rh) {
        if (this.x1 >= 0 && this.x1 <= rw && this.y1 >= 0 && this.y1 <= rh) {
            return true;
        }
        if (this.x2 >= 0 && this.x2 <= rw && this.y2 >= 0 && this.y2 <= rh) {
            return true;
        }

        // check if the line has hit any of the rectangle's sides
        // uses the Line/Line function below
        let left =   Line.intersectLine(this.x1, this.y1, this.x2, this.y2, 0, 0, 0, rh);
        let right =  Line.intersectLine(this.x1, this.y1, this.x2, this.y2, rw, 0, rw, rh);
        let top =    Line.intersectLine(this.x1, this.y1, this.x2, this.y2, 0, 0, rw, 0);
        let bottom = Line.intersectLine(this.x1, this.y1, this.x2, this.y2, 0, rh, rw, rh);
    
        // if ANY of the above are true, the line
        // has hit the rectangle
        if (left || right || top || bottom) {
            return true;
        }

        return false;
    };

    Line.prototype.draw = function(ctx) {
        ctx.moveTo(this.x1, this.y1);
        ctx.lineTo(this.x2, this.y2);
    };

    Line.intersectLine = function(x1, y1, x2, y2, x3, y3, x4, y4) {
        // Calculate the direction of the lines
        let uA = ((x4-x3)*(y1-y3) - (y4-y3)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
        let uB = ((x2-x1)*(y1-y3) - (y2-y1)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
    
        // If uA and uB are between 0-1, lines are colliding
        if (uA >= 0 && uA <= 1 && uB >= 0 && uB <= 1) {
            return true;
        }
        return false;
    };

    return Line;
})();