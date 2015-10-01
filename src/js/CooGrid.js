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
    
    var NB_STEPS = 10;
    var NB_LINES = 10;
    
    CooGrid.prototype.redraw = function(ctx, projection, frame, width, height, largestDim, zoomFactor, fov) {
        if (fov>60) { // currently not supported
            return; 
        }
        
        var lonMax = 0, lonMin = 359.9999, latMax = -90, latMin = 90;
        var lonlat1 = viewxy2lonlat(projection, 0, 0, width, height, largestDim, zoomFactor);
        var lonlat2 = viewxy2lonlat(projection, width-1, height-1, width, height, largestDim, zoomFactor);
        lonMin = Math.min(lonlat1.lon, lonlat2.lon);
        lonMax = Math.max(lonlat1.lon, lonlat2.lon);
        latMin = Math.min(lonlat1.lat, lonlat2.lat);
        latMax = Math.max(lonlat1.lat, lonlat2.lat);
        
        var lonlat3 = viewxy2lonlat(projection, 0, height-1, width, height, largestDim, zoomFactor);
        lonMin = Math.min(lonMin, lonlat3.lon);
        lonMax = Math.max(lonMax, lonlat3.lon);
        latMin = Math.min(latMin, lonlat3.lat);
        latMax = Math.max(latMax, lonlat3.lat);
        
        var lonlat4 = viewxy2lonlat(projection, width-1, 0, width, height, largestDim, zoomFactor);
        lonMin = Math.min(lonMin, lonlat4.lon);
        lonMax = Math.max(lonMax, lonlat4.lon);
        latMin = Math.min(latMin, lonlat4.lat);
        latMax = Math.max(latMax, lonlat4.lat);
        

        
        var lonDiff = lonMax - lonMin;
        var latDiff = latMax - latMin;
        
        var LON_STEP, LAT_STEP;
        if (fov>10) {
            LON_STEP = 4;
            LAT_STEP = 4;
        }
        else if (fov>1) {
            LON_STEP = 1;
            LAT_STEP = 1;
        }
        else if (fov>0.1) {
            LON_STEP = 0.1;
            LAT_STEP = 0.1;
        }
        else {
            LON_STEP = 0.01;
            LAT_STEP = 0.01;
        }
        
        var lonStart = Math.round(lonMin % LON_STEP) * (LON_STEP);
        var latStart = Math.round(latMin % LAT_STEP) * (LAT_STEP);
        
        
        
        ctx.lineWidth = 1;
        ctx.strokeStyle = "rgb(120,120,255)";
        // draw iso-latitudes lines
        for (var lat=latStart; lat<latMax+LAT_STEP; lat+=LAT_STEP) {
            ctx.beginPath();
            
            var vxy;
            vxy = AladinUtils.radecToViewXy(lonMin, lat, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
            if (!vxy) {
                continue;
            }
            ctx.moveTo(vxy.vx, vxy.vy);
            var k = 0;
            for (var lon=lonMin; lon<lonMax+LON_STEP; lon+=lonDiff/10) {
                k++;
                vxy = AladinUtils.radecToViewXy(lon, lat, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
                ctx.lineTo(vxy.vx, vxy.vy);
                if (k==3 ) {
                    ctx.strokeText(lat.toFixed(2), vxy.vx, vxy.vy-2);
                }
                
            }
            ctx.stroke();
        }
        
        for (var lon=lonStart; lon<lonMax+LON_STEP; lon+=LON_STEP) {
            ctx.beginPath();
            
            var vxy;
            vxy = AladinUtils.radecToViewXy(lon, latMin, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
            if (!vxy) {
                continue;
            }
            ctx.moveTo(vxy.vx, vxy.vy);
            var k = 0;
            for (var lat=latMin; lat<latMax+LAT_STEP; lat+=latDiff/10) {
                k++;
                vxy = AladinUtils.radecToViewXy(lon, lat, projection, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);
                ctx.lineTo(vxy.vx, vxy.vy);
                if (k==3 ) {
                    ctx.strokeText(lon.toFixed(2), vxy.vx, vxy.vy-2);
                }
            }
            ctx.stroke();
        }
        
        
        
    };

    
    
    return CooGrid;
})();
