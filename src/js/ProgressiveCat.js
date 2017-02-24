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

// TODO: index sources according to their HEALPix ipix
// TODO : merge parsing with class Catalog
ProgressiveCat = (function() {
    
    // TODO : test if CORS support. If no, need to pass through a proxy
    // currently, we suppose CORS is supported
    
    // constructor
    ProgressiveCat = function(rootUrl, frameStr, maxOrder, options) {
        options = options || {};

        this.type = 'progressivecat';
        
        this.rootUrl = rootUrl; // TODO: method to sanitize rootURL (absolute, no duplicate slashes, remove end slash if existing)
        this.frameStr = frameStr;
        this.frame = CooFrameEnum.fromString(frameStr) || CooFrameEnum.J2000;
        this.maxOrder = maxOrder;
        this.isShowing = true; // TODO : inherit from catalogue

        this.name = options.name || "progressive-cat";
        this.color = options.color || Color.getNextColor();
        this.shape = options.shape || "square";
        this.sourceSize = options.sourceSize || 6;
        this.selectSize = this.sourceSize + 2;
        this.selectionColor = '#00ff00'; // TODO: to be merged with Catalog


        this.onClick = options.onClick || undefined; // TODO: inherit from catalog

        

        // we cache the list of sources in each healpix tile. Key of the cache is norder+'-'+npix
        this.sourcesCache = new Utils.LRUCache(100);

        this.cacheCanvas = cds.Catalog.createShape(this.shape, this.color, this.sourceSize);

        this.cacheSelectCanvas = document.createElement('canvas');
        this.cacheSelectCanvas.width = this.selectSize;
        this.cacheSelectCanvas.height = this.selectSize;
        var cacheSelectCtx = this.cacheSelectCanvas.getContext('2d');
        cacheSelectCtx.beginPath();
        cacheSelectCtx.strokeStyle = this.selectionColor;
        cacheSelectCtx.lineWidth = 2.0;
        cacheSelectCtx.moveTo(0, 0);
        cacheSelectCtx.lineTo(0,  this.selectSize);
        cacheSelectCtx.lineTo( this.selectSize,  this.selectSize);
        cacheSelectCtx.lineTo( this.selectSize, 0);
        cacheSelectCtx.lineTo(0, 0);
        cacheSelectCtx.stroke(); // TODO: to be merged with Catalog



        this.maxOrderAllsky = 2;
        this.isReady = false;
    };

    // TODO: to be put higher in the class diagram, in a HiPS generic class
    ProgressiveCat.readProperties = function(rootUrl, successCallback, errorCallback) {
        if (! successCallback) {
            return;
        }

        var propertiesURL = rootUrl + '/properties';
        $.ajax({
            url: propertiesURL,
            method: 'GET',
            dataType: 'text',
            success: function(propertiesTxt) {
                var props = {};
                var lines = propertiesTxt.split('\n');
                for (var k=0; k<lines.length; k++) {
                    var line = lines[k];
                    var idx = line.indexOf('=');
                    var propName  = $.trim(line.substring(0, idx));
                    var propValue = $.trim(line.substring(idx + 1));
                    
                    props[propName] = propValue;
                }
    
                successCallback(props);
                
            },
            error: function(err) { // TODO : which parameters should we put in the error callback
                errorCallback && errorCallback(err);
            }
        });




        
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
        var newSource;
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
            newSource = new cds.Source(ra, dec, mesures);
            sources.push(newSource);
            newSource.setCatalog(instance);
        }
        return sources;
    };

    ProgressiveCat.prototype = {

        init: function(view) {
            var self = this;
            this.view = view;

            if (this.maxOrder && this.frameStr) {
                this._loadMetadata();
            }

            else {
                ProgressiveCat.readProperties(self.rootUrl,
                    function (properties) {
                        self.properties = properties;
                        self.maxOrder = self.properties['hips_order'];
                        self.frame = CooFrameEnum.fromString(self.properties['hips_frame']);

                        self._loadMetadata();
                    }, function(err) {
                        console.log('Could not find properties for HiPS ' + self.rootUrl);
                    }
                );
            }
        },

        _loadMetadata: function() {
            var self = this;
            $.ajax({
                url: self.rootUrl + '/' + 'Metadata.xml',
                method: 'GET',
                success: function(xml) {
                    self.fields = getFields(self, xml);
                    self._loadAllskyNewMethod();
                },
                error: function(err) {
                    self._loadAllskyOldMethod();
                }
            });
        },

        _loadAllskyNewMethod: function() {
            var self = this;
            $.ajax({
                url: self.rootUrl + '/' + 'Norder1/Allsky.tsv',
                method: 'GET',
                success: function(tsv) {
                    self.order1Sources = getSources(self, tsv, self.fields);

                    if (self.order2Sources) {
                        self.isReady = true;
                        self.view.requestRedraw();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });

            $.ajax({
                url: self.rootUrl + '/' + 'Norder2/Allsky.tsv',
                method: 'GET',
                success: function(tsv) {
                    self.order2Sources = getSources(self, tsv, self.fields);

                    if (self.order1Sources) {
                        self.isReady = true;
                        self.view.requestRedraw();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });

        },

        _loadAllskyOldMethod: function() {
            this.maxOrderAllsky = 3;
            this._loadLevel2Sources();
            this._loadLevel3Sources();
        },

        _loadLevel2Sources: function() {
            var self = this;
            $.ajax({
                url: self.rootUrl + '/' + 'Norder2/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.fields = getFields(self, xml);
                    self.order2Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    if (self.order3Sources) {
                        self.isReady = true;
                        self.view.requestRedraw();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        _loadLevel3Sources: function() {
            var self = this;
            $.ajax({
                url: self.rootUrl + '/' + 'Norder3/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.order3Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    if (self.order2Sources) {
                        self.isReady = true;
                        self.view.requestRedraw();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        draw: function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
            if (! this.isShowing || ! this.isReady) {
                return;
            }
            this.drawSources(this.order1Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            this.drawSources(this.order2Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            this.drawSources(this.order3Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            
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
            if (! sources) {
                return;
            }
            for (var k=0, len = sources.length; k<len; k++) {
                cds.Catalog.drawSource(this, sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor);
            }
            for (var k=0, len = sources.length; k<len; k++) {
                if (! sources[k].isSelected) {
                    continue;
                }
                cds.Catalog.drawSourceSelection(this, sources[k], ctx);
            }
        },

        getSources: function() {
            var ret = [];
            if (this.order1Sources) {
                ret = ret.concat(this.order1Sources);
            }
            if (this.order2Sources) {
                ret = ret.concat(this.order2Sources);
            }
            if (this.order3Sources) {
                ret = ret.concat(this.order3Sources);
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


        
        deselectAll: function() {
            if (this.order1Sources) {
                for (var k=0; k<this.order1Sources.length; k++) {
                    this.order1Sources[k].deselect();
                }
            }

            if (this.order2Sources) {
                for (var k=0; k<this.order2Sources.length; k++) {
                    this.order2Sources[k].deselect();
                }
            }

            if (this.order3Sources) {
                for (var k=0; k<this.order3Sources.length; k++) {
                    this.order3Sources[k].deselect();
                }
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
            if (norder<=this.maxOrderAllsky) {
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
                            // ATTENTIOn : je passe en JSON direct, car je n'arrive pas a choper les 404 en JSONP
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
        },

        reportChange: function() { // TODO: to be shared with Catalog
            this.view && this.view.requestRedraw();
        }
    

    }; // END OF .prototype functions
    
    
    return ProgressiveCat;
})();
    
