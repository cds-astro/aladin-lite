/******************************************************************************
 * Aladin Lite project
 * 
 * File MOC
 *
 * This class represents a MOC (Multi Order Coverage map) layer
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

MOC = (function() {
    MOC = function(options) {
        this.norder = undefined;
        this._highResCells = {};
        this._lowResCells = {};

        // TODO homogenize options parsing for all kind of overlay (footprints, catalog, MOC)
        options = options || {};
        this.name = options.name || "MOC";
        this.color = options.color || Color.getNextColor();
        this.lineWidth = options["lineWidth"] || 1;



        this.isShowing = true;
    }

    
    function log2(val) {
        return Math.log(val) / Math.LN2;
    }

    // max norder we can currently handle (limitation of healpix.js)
    MOC.MAX_NORDER = 13; // NSIDE = 8192

    MOC.LOWRES_MAXORDER = 5; // 5 or 6 ??
    MOC.HIGHRES_MAXORDER = 11; // ??

    // TODO: options to modifiy this ?
    MOC.PIVOT_FOV = 20; // when do we switch from low res cells to high res cells (fov in degrees)

    /**
     * set MOC data by parsing a URL pointing to a MOC file
     */
    MOC.prototype.dataFromURL = function(mocURL, successCallback) {

        var self = this;
        var callback = function() {
            // first, let's find MOC norder
            var hdr0;
            try {
                hdr0 = this.getHeader(0);
            }
            catch (e) {
                console.err('Could not get header of extension #0');
                return;
            }
            var hdr1 = this.getHeader(1);

            if (hdr0.contains('HPXMOC')) {
                self.order = hdr0.get('HPXMOC')
            }
            else if (hdr0.contains('MOCORDER')) {
                self.order = hdr0.get('MOCORDER')
            }
            else if (hdr1.contains('HPXMOC')) {
                self.order = hdr1.get('HPXMOC')
            }
            else if (hdr1.contains('MOCORDER')) {
                self.order = hdr1.get('MOCORDER')
            }
            else {
                console.err('Can not find MOC order in FITS file');
                return;
            }

            var data = this.getDataUnit(1);
            var colName = data.columns[0];
            data.getRows(0, data.rows, function(rows) {
                for (var k=0; k<rows.length; k++) {
                    var uniq = rows[k][colName];
                    var order = Math.floor(Math.floor(log2(Math.floor(uniq/4))) / 2);
                    var ipix = uniq - 4 *(Math.pow(4, order));

                    // fill low and high level cells
                    // 1. if order <= LOWRES_MAXORDER, just store value in low and high res cells
                    if (order<=MOC.LOWRES_MAXORDER) {
                        if (! (order in self._lowResCells)) {
                            self._lowResCells[order] = [];
                            self._highResCells[order] = [];
                        }
                        self._lowResCells[order].push(ipix);
                        self._highResCells[order].push(ipix);
                    }
                    // 2. if LOWRES_MAXORDER < order <= HIGHRES_MAXORDER , degrade ipix for low res cells
                    else if (order<=MOC.HIGHRES_MAXORDER) {
                        if (! (order in self._highResCells)) {
                            self._highResCells[order] = [];
                        }
                        self._highResCells[order].push(ipix);
                        
                        var degradedOrder = MOC.LOWRES_MAXORDER; 
                        var degradedIpix  = Math.floor(ipix / Math.pow(4, (order - degradedOrder)));
                        if (! (degradedOrder in self._lowResCells)) {
                            self._lowResCells= [];
                        }
                        self._lowResCells[degradedOrder].push(degradedIpix);
                    }
                    // 3. if order > HIGHRES_MAXORDER , degrade ipix for low res and high res cells
                    else {
                        // low res cells
                        var degradedOrder = MOC.LOWRES_MAXORDER; 
                        var degradedIpix  = Math.floor(ipix / Math.pow(4, (order - degradedOrder)));
                        if (! (degradedOrder in self._lowResCells)) {
                            self._lowResCells = [];
                        }
                        self._lowResCells[degradedOrder].push(degradedIpix);
                        
                        // high res cells
                        degradedOrder = MOC.HIGHRES_MAXORDER; 
                        degradedIpix  = Math.floor(ipix / Math.pow(4, (order - degradedOrder)));
                        if (! (degradedOrder in self._highResCells)) {
                            self._highResCells = [];
                        }
                        self._highResCells[degradedOrder].push(degradedIpix);
                    }
                }

            });
            data = null; // this helps releasing memory

            if (successCallback) {
                successCallback();
            }
        }; // end of callback function

        this.dataURL = mocURL;
        // TODO: general method to retrieve data (through dedicated class ?)
        var proxiedURL = Aladin.JSONP_PROXY + '?url=' + encodeURIComponent(this.dataURL);
        new astro.FITS(proxiedURL, callback);
    }

    MOC.prototype.setView = function(view) {
        this.view = view;
    };
    
    MOC.prototype.draw = function(ctx, projection, viewFrame, width, height, largestDim, zoomFactor, fov) {
        if (! this.isShowing) {
            return;
        }

        var mocCells = fov > MOC.PIVOT_FOV ? this._lowResCells : this._highResCells;

        this._drawCells(ctx, mocCells, fov, projection, viewFrame, CooFrameEnum.J2000, width, height, largestDim, zoomFactor);

        
    };

    MOC.prototype._drawCells = function(ctx, mocCells, fov, projection, viewFrame, surveyFrame, width, height, largestDim, zoomFactor) {
        ctx.lineWidth = this.lineWidth;
        ctx.strokeStyle = this.color;
        ctx.beginPath();

        var orderedKeys = [];
        for (key in mocCells) {
            orderedKeys.push(key);
        }
        orderedKeys.sort();
        var nside, xyCorners, ipix;

        // go through all MOC cells
        if (fov>80) {
            var norder;
            for (var i=0; i<orderedKeys.length; i++) {
                norder = parseInt(orderedKeys[i]);
                nside = 1 << norder;

                for (var j=0; j<mocCells[norder].length; j++) {
                    ipix = mocCells[norder][j];
                    if (norder>=3) {
                        xyCorners = getXYCorners(nside, ipix, viewFrame, surveyFrame, width, height, largestDim, zoomFactor, projection);
                        if (xyCorners) {
                            drawCorners(ctx, xyCorners);
                        }
                    }
                    else { // compute all norder 3 ipix indexes
                        var factor = Math.pow(4, (3-norder));
                        var startIpix = ipix * factor;

                        for (var k=0; k<factor; k++) {
                            xyCorners = getXYCorners(8, startIpix + k, viewFrame, surveyFrame, width, height, largestDim, zoomFactor, projection);
                            if (xyCorners) {
                                drawCorners(ctx, xyCorners);
                            }
                        }
                    }
                }
            }
        }
        else {
            var visibleHpxCellsOrder3 = this.view.getVisiblePixList(3, CooFrameEnum.J2000);
            var cellsOrder3ToIgnore = {}
            var norderMax = parseInt(orderedKeys[orderedKeys.length-1]);
            for (var norder=1; norder<=norderMax; norder++) {
                nside = 1 << norder;

                if (norder<=3) {
                    for (var j=0; j<mocCells[norder].length; j++) {
                        ipix = mocCells[norder][j];
                        var factor = Math.pow(4, (3-norder));
                        var startIpix = ipix * factor;
                        for (var k=0; k<factor; k++) {
                            norder3Ipix = startIpix + k;
                            xyCorners = getXYCorners(8, norder3Ipix, viewFrame, surveyFrame, width, height, largestDim, zoomFactor, projection);
                            if (xyCorners) {
                                drawCorners(ctx, xyCorners);
                            }
                            cellsOrder3ToIgnore[norder3Ipix] = 1;
                        }
                    }
                }
                // TODO: this part could be improved by eliminating ipix already rendered
                else {
                    for (var j=0; j<mocCells[norder].length; j++) {
                        ipix = mocCells[norder][j];
                        var parentIpixOrder3 = Math.floor(ipix/Math.pow(4, norder-3));
                        if (parentIpixOrder3 in cellsOrder3ToIgnore) {
                            continue;
                        }
                        xyCorners = getXYCorners(nside, ipix, viewFrame, surveyFrame, width, height, largestDim, zoomFactor, projection);
                        if (xyCorners) {
                            drawCorners(ctx, xyCorners);
                        }
                    }
                }
            }
        }
/*
*/
        ctx.stroke();
    };

    var drawCorners = function(ctx, xyCorners) {
        ctx.moveTo(xyCorners[0].vx, xyCorners[0].vy);
        ctx.lineTo(xyCorners[1].vx, xyCorners[1].vy);
        ctx.lineTo(xyCorners[2].vx, xyCorners[2].vy);
        ctx.lineTo(xyCorners[3].vx, xyCorners[3].vy);
        ctx.lineTo(xyCorners[0].vx, xyCorners[0].vy);
    }

    // TODO: merge with what is done in View.getVisibleCells
    var getXYCorners = function(nside, ipix, viewFrame, surveyFrame, width, height, largestDim, zoomFactor, projection) {
        var cornersXYView = [];
        var cornersXY = [];

        var spVec = new SpatialVector();

        var corners = HealpixCache.corners_nest(ipix, nside);
        for (var k=0; k<4; k++) {
            spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);

            // need for frame transformation ?
            if (surveyFrame && surveyFrame != viewFrame) {
                if (surveyFrame==CooFrameEnum.J2000) {
                    var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]);
                    lon = radec[0];
                    lat = radec[1];
                }
                else if (surveyFrame==CooFrameEnum.GAL) {
                    var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]);
                    lon = radec[0];
                    lat = radec[1];
                }
            }
            else {
                lon = spVec.ra();
                lat = spVec.dec();
            }

            cornersXY[k] = projection.project(lon, lat);
        }


        if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
            return null;
        }

        for (var k=0; k<4; k++) {
            cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, width, height, largestDim, zoomFactor);
        }

        var indulge = 10;
        // detect pixels outside view. Could be improved !
        // we minimize here the number of cells returned
        if( cornersXYView[0].vx<0 && cornersXYView[1].vx<0 && cornersXYView[2].vx<0 &&cornersXYView[3].vx<0) {
            return null;
        }
        if( cornersXYView[0].vy<0 && cornersXYView[1].vy<0 && cornersXYView[2].vy<0 &&cornersXYView[3].vy<0) {
            return null;
        }
        if( cornersXYView[0].vx>=width && cornersXYView[1].vx>=width && cornersXYView[2].vx>=width &&cornersXYView[3].vx>=width) {
            return null;
        }
        if( cornersXYView[0].vy>=height && cornersXYView[1].vy>=height && cornersXYView[2].vy>=height &&cornersXYView[3].vy>=height) {
            return null;
        }

        return cornersXYView;
    };

    MOC.prototype.reportChange = function() {
        this.view.requestRedraw();
    };

    MOC.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };

    MOC.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        this.reportChange();
    };



    return MOC;
})();

    
