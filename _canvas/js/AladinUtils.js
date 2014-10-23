/******************************************************************************
 * Aladin HTML5 project
 * 
 * File AladinUtils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

AladinUtils = {
	/**
	 * passage de xy projection à xy dans la vue écran 
	 * @param x
	 * @param y
	 * @param width
	 * @param height
	 * @param largestDim largest dimension of the view
	 * @returns position in the view
	 */
	xyToView: function(x, y, width, height, largestDim, zoomFactor) {
		// we round the result for performance gains
		return {vx: AladinUtils.myRound(largestDim/2*(1+zoomFactor*x)-(largestDim-width)/2), vy: AladinUtils.myRound(largestDim/2*(1+zoomFactor*y)-(largestDim-height)/2)};
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
	
	// passage d'un numéro de pixel à un certain ordre à un numéro de pixel à un autre ordre
	ipixToIpix: function(npixIn, norderIn, norderOut) {
		var npixOut = [];
		if (norderIn>=norderOut) {
		}
	}
	
	
};

