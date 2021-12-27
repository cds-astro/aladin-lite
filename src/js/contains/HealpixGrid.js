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
 * File HealpixGrid
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HealpixGrid = (function() {
	var HealpixGrid = function() {
	};
	
	HealpixGrid.prototype.redraw = function(ctx, cornersXYViewMap, fov, norder) {
		// on dessine les lignes
		ctx.lineWidth = 1;
		ctx.strokeStyle = "rgb(150,150,220)";
		ctx.beginPath();
		var cornersXYView;
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;
			
			// draw pixel
			ctx.moveTo(cornersXYView[0].vx, cornersXYView[0].vy);
			ctx.lineTo(cornersXYView[1].vx, cornersXYView[1].vy);
			ctx.lineTo(cornersXYView[2].vx, cornersXYView[2].vy);
			//ctx.lineTo(cornersXYView[3].vx, cornersXYView[3].vy);
			

            //ctx.strokeText(ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
		
		// on dessine les numéros de pixel HEALpix
        ctx.strokeStyle="#FFDDDD";
		ctx.beginPath();
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;

            ctx.strokeText(norder + '/' + ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
	};

	
	
	return HealpixGrid;
})();
