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
 * File CooGrid
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

CooGrid = (function() {
    var CooGrid = function() {
    };
    
    function viewxy2lonlat(projection, vx, vy, width, height, largestDim, zoomFactor) {
        var xy = AladinUtils.viewToXy(vx, vy, width, height, largestDim, zoomFactor);
        var lonlat;
        try {
            lonlat = projection.unproject(xy.x, xy.y);
        }
        catch(err) {
            return null;
        }
        return {lon: lonlat.ra, lat: lonlat.dec};
    };
    
    var NB_STEPS = 50;

    CooGrid.prototype.redraw = function(ctx, projection, cooFrame, width, height, largestDim, zoomFactor, viewCenter, fov) {
        ctx.save();

     	var fovLat = fov * height / width;
     	
     	a = Math.abs(viewCenter.lat)/90;
     	size = a*fov + (1-a)*fovLat;
		var latMin =  Math.max(viewCenter.lat - size * 0.5, -90);
		var latMax =  Math.min(viewCenter.lat + size * 0.5, 90);
		var lonMin, lonMax;
		
		if(Math.abs(viewCenter.lat) + fovLat/2 > 90.0 ){
			lonMin = viewCenter.lon - 179;
			lonMax = viewCenter.lon + 180;
		}else{
			dist2Pole = 90 - Math.abs(viewCenter.lat) - fovLat / 2;
			ratio = fov / (2 * dist2Pole)
			angle = Math.atan(ratio) * 180 / Math.PI;
			change = Math.max(fov / 2, angle) * 1.2; 
			lonMin = viewCenter.lon - change;
			lonMax = viewCenter.lon + change;
		}
        
        var lonDiff = lonMax - lonMin;
        var latDiff = latMax - latMin;
        
        if(lonDiff > 359){
       		lonMin = lonMax-360;
       		lonDiff = lonMax - lonMin;
       	}

        var LON_STEP, LAT_STEP;
        if(lonDiff > 180){
            LON_STEP = 30;
        }else if(lonDiff > 100){
            LON_STEP = 20;
        }else if(lonDiff > 20){
            LON_STEP = 5;
        }else if(lonDiff > 5){
            LON_STEP = 1;
        }else if(lonDiff > 1){
            LON_STEP = 0.25;
        }else if(lonDiff > 0.2){
        	if(cooFrame == CooFrameEnum.J2000){
            	LON_STEP = .25/6;
        	}else{
	            LON_STEP = 0.05;
        	}
        }else{
        	if(cooFrame == CooFrameEnum.J2000){
            	LON_STEP = .25/30;
        	}else{
	            LON_STEP = 0.01;
        	}
        }
        
        if(latDiff > 180){
            LAT_STEP = 30;
        }else if(latDiff > 50){
            LAT_STEP = 20;
        }else if(latDiff > 15){
            LAT_STEP = 5;
        }else if(latDiff > 5){
            LAT_STEP = 1;
        }else if(latDiff > .6){
            LAT_STEP = 0.25;
        }else if(latDiff > 0.2){
            LAT_STEP = 0.05;
        }else{
	        if(cooFrame == CooFrameEnum.J2000){
	            LAT_STEP = 1/60;
	        }else{
	            LAT_STEP = 0.01;
	        }
        }

        var lonStart = lonMin - (lonMin % LON_STEP) ;
        var lonEnd = Math.min(lonMax - (lonMax % LON_STEP) + LON_STEP, lonStart + 360);
        var latStart = Math.max(latMin - (latMin % LAT_STEP) - LAT_STEP, -90);
        var latEnd = Math.min(latMax - (latMax % LAT_STEP) + LAT_STEP, 90);

        var lonDiff = lonEnd - lonStart;
        var latDiff = latEnd - latStart;
        
        ctx.lineWidth = 1;
        var alpha = 0.5;
    	ctx.globalAlpha = alpha;
		ctx.font = "12px Arial"
    	mainColour = "rgb(120,255,200)";
    	secondColour = "black";
    	
       	this.drawLines(ctx, "lon", false, 0, 360, lonStart, lonEnd, LON_STEP, latStart, latEnd, latDiff/NB_STEPS, secondColour, projection, width, height, largestDim, zoomFactor, cooFrame);
        this.drawLines(ctx, "lon", true, 0, 360, lonStart, lonEnd, LON_STEP, latStart, latEnd, latDiff/NB_STEPS, mainColour, projection, width, height, largestDim, zoomFactor, cooFrame);
	  	this.drawLines(ctx, "lat", false, -90, 90, latStart, latEnd, LAT_STEP, lonStart, lonEnd, lonDiff/NB_STEPS, secondColour, projection, width, height, largestDim, zoomFactor, cooFrame);
        this.drawLines(ctx, "lat", true, -90, 90, latStart, latEnd, LAT_STEP, lonStart, lonEnd, lonDiff/NB_STEPS, mainColour, projection, width, height, largestDim, zoomFactor, cooFrame);
     
    	
        ctx.restore()
    };

	CooGrid.prototype.drawLines = function(ctx, dir, isMain, textMin, textMax, dim1Start, dim1End, dim1Step, dim2Start, dim2End, dim2Step, colour, projection, width, height, largestDim, zoomFactor, cooFrame) {
		var alpha = ctx.globalAlpha;
		ctx.strokeStyle = colour;
		if(isMain){
			ctx.lineWidth = 2;
		}else{
			ctx.lineWidth = 3;
		}
		
		for (var dim1 = dim1Start; dim1 < dim1End + dim1Step; dim1 += dim1Step) {
			if(dim1 > dim1End){dim1 = dim1End;}
			ctx.beginPath();
            var vxy;
            if (dir == "lat"){
            	vxy = AladinUtils.radecToViewXy(dim2Start, dim1, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
    		}else{
            	vxy = AladinUtils.radecToViewXy(dim1, dim2Start, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
    		}
    		var hasMoved = false;
            var k = 0;
            if (vxy) {
	            ctx.moveTo(vxy.vx, vxy.vy);
	            hasMoved = true;
            }
            
            for (var dim2 = dim2Start; dim2 <= dim2End + dim2Step; dim2 += dim2Step) {
                k++;
           		if (dir == "lat"){
               		vxy = AladinUtils.radecToViewXy(dim2, dim1, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
            	}else{
               		vxy = AladinUtils.radecToViewXy(dim1, dim2, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
    			}
            	if (!vxy) {
            		if(hasMoved){
            			ctx.stroke();
            			hasMoved = false;
            		}
                    continue;
            	}else if (!hasMoved){
		            ctx.moveTo(vxy.vx, vxy.vy);
	            	hasMoved = true;
            	}else{
               		ctx.lineTo(vxy.vx, vxy.vy);
           		}
           		if (k == NB_STEPS / 2 - 5) {
	                	if(dim1<textMin){
	                		textCoor = dim1 + (textMax-textMin);
	                	}else if(dim1>textMax){
	                		textCoor = dim1 - (textMax-textMin);
	                	}else{
	                		textCoor = dim1;
	                	}
	                    if (cooFrame == CooFrameEnum.J2000 && dir == 'lon'){
            				h = Math.floor(textCoor / 15.0);	
            				m = Math.floor((textCoor-h*15) / 0.25);
            				s = Math.round((textCoor-h*15 - m * 0.25) * 60 * 4);
            				h = ("0" + h).slice(-2);
            				m = ("0" + m).slice(-2);
            				s = ("0" + s).slice(-2);
                			text = h + ":" + m + ":" + s;
            			}else  if (cooFrame == CooFrameEnum.J2000 && dir == 'lat'){
            				text = ""
            				if( textCoor < 0 ) text = "-";
            				absNum = Math.abs(textCoor);
            				d = Math.floor(absNum);
            				m = Math.round((absNum-d) * 60);
        					d = ("0" + d).slice(-2);
            				m = ("0" + m).slice(-2);
                			text = text + d + ":" + m;
            			}else{
                			text = textCoor.toFixed(2);
        				}
	                	ctx.globalAlpha = 0.8;
	                	if(isMain){
			            	ctx.fillStyle = colour;
	                    	ctx.fillText(text, vxy.vx, vxy.vy - 2);
	                	}else{
		                    ctx.strokeText(text, vxy.vx, vxy.vy - 2);
	                	}
	                	ctx.globalAlpha = alpha;
                	}
       		
            }
            ctx.stroke();
        }
	};

    
    
    return CooGrid;
})();
