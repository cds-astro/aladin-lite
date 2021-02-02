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
 * Class Line
 * 
 * A line is a graphical overlay connecting 2 points
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

export let Line = (function() {
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