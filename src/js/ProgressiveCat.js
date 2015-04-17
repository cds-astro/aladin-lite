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
 * File ProgressiveCat.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// TODO: indexer sources par numéro HEALPix
// TODO : harmoniser parsing avec classe Catalog
ProgressiveCat = (function() {
    
    // TODO : test if CORS support. If no, need to pass through a proxy
    // currently, we suppose CORS is supported
    
    // constructor
    ProgressiveCat = function(rootUrl, frameStr, maxOrder, options) {
        options = options || {};

        this.type = 'progressivecat';
        
        this.rootUrl = rootUrl;
        this.frame = CooFrameEnum.fromString(frameStr) || CooFrameEnum.J2000;
        this.maxOrder = maxOrder;
        this.isShowing = true; // TODO : inherit from catalogue

        this.name = options.name || "progressive-cat";
        this.color = options.color || Color.getNextColor();
        this.sourceSize = options.sourceSize || 10;
        

        // we cache the list of sources in each healpix tile. Key of the cache is norder+'-'+npix
        this.sourcesCache = new Utils.LRUCache(100);

        this.cacheCanvas = document.createElement('canvas');
        this.cacheCanvas.width = this.sourceSize;
        this.cacheCanvas.height = this.sourceSize;
        var cacheCtx = this.cacheCanvas.getContext('2d');
        cacheCtx.beginPath();
        cacheCtx.strokeStyle = this.color;
        cacheCtx.lineWidth = 2.0;
        cacheCtx.moveTo(0, 0);
        cacheCtx.lineTo(0,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize, 0);
        cacheCtx.lineTo(0, 0);
        cacheCtx.stroke();
    };

    function getFields(instance, xml) {
        var attributes = ["name", "ID", "ucd", "utype", "unit", "datatype", "arraysize", "width", "precision"];

        var fields = [];
        var k = 0;
        instance.keyRa = instance.keyDec = null;
        $(xml).find("FIELD").each(function() {
            var f = {};
            for (var i=0; i<attributes.length; i++) {
                var attribute = attributes[i];
                if ($(this).attr(attribute)) {
                    f[attribute] = $(this).attr(attribute);
                }
                
            }
            if ( ! f.ID) {
                f.ID = "col_" + k;
            }
            
            if (!instance.keyRa && f.ucd && (f.ucd.indexOf('pos.eq.ra')==0 || f.ucd.indexOf('POS_EQ_RA')==0)) {
                if (f.name) {
                    instance.keyRa = f.name;
                }
                else {
                    instance.keyRa = f.ID;
                }
            }
            if (!instance.keyDec && f.ucd && (f.ucd.indexOf('pos.eq.dec')==0 || f.ucd.indexOf('POS_EQ_DEC')==0)) {
                if (f.name) {
                    instance.keyDec = f.name;
                }
                else {
                    instance.keyDec = f.ID;
                }
            }
            
            fields.push(f);
            k++;
        });

        return fields;
    }

    function getSources(instance, csv, fields) {
        // TODO : find ra and dec key names (see in Catalog)
        if (!instance.keyRa || ! instance.keyDec) {
            return [];
        }
        lines = csv.split('\n');
        var mesureKeys = [];
        for (var k=0; k<fields.length; k++) {
            if (fields[k].name) {
                mesureKeys.push(fields[k].name);
            }
            else {
                mesureKeys.push(fields[k].ID);
            }
        }
        

        var sources = [];
        var coo = new Coo();
        // start at i=1, as first line repeat the fields names
        for (var i=2; i<lines.length; i++) {
            var mesures = {};
            var data = lines[i].split('\t');
            if (data.length<mesureKeys.length) {
                continue;
            }
            for (var j=0; j<mesureKeys.length; j++) {
                mesures[mesureKeys[j]] = data[j];
            }
            var ra, dec;
            if (Utils.isNumber(mesures[instance.keyRa]) && Utils.isNumber(mesures[instance.keyDec])) {
                ra = parseFloat(mesures[instance.keyRa]);
                dec = parseFloat(mesures[instance.keyDec]);
            }
            else {
                coo.parse(mesures[instance.keyRa] + " " + mesures[instance.keyDec]);
                ra = coo.lon;
                dec = coo.lat;
            }
            sources.push(new cds.Source(ra, dec, mesures));
        }
        return sources;
    }

    ProgressiveCat.prototype = {

        init: function(view) {
            this.view = view;
            if (this.level3Sources) {
                return; // if already loaded, do nothing
            }
            this.loadLevel2Sources();
        },

        loadLevel2Sources: function() {
            var self = this;
            $.ajax({
                /*
                url: Aladin.JSONP_PROXY,
                data: {"url": self.rootUrl + '/' + 'Norder2/Allsky.xml'},
                datatype: 'jsonp',
                */
                url: self.rootUrl + '/' + 'Norder2/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.fields = getFields(self, xml);
                    self.level2Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    self.loadLevel3Sources();
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        loadLevel3Sources: function() {
            var self = this;
            $.ajax({
                /*
                url: Aladin.JSONP_PROXY,
                data: {"url": self.rootUrl + '/' + 'Norder3/Allsky.xml'},
                datatype: 'jsonp',
                */
                url: self.rootUrl + '/' + 'Norder3/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.level3Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    //console.log(self.level3Sources);
                    self.view.requestRedraw();
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        draw: function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
            if (! this.isShowing || ! this.level3Sources) {
                return;
            }
            //var sources = this.getSources();
            this.drawSources(this.level2Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            this.drawSources(this.level3Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            
            if (!this.tilesInView) {
                return;
            }
            var sources, key, t;
            for (var k=0; k<this.tilesInView.length; k++) {
                t = this.tilesInView[k];
                key = t[0] + '-' + t[1];
                sources = this.sourcesCache.get(key);
                if (sources) {
                    this.drawSources(sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
                }
            }
            
            
            
        },
        drawSources: function(sources, ctx, projection, frame, width, height, largestDim, zoomFactor) {
            for (var k=0, len = sources.length; k<len; k++) {
                this.drawSource(sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor);
            }
        },
        getSources: function() {
            var ret = [];
            if (this.level2Sources) {
                ret = ret.concat(this.level2Sources);
            }
            if (this.level3Sources) {
                ret = ret.concat(this.level3Sources);
            }
            if (this.tilesInView) {
                var sources, key, t;
                for (var k=0; k<this.tilesInView.length; k++) {
                    t = this.tilesInView[k];
                    key = t[0] + '-' + t[1];
                    sources = this.sourcesCache.get(key);
                    if (sources) {
                        ret = ret.concat(sources);
                    }
                }
            }
            
            return ret;
        },

        // TODO : factoriser avec drawSource de Catalog
        drawSource: function(s, ctx, projection, frame, width, height, largestDim, zoomFactor) {
            if (! s.isShowing) {
                return;
            }
            var sourceSize = this.sourceSize;
            var xy;
            if (frame!=CooFrameEnum.J2000) {
                var lonlat = CooConversion.J2000ToGalactic([s.ra, s.dec]);
                xy = projection.project(lonlat[0], lonlat[1]);
            }
            else {
                xy = projection.project(s.ra, s.dec);
            }
            if (xy) {
                var xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
                if (xyview) {
                    // TODO : index sources
                    // check if source is visible in view ?
                    if (xyview.vx>(width+sourceSize)  || xyview.vx<(0-sourceSize) ||
                        xyview.vy>(height+sourceSize) || xyview.vy<(0-sourceSize)) {
                        s.x = s.y = undefined;
                        return;
                    }

                    s.x = xyview.vx;
                    s.y = xyview.vy;
                    ctx.drawImage(this.cacheCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
                }
            }
        },
        
        deselectAll: function() {
            for (var k=0; k<this.level2Sources.length; k++) {
                this.level2Sources[k].deselect();
            }
            for (var k=0; k<this.level3Sources.length; k++) {
                this.level3Sources[k].deselect();
            }
            var keys = this.sourcesCache.keys();
            for (key in keys) {
                if ( ! this.sourcesCache[key]) {
                    continue;
                }
                var sources = this.sourcesCache[key];
                for (var k=0; k<sources.length; k++) {
                    sources[k].deselect();
                }
            }
        },

        show: function() {
            if (this.isShowing) {
                return;
            }
            this.isShowing = true;
            this.reportChange();
        },
        hide: function() {
            if (! this.isShowing) {
                return;
            }
            this.isShowing = false;
            this.reportChange();
        },
        reportChange: function() {
            this.view.requestRedraw();
        },
        
        getTileURL: function(norder, npix) {
            var dirIdx = Math.floor(npix/10000)*10000;
            return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + ".tsv";
        },
    
        loadNeededTiles: function() {
            this.tilesInView = [];
            
            this.otherSources = [];
            var norder = this.view.realNorder;
            if (norder>this.maxOrder) {
                norder = this.maxOrder;
            }
            if (norder<=3) {
                return; // nothing to do, hurrayh !
            }
            var cells = this.view.getVisibleCells(norder, this.frame);
            var ipixList, ipix;
            for (var curOrder=4; curOrder<=norder; curOrder++) {
                ipixList = [];
                for (var k=0; k<cells.length; k++) {
                    ipix = Math.floor(cells[k].ipix / Math.pow(4, norder - curOrder));
                    if (ipixList.indexOf(ipix)<0) {
                        ipixList.push(ipix);
                    }
                }
                
                // load needed tiles
                for (var i=0; i<ipixList.length; i++) {
                    this.tilesInView.push([curOrder, ipixList[i]]);
                }
            }
            
            var t, key;
            var self = this;
            for (var k=0; k<this.tilesInView.length; k++) {
                t = this.tilesInView[k];
                key = t[0] + '-' + t[1]; // t[0] is norder, t[1] is ipix
                if (!this.sourcesCache.get(key)) {
                    (function(self, norder, ipix) { // wrapping function is needed to be able to retrieve norder and ipix in ajax success function
                        var key = norder + '-' + ipix;
                        $.ajax({
                            /*
                            url: Aladin.JSONP_PROXY,
                            data: {"url": self.getTileURL(norder, ipix)},
                            */
                            // ATTENTIOn : je passe en JSON direct, car je n'arrive pas à choper les 404 en JSONP
                            url: self.getTileURL(norder, ipix),
                            method: 'GET',
                            //dataType: 'jsonp',
                            success: function(tsv) {
                                self.sourcesCache.set(key, getSources(self, tsv, self.fields));
                                //self.otherSources = self.otherSources.concat(getSources(tsv, self.fields));
                                self.view.requestRedraw();
                            },
                            error: function() {
                                // on suppose qu'il s'agit d'une erreur 404
                                self.sourcesCache.set(key, []);
                            }
                        });
                    })(this, t[0], t[1]);
                }
            }
        }



    }; // END OF .prototype functions
    
    
    return ProgressiveCat;
})();
    
