/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Catalog = function(name, url) {
	this.name = name;
	this.url = url;
	
	this.indexationNorder = 5; // à quel niveau indexe-t-on les sources
	this.sources = [];
	this.color = "#ff0000";
	this.hpxIdx = new HealpixIndex(this.indexationNorder);
	this.hpxIdx.init();
	
	this.sourceSize = 5;
};

Catalog.prototype.addSource = function(source) {
	this.sources.push(source);
};

Catalog.prototype.draw = function(ctx, projection, width, height, largestDim, zoomFactor) {
	var s;
	var xy, xyview;
	var sourceSize = this.sourceSize;
	ctx.fillStyle = this.color;
	for (var k=0, len = this.sources.length; k<len; k++) {
		s = this.sources[k];
		xy = projection.project(s.ra, s.dec);
		if (xy) {
			xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
			if (xyview) {
				ctx.fillRect(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2, sourceSize, sourceSize);
			}
		}
	}
};



Catalog.prototype.drawSource = function(source, projection) {

};