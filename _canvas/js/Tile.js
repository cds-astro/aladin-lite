/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Tile
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Tile = (function() {
	var Tile = function(img, url) {
		this.img = img;
		this.url = url;
	};
	
	// check whether the image corresponding to the tile is loaded and ready to be displayed
	//
	// source : http://www.sajithmr.me/javascript-check-an-image-is-loaded-or-not
	Tile.isImageOk = function(img) {
		if (img.allSkyTexture) {
			return true;
		}
		
        if (!img.src) {
            return false;
        }

	    // During the onload event, IE correctly identifies any images that
	    // weren’t downloaded as not complete. Others should too. Gecko-based
	    // browsers act like NS4 in that they report this incorrectly.
	    if (!img.complete) {
	        return false;
	    }

	    // However, they do have two very useful properties: naturalWidth and
	    // naturalHeight. These give the true size of the image. If it failed
	    // to load, either of these should be zero.

	    if (typeof img.naturalWidth != "undefined" && img.naturalWidth == 0) {
	        return false;
	    }

	    // No other way of checking: assume it’s ok.
	    return true;
	};
	
	return Tile;
})();
