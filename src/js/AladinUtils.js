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

import { Projection } from "./libs/astro/projection.js";
import { CooFrameEnum } from "./CooFrameEnum.js";

export let AladinUtils = (function() {

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
    	        // we round the result for potential performance gains
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
        /*radecToViewXy: function(ra, dec, currentProjection, currentFrame, width, height, largestDim, zoomFactor) {
            var xy;
            if (currentFrame.system != CooFrameEnum.SYSTEMS.J2000) {
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
        },*/
        radecToViewXy: function(ra, dec, view) {
            let xy = view.wasm.worldToScreen(ra, dec);
            return xy;
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
    	 * Test whether a xy position is the view
    	 * @param vx
    	 * @param vy
    	 * @param width
    	 * @param height
    	 * @returns a boolean whether (vx, vy) is in the screen
    	 */
    	isInsideViewXy: function(vx, vy, width, height) {
    		return vx >= 0 && vx < width && vy >= 0 && vy < height
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
        // Zoom is handled in the backend
        /*getZoomFactorForAngle: function(angleInDegrees, projectionMethod) {
            var p1 = {ra: 0, dec: 0};
            var p2 = {ra: angleInDegrees, dec: 0};
            var projection = new Projection(angleInDegrees/2, 0);
            projection.setProjection(projectionMethod);
            var p1Projected = projection.project(p1.ra, p1.dec);
            var p2Projected = projection.project(p2.ra, p2.dec);
           
            var zoomFactor = 1/Math.abs(p1Projected.X - p2Projected.Y);

            return zoomFactor;
        },*/

        counterClockwiseTriangle: function(x1, y1, x2, y2, x3, y3) {
            // From: https://math.stackexchange.com/questions/1324179/how-to-tell-if-3-connected-points-are-connected-clockwise-or-counter-clockwise
            // | x1, y1, 1 |
            // | x2, y2, 1 | > 0 => the triangle is given in anticlockwise order
            // | x3, y3, 1 |
    
            return x1*y2 + y1*x3 + x2*y3 - x3*y2 - y3*x1 - x2*y1 >= 0;
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
        },

        // SVG icons templates are stored here rather than in a CSS, as to allow
        // to dynamically change the fill color
        // Pretty ugly, haven't found a prettier solution yet
        //
        // TODO: store this in the Stack class once it will exist
        //
        SVG_ICONS: {
            CATALOG: '<svg xmlns="http://www.w3.org/2000/svg"><polygon points="1,0,5,0,5,3,1,3"  fill="FILLCOLOR" /><polygon points="7,0,9,0,9,3,7,3"  fill="FILLCOLOR" /><polygon points="10,0,12,0,12,3,10,3"  fill="FILLCOLOR" /><polygon points="13,0,15,0,15,3,13,3"  fill="FILLCOLOR" /><polyline points="1,5,5,9"  stroke="FILLCOLOR" /><polyline points="1,9,5,5" stroke="FILLCOLOR" /><line x1="7" y1="7" x2="15" y2="7" stroke="FILLCOLOR" stroke-width="2" /><polyline points="1,11,5,15"  stroke="FILLCOLOR" /><polyline points="1,15,5,11"  stroke="FILLCOLOR" /><line x1="7" y1="13" x2="15" y2="13" stroke="FILLCOLOR" stroke-width="2" /></svg>',
            MOC: '<svg xmlns="http://www.w3.org/2000/svg"><polyline points="0.5,7,2.5,7,2.5,5,7,5,7,3,10,3,10,5,13,5,13,7,15,7,15,9,13,9,13,12,10,12,10,14,7,14,7,12,2.5,12,2.5,10,0.5,10,0.5,7" stroke-width="1" stroke="FILLCOLOR" fill="transparent" /><line x1="1" y1="10" x2="6" y2="5" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="2" y1="12" x2="10" y2="4" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="5" y1="12" x2="12" y2="5" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="7" y1="13" x2="13" y2="7" stroke="FILLCOLOR" stroke-width="0.5" /><line x1="10" y1="13" x2="13" y2="10" stroke="FILLCOLOR" stroke-width="0.5" /></svg>',
            OVERLAY: '<svg xmlns="http://www.w3.org/2000/svg"><polygon points="10,5,10,1,14,1,14,14,2,14,2,9,6,9,6,5" fill="transparent" stroke="FILLCOLOR" stroke-width="2"/></svg>'
        }
 
    };

})();

