/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


Location = function(locationDiv) {
		this.div = $(locationDiv);
	};
	
	Location.prototype.update = function(lon, lat, cooFrame) {
		var coo = new Coo(lon, lat, 7);
		if (cooFrame==CooFrameEnum.J2000) {
            this.div.html('&alpha;, &delta;: ' + coo.format('s/'));
        }
        else {
            this.div.html('l, b: ' + coo.format('d/'));
        }
	};
	
