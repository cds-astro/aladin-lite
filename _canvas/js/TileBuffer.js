/******************************************************************************
 * Aladin HTML5 project
 * 
 * File TileBuffer
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

TileBuffer = (function() {
	var NB_MAX_TILES = 1000; // buffer size
	
	var TileBuffer = function() {
		this.pointer = 0;
		this.tilesMap = {};
		this.tilesArray = new Array(NB_MAX_TILES);

		for (var i=0; i<NB_MAX_TILES; i++) {
			this.tilesArray[i] = new Tile(new Image(), null, null);
		}
	};
	
	TileBuffer.prototype.addTile = function(url) {
        if (this.getTile(url)) {
            return null;
        }

        // delete existing tile
        var curTile = this.tilesArray[this.pointer];
        curTile.img.src = null;
        delete this.tilesMap[curTile.url];

        this.tilesArray[this.pointer].url = url;
        this.tilesMap[url] = this.tilesArray[this.pointer];

        this.pointer++;
        if (this.pointer>=NB_MAX_TILES) {
            this.pointer = 0;
        }

        return this.tilesMap[url];
	};
	
	TileBuffer.prototype.getTile = function(url) {
        return this.tilesMap[url];
		
	};
	
	
	return TileBuffer;
})();
