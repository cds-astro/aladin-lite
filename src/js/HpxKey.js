// Copyright 2016 - UDS/CNRS
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
 * File HpxKey
 * This class represents a HEALPix cell
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HpxKey = (function() {

    "use strict";

    /** Constructor
     *  
     */
    var HpxKey = function(norder, npix, hips, width, height, dx, dy, allskyTexture, allskyTextureSize) {
        this.norder = norder;
        this.npix = npix;

        this.nside = Math.pow(2, norder);

        this.hips = hips; // survey to which this HpxKey is attached
        this.frame = hips.cooFrame; // coordinate frame of the survey to which this HpxKey is attached

        this.width = width; // width of the tile
        this.height = height; // height of the tile

        this.dx = dx || 0; // shift in x (for all-sky tiles)
        this.dy = dy || 0; // shift in y (for all-sky tiles)

        this.allskyTexture = allskyTexture || undefined;
        this.allskyTextureSize = allskyTextureSize;

        this.parente = 0; // if this key comes from an ancestor, length of the filiation

        this.children = null; 
        this.ancestor = null; // ancestor having the pixels
    }

    // "static" methods
    HpxKey.createHpxKeyfromAncestor = function(father, childNb) {
        var hpxKey = new HpxKey(father.norder+1, father.npix*4 + childNb, father.hips, father.width/2, father.height/2,
                                childNb==2 || childNb==3 ? father.dx+father.width/2 : father.dx, childNb==1 || childNb==3 ? father.dy+father.height/2 : father.dy, father.allskyTexture, father.allskyTextureSize);
        hpxKey.parente = father.parente + 1;
        hpxKey.ancestor = father.ancestor || father;


        return hpxKey;
    };

    var MAX_PARENTE = 4;

    HpxKey.prototype = {

        draw: function(ctx, view) {
//console.log('Drawing ', this.norder, this.npix);
            var n = 0; // number of traced triangles
            var corners = this.getProjViewCorners(view);

            if (corners==null) {
                return 0;
            }
     

            var now = new Date().getTime();
            var updateNeededTiles = this.ancestor==null && this.norder>=3 && (now-this.hips.lastUpdateDateNeededTiles) > 0.1;

            try {
                if (isTooLarge(corners)) {
//console.log('too large');
                    var m = this.drawChildren(ctx, view, MAX_PARENTE);

                    // Si aucun sous-losange n'a pu être dessiné, je trace tout de même le père
                    if( m>0 ) {
                        return m;
                    }
                }
            }
            catch(e) {
                return 0;
            }


            // actual drawing
            var norder = this.ancestor==null ? this.norder : this.ancestor.norder;
            var npix = this.ancestor==null ? this.npix : this.ancestor.npix;

            //corners = AladinUtils.grow2(corners, 1); // grow by 1 pixel in each direction
            var url = this.hips.getTileURL(norder, npix);
            var tile = this.hips.tileBuffer.getTile(url);
            if (tile && Tile.isImageOk(tile.img) || this.allskyTexture) {
                if (!this.allskyTexture && !this.hips.tileSize) {
                    this.hips.tileSize = tile.img.width;
                }
                var img = this.allskyTexture || tile.img;
                var w = this.allskyTextureSize || img.width;
                if (this.parente) {
                    w = w / Math.pow(2, this.parente);
                } 
                this.hips.drawOneTile2(ctx, img, corners, w, null, this.dx, this.dy, true, norder);
                n += 2;

                //var ctx2 = view.reticleCtx;
/*
                var ctx2 = ctx;

                ctx2.strokeStyle = 'red';
                ctx2.beginPath();
                ctx2.moveTo(corners[0].vx, corners[0].vy);
                ctx2.lineTo(corners[1].vx, corners[1].vy);
                ctx2.lineTo(corners[2].vx, corners[2].vy);
                ctx2.lineTo(corners[3].vx, corners[3].vy);
                ctx2.lineTo(corners[0].vx, corners[0].vy);
                ctx2.stroke();
*/
            }
            else if (updateNeededTiles && ! tile) {
                tile = this.hips.tileBuffer.addTile(url);
                view.downloader.requestDownload(tile.img, tile.url, this.hips.useCors);
                this.hips.lastUpdateDateNeededTiles = now;
                view.requestRedrawAtDate(now+HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY+10);
            }


            return n;
        },

        drawChildren: function(ctx, view, maxParente) {
            var n=0;
            var limitOrder = 13; // corresponds to NSIDE=8192, current HealpixJS limit
            if ( this.width>1 && this.norder<limitOrder && this.parente<maxParente ) {
                var children = this.getChildren();
                if ( children!=null ) {
                    for ( var i=0; i<4; i++ ) {
//console.log(i);
                        if ( children[i]!=null ) {
                            n += children[i].draw(ctx , view, maxParente);
                        }
                    }
                }
            }

            return n;
        },


        // returns the 4 HpxKey children
        getChildren: function() {
            if (this.children!=null) {
                return this.children;
            }

            var children = [];
            for ( var childNb=0; childNb<4; childNb++ ) {
                var child = HpxKey.createHpxKeyfromAncestor(this, childNb);
                children[childNb] = child;
            }
            this.children = children;


            return this.children;
        },



        getProjViewCorners: function(view) {
            var cornersXY = [];
            var cornersXYView = [];
            var spVec = new SpatialVector();

            corners = HealpixCache.corners_nest(this.npix, this.nside);

            var lon, lat;
            for (var k=0; k<4; k++) {
                spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);

                // need for frame transformation ?
                if (this.frame != view.cooFrame) {
                    if (this.frame==CooFrameEnum.J2000) {
                        var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]);
                        lon = radec[0];
                        lat = radec[1];
                    }
                    else if (this.frame==CooFrameEnum.GAL) {
                        var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]);
                        lon = radec[0];
                        lat = radec[1];
                    }
                }
                else {
                    lon = spVec.ra();
                    lat = spVec.dec();
                }
                cornersXY[k] = view.projection.project(lon, lat);
            }


            if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
                return null;
            }



            for (var k=0; k<4; k++) {
                cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, view.width, view.height, view.largestDim, view.zoomFactor);
            }

            return cornersXYView;
        }

    } // end of HpxKey.prototype




    /** Returns the squared distance for points in array c at indexes g and d
     */
    var dist = function(c, g, d) {
        var dx=c[g].vx-c[d].vx;
        var dy=c[g].vy-c[d].vy;
        return  dx*dx + dy*dy;
    }


    var M = 280*280;
    var N = 150*150;
    var RAP=0.7;

    /** Returns true if the HEALPix rhomb described by its 4 corners (array c)
     * is too large to be drawn in one pass ==> need to be subdivided */
    var isTooLarge = function(c) {

        var d1,d2;
        if ( (d1=dist(c,0,2))>M || (d2=dist(c,2,1))>M ) {
            return true;
        }
        if ( d1==0 || d2==0 ) {
            throw "Rhomb error";
        }
        var diag1 = dist(c,0,3);
        var diag2 = dist(c,1,2);
        if ( diag2==0 || diag2==0 ) {
            throw "Rhomb error";
        }
        var rap = diag2>diag1 ? diag1/diag2 : diag2/diag1;

        return rap<RAP && (diag1>N || diag2>N);
    }


    return HpxKey;

})();


