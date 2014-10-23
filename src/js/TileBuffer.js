/******************************************************************************
 * Aladin HTML5 project
 * 
 * File TileBuffer
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

TileBuffer = (function() {
	var NB_MAX_TILES = 800; // buffer size
	
	// constructor
	function TileBuffer() {
		this.pointer = 0;
		this.tilesMap = {};
		this.tilesArray = new Array(NB_MAX_TILES);

		for (var i=0; i<NB_MAX_TILES; i++) {
			this.tilesArray[i] = new Tile(new Image(), null);
		}
	};
	
	TileBuffer.prototype.addTile = function(url) {
	    // return null if already in buffer
        if (this.getTile(url)) {
            return null;
        }

        // delete existing tile
        var curTile = this.tilesArray[this.pointer];
        if (curTile.url != null) {
            curTile.img.src = null;
            delete this.tilesMap[curTile.url];
        }

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
