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
 * File Overlay
 *
 * Description: a plane holding overlays (footprints, polylines, circles)
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Overlay = (function() {
   Overlay = function(options) {
        options = options || {};
    	this.name = options.name || "overlay";
    	this.color = options.color || Color.getNextColor();
        
    	this.lineWidth = options["lineWidth"] || 2;
    	
    	//this.indexationNorder = 5; // à quel niveau indexe-t-on les overlays
    	this.overlays = [];
    	this.overlay_items = []; // currently Circle or Polyline
    	//this.hpxIdx = new HealpixIndex(this.indexationNorder);
    	//this.hpxIdx.init();
    	
    	this.isShowing = true;
    };
    

    // TODO : show/hide methods should be integrated in a parent class 
    Overlay.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };
    
    Overlay.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        this.reportChange();
    };
    
    // return an array of Footprint from a STC-S string
    Overlay.parseSTCS = function(stcs) {
        var polygons = [];
        var parts = stcs.match(/\S+/g);
        var k = 0, len = parts.length;
        var curPolygon;
        while(k<len) {
            var s = parts[k].toLowerCase();
            if(s=='polygon') {
                curPolygon = [];
                k++;
                frame = parts[k].toLowerCase();
                if (frame=='icrs' || frame=='j2000') {
                    while(k+2<len) {
                        var ra = parseFloat(parts[k+1]);
                        if (isNaN(ra)) {
                            break;
                        }
                        var dec = parseFloat(parts[k+2]);
                        curPolygon.push([ra, dec]);
                        k += 2;
                    }
                    curPolygon.push(curPolygon[0]);
                    polygons.push(curPolygon);
                }
            }
            k++;
        }

        return polygons;
    };
    
    // ajout d'un tableau d'overlays (= footprints)
    Overlay.prototype.addFootprints = function(overlaysToAdd) {
    	this.overlays = this.overlays.concat(overlaysToAdd);
    	for (var k=0, len=overlaysToAdd.length; k<len; k++) {
    	    overlaysToAdd[k].setOverlay(this);
    	}
        this.view.requestRedraw();
    };

    // TODO : item doit pouvoir prendre n'importe quoi en param (footprint, circle, polyline)
    Overlay.prototype.add = function(item) {
        this.overlay_items.push(item);
        item.setOverlay(this);
        
        this.view.requestRedraw();
    };

    
    // return a footprint by index
   Overlay.prototype.getFootprint = function(idx) {
        if (idx<this.footprints.length) {
            return this.footprints[idx];
        }
        else {
            return null;
        }
    };
    
    Overlay.prototype.setView = function(view) {
        this.view = view;
    };
    
    Overlay.prototype.removeAll = function() {
        // TODO : RAZ de l'index
        this.overlays = [];
        this.overlay_items = [];
    };
    
    Overlay.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (!this.isShowing) {
            return;
        }
        
        // tracé simple
        ctx.strokeStyle= this.color;

        // 1. Tracé polygons
        
        // TODO: les overlay polygons devrait se tracer lui meme (méthode draw)
        ctx.lineWidth = this.lineWidth;
    	ctx.beginPath();
    	xyviews = [];
    	for (var k=0, len = this.overlays.length; k<len; k++) {
    		xyviews.push(this.drawFootprint(this.overlays[k], ctx, projection, frame, width, height, largestDim, zoomFactor));
    	}
        ctx.stroke();

    	// tracé sélection
        ctx.strokeStyle= Overlay.increase_brightness(this.color, 80);
        //ctx.strokeStyle= 'green';
        ctx.beginPath();
        for (var k=0, len = this.overlays.length; k<len; k++) {
            if (! this.overlays[k].isSelected) {
                continue;
            }
            this.drawFootprintSelected(ctx, xyviews[k]);
            
        }
    	ctx.stroke();
    	
        // 2. Tracé cercles ou polylines
    	for (var k=0; k<this.overlay_items.length; k++) {
    	    this.overlay_items[k].draw(ctx, projection, frame, width, height, largestDim, zoomFactor);
    	}
    };

    Overlay.increase_brightness = function(hex, percent){
        // strip the leading # if it's there
        hex = hex.replace(/^\s*#|\s*$/g, '');

        // convert 3 char codes --> 6, e.g. `E0F` --> `EE00FF`
        if(hex.length == 3){
            hex = hex.replace(/(.)/g, '$1$1');
        }

        var r = parseInt(hex.substr(0, 2), 16),
            g = parseInt(hex.substr(2, 2), 16),
            b = parseInt(hex.substr(4, 2), 16);

        return '#' +
                ((0|(1<<8) + r + (256 - r) * percent / 100).toString(16)).substr(1) +
                ((0|(1<<8) + g + (256 - g) * percent / 100).toString(16)).substr(1) +
                ((0|(1<<8) + b + (256 - b) * percent / 100).toString(16)).substr(1);
    };
    
    
    
    Overlay.prototype.drawFootprint = function(f, ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! f.isShowing) {
            return null;
        }
        var xyviewArray = [];
        var show = false;
        var radecArray = f.polygons;
        // for
            for (var k=0, len=radecArray.length; k<len; k++) {
                var xy;
                if (frame!=CooFrameEnum.J2000) {
                    var lonlat = CooConversion.J2000ToGalactic([radecArray[k][0], radecArray[k][1]]);
                    xy = projection.project(lonlat[0], lonlat[1]);
                }
                else {
                    xy = projection.project(radecArray[k][0], radecArray[k][1]);
                }
                if (!xy) {
                    return null;
                }
                var xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
                xyviewArray.push(xyview);
                if (!show && xyview.vx<width  && xyview.vx>=0 && xyview.vy<=height && xyview.vy>=0) {
                    show = true;
                }
            }

            if (show) {
                ctx.moveTo(xyviewArray[0].vx, xyviewArray[0].vy);
                for (var k=1, len=xyviewArray.length; k<len; k++) {
                    ctx.lineTo(xyviewArray[k].vx, xyviewArray[k].vy);
                }
            }
            else {
                //return null;
            }
        // end for

        return xyviewArray;



    };

    Overlay.prototype.drawFootprintSelected = function(ctx, xyview) {
        if (!xyview) {
            return;
        }

        var xyviewArray = xyview;
        ctx.moveTo(xyviewArray[0].vx, xyviewArray[0].vy);
        for (var k=1, len=xyviewArray.length; k<len; k++) {
            ctx.lineTo(xyviewArray[k].vx, xyviewArray[k].vy);
        }
    };


    
    // callback function to be called when the status of one of the footprints has changed
    Overlay.prototype.reportChange = function() {
        this.view.requestRedraw();
    };

    return Overlay;
})();
