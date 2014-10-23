/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HealpixCache
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// class holding some HEALPix computations for better performances
//
// it is made of :
// - a static cache for HEALPix corners at nside=8 
// - a dynamic cache for 
HealpixCache = (function() {

    var HealpixCache = {};
    
    HealpixCache.staticCache = {corners: {nside8: []}};
    // TODO : utilisation du dynamicCache
    HealpixCache.dynamicCache = {};
    
    HealpixCache.lastNside = 8;
    
    HealpixCache.hpxIdxCache = null;
    
    // TODO : conserver en cache le dernier résultat ?
    
    HealpixCache.init = function() {
    	// pre-compute corners position for nside=8
    	var hpxIdx = new HealpixIndex(8);
    	hpxIdx.init();
    	var npix = HealpixIndex.nside2Npix(8);
    	for (var ipix=0; ipix<npix; ipix++) {
    		HealpixCache.staticCache.corners.nside8[ipix] = hpxIdx.corners_nest(ipix, 1);
    	}
    	
    	HealpixCache.hpxIdxCache = hpxIdx;
    };
    
    HealpixCache.corners_nest = function(ipix, nside) {
    	if (nside==8) {
    		return HealpixCache.staticCache.corners.nside8[ipix];
    	}
    	
    	if (nside != HealpixCache.lastNside) {
    		HealpixCache.hpxIdxCache = new HealpixIndex(nside);
    		HealpixCache.hpxIdxCache.init();
    		HealpixCache.lastNside = nside;
    	}
    	
    	return HealpixCache.hpxIdxCache.corners_nest(ipix, 1);
    	
    };
    
    return HealpixCache;
})();
	
