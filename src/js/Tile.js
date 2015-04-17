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
 * File Tile
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Tile = (function() {
    // constructor
	function Tile(img, url) {
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
