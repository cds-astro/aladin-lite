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
    
    Polyline.prototype.draw = function(ctx, view, frame, width, height, largestDim, zoomFactor) {
        if (! this.isShowing) {
            return;
        }

        if (! this.radecArray || this.radecArray.length<2) {
            return;
        }

        if (this.color) {
            ctx.strokeStyle= this.color;
        }

        var xyviewArray = [];
        for (var k=0, len=this.radecArray.length; k<len; k++) {
            var xyview = AladinUtils.radecToViewXy(this.radecArray[k][0], this.radecArray[k][1], view);
            if (!xyview) {
                return;
            }

            xyviewArray.push(xyview);
        }
        
        ctx.lineWidth = this.lineWidth;
        ctx.beginPath();
        ctx.moveTo(xyviewArray[0][0], xyviewArray[0][1]);
        for (var k=0, len=xyviewArray.length-1; k<len; k++) {
            const line = new Line(xyviewArray[k][0], xyviewArray[k][1], xyviewArray[k+1][0], xyviewArray[k+1][1]);
            if (line.isInsideView(width, height)) {
                ctx.lineTo(xyviewArray[k+1][0], xyviewArray[k+1][1]);
            } else {
                ctx.moveTo(xyviewArray[k+1][0], xyviewArray[k+1][1]);
            }
        }
        ctx.stroke();
    };

    return Polyline;
})();