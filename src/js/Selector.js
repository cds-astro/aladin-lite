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
 * Class Selector
 * 
 * A selector
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

export let Selector = (function() {
    // constructor
    let Selector = function() {};

    Selector.prototype.start = function(startCoo, mode, callbackFn) {
        this.startCoo = { x: startCoo.x, y: startCoo.y};
        this.mode = mode;
        this.callbackFn = callbackFn;
    };

    Selector.prototype.draw = function(ctx, dragCoo) {
        ctx.fillStyle = "rgba(100, 240, 110, 0.20)";
        ctx.strokeStyle = "rgb(100, 240, 110)";
        ctx.lineWidth = 2;
        switch (this.mode) {
            case 'rect':
                var w = dragCoo.x - this.startCoo.x;
                var h = dragCoo.y - this.startCoo.y;

                ctx.fillRect(this.startCoo.x, this.startCoo.y, w, h);
                ctx.strokeRect(this.startCoo.x, this.startCoo.y, w, h);
                break;
            case 'circle':
                var r2 = (dragCoo.x - this.startCoo.x) * (dragCoo.x - this.startCoo.x) + (dragCoo.y - this.startCoo.y) * (dragCoo.y - this.startCoo.y);
                var r = Math.sqrt(r2);

                ctx.beginPath();
                ctx.arc(this.startCoo.x, this.startCoo.y, r, 0, 2 * Math.PI);
                ctx.fill();
                ctx.stroke();
                break;
            default:
                break;
        }
    };

    Selector.prototype.finish = function(dragCoo) {
        switch (this.mode) {
            case 'rect':
                var w = dragCoo.x - this.startCoo.x;
                var h = dragCoo.y - this.startCoo.y;

                (typeof this.callbackFn === 'function') && this.callbackFn({
                    x: this.startCoo.x,
                    y: this.startCoo.y,
                    w: w,
                    h: h
                });
                break;
            case 'circle':
                var r2 = (dragCoo.x - this.startCoo.x) * (dragCoo.x - this.startCoo.x) + (dragCoo.y - this.startCoo.y) * (dragCoo.y - this.startCoo.y);
                var r = Math.sqrt(r2);

                (typeof this.callbackFn === 'function') && this.callbackFn({
                    x: this.startCoo.x,
                    y: this.startCoo.y,
                    r: r,
                });
                break;
            default:
                break;
        }
    }

    Selector.prototype.getBBox = function(dragCoo) {
        let x, y, w, h;
        switch (this.mode) {
            case 'rect':
                w = dragCoo.x - this.startCoo.x;
                h = dragCoo.y - this.startCoo.y;
                x = this.startCoo.x;
                y = this.startCoo.y;
                break;
            case 'circle':
                var r2 = (dragCoo.x - this.startCoo.x) * (dragCoo.x - this.startCoo.x) + (dragCoo.y - this.startCoo.y) * (dragCoo.y - this.startCoo.y);
                var r = Math.sqrt(r2);

                x = this.startCoo.x - r;
                y = this.startCoo.y - r;
                w = 2 * r;
                h = 2 * r;
                break;
            default:
                break;
        }

        return {x: x, y: y, w: w, h: h};
    };

    return Selector;
})();