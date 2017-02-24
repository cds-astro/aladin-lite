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
 * File AladinUtils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
AladinUtils = (function() {
    return {
    	/**
    	 * passage de xy projection à xy dans la vue écran 
    	 * @param x
    	 * @param y
    	 * @param width
    	 * @param height
    	 * @param largestDim largest dimension of the view
    	 * @returns position in the view
    	 */
    	xyToView: function(x, y, width, height, largestDim, zoomFactor, round) {
    	    if (round==undefined) {
                // we round by default
    	        round = true;
    	    }

    	    if (round) {
    	        // we round the result for presumed performance gains
    	        return {vx: AladinUtils.myRound(largestDim/2*(1+zoomFactor*x)-(largestDim-width)/2), vy: AladinUtils.myRound(largestDim/2*(1+zoomFactor*y)-(largestDim-height)/2)};

    	    }
    	    else {
                return {vx: largestDim/2*(1+zoomFactor*x)-(largestDim-width)/2, vy: largestDim/2*(1+zoomFactor*y)-(largestDim-height)/2};
    	    }
    	},
    	
    	/**
    	 * passage de xy dans la vue écran à xy projection
    	 * @param vx
    	 * @param vy
    	 * @param width
    	 * @param height
    	 * @param largestDim
    	 * @param zoomFactor
    	 * @returns position in xy projection
    	 */
    	viewToXy: function(vx, vy, width, height, largestDim, zoomFactor) {
    		return {x: ((2*vx+(largestDim-width))/largestDim-1)/zoomFactor, y: ((2*vy+(largestDim-height))/largestDim-1)/zoomFactor};
    	},

    	/**
    	 * convert a 
    	 * @returns position x,y in the view. Null if projection is impossible
    	 */
        radecToViewXy: function(ra, dec, currentProjection, currentFrame, width, height, largestDim, zoomFactor) {
            var xy;
            if (currentFrame!=CooFrameEnum.J2000) {
                var lonlat = CooConversion.J2000ToGalactic([ra, dec]);
                xy = currentProjection.project(lonlat[0], lonlat[1]);
            }
            else {
                xy = currentProjection.project(ra, dec);
            }
            if (!xy) {
                return null;
            }
            
            return AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor, false);
        },

    	
    	myRound: function(a) {
    		if (a<0) {
    			return -1*( (-a) | 0);
    		}
    		else {
    			return a | 0;
    		}
    	},
    	
    	
    	
    	/**
    	 * tests whether a healpix pixel is visible or not
    	 * @param pixCorners array of position (xy view) of the corners of the pixel
    	 * @param viewW
    	 */
    	isHpxPixVisible: function(pixCorners, viewWidth, viewHeight) {
    		for (var i = 0; i<pixCorners.length; i++) {
    			if ( pixCorners[i].vx>=-20 && pixCorners[i].vx<(viewWidth+20) &&
    				 pixCorners[i].vy>=-20 && pixCorners[i].vy<(viewHeight+20) ) {
    				return true;
    			}
    		}
    		return false;
    	},
    	
    	ipixToIpix: function(npixIn, norderIn, norderOut) {
    		var npixOut = [];
    		if (norderIn>=norderOut) {
    		}
    	},
        
        getZoomFactorForAngle: function(angleInDegrees, projectionMethod) {
            var p1 = {ra: 0, dec: 0};
            var p2 = {ra: angleInDegrees, dec: 0};
            var projection = new Projection(angleInDegrees/2, 0);
            projection.setProjection(projectionMethod);
            var p1Projected = projection.project(p1.ra, p1.dec);
            var p2Projected = projection.project(p2.ra, p2.dec);
           
            var zoomFactor = 1/(p1Projected.X - p2Projected.Y);
            return zoomFactor;
        },

        // grow array b of vx,vy view positions by *val* pixels
        grow2: function(b, val) {
            var j=0;
            for ( var i=0; i<4; i++ ) {
                if ( b[i]==null ) {
                    j++;
                }
            }

            if( j>1 ) {
                return b;
            }

            var b1 = [];
            for ( var i=0; i<4; i++ ) {
                b1.push( {vx: b[i].vx, vy: b[i].vy} );
            }
    
            for ( var i=0; i<2; i++ ) {
                var a = i==1 ? 1 : 0;
                var c = i==1 ? 3 : 2;

                if ( b1[a]==null ) {
                    var d,g;
                    if ( a==0 || a==3 ) {
                        d=1;
                        g=2;
                    }
                    else {
                        d=0;
                        g=3;
                    }
                    b1[a] = {vx: (b1[d].vx+b1[g].vx)/2, vy: (b1[d].vy+b1[g].vy)/2};
                }
                if ( b1[c]==null ) {
                    var d,g;
                    if ( c==0 || c==3 ) {
                        d=1;
                        g=2;
                    }
                    else {
                        d=0;
                        g=3;
                    }
                    b1[c] = {vx: (b1[d].vx+b1[g].vx)/2, vy: (b1[d].vy+b1[g].vy)/2};
                }
                if( b1[a]==null || b1[c]==null ) {
                    continue;
                }

                var angle = Math.atan2(b1[c].vy-b1[a].vy, b1[c].vx-b1[a].vx);
                var chouilla = val*Math.cos(angle);
                b1[a].vx -= chouilla;
                b1[c].vx += chouilla;
                chouilla = val*Math.sin(angle);
                b1[a].vy-=chouilla;
                b1[c].vy+=chouilla;
            }
            return b1;
        }
 
    	
    };
})();

