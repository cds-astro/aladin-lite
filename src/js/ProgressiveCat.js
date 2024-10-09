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
import { Catalog } from "./Catalog.js";
import { Source } from "./Source.js";
import { Color } from "./Color.js";
import { Coo } from "./libs/astro/coo.js";
import { Utils } from "./Utils";
import { CooFrameEnum } from "./CooFrameEnum.js";
// TODO: index sources according to their HEALPix ipix
// TODO : merge parsing with class Catalog
export let ProgressiveCat = (function() {
    
    // TODO : test if CORS support. If no, need to pass through a proxy
    // currently, we suppose CORS is supported
    
    // constructor
    let ProgressiveCat = function(rootUrl, frameStr, maxOrder, options) {
        options = options || {};

        this.uuid = Utils.uuidv4();
        this.type = 'progressivecat';
        
        this.rootUrl = rootUrl; // TODO: method to sanitize rootURL (absolute, no duplicate slashes, remove end slash if existing)
        // fast fix for HTTPS support --> will work for all HiPS served by CDS
        if (Utils.isHttpsContext() && ( /u-strasbg.fr/i.test(this.rootUrl) || /unistra.fr/i.test(this.rootUrl)  ) ) {
            this.rootUrl = this.rootUrl.replace('http://', 'https://');
        }

        this.frameStr = frameStr;
        this.frame = CooFrameEnum.fromString(frameStr) || CooFrameEnum.J2000;
        this.maxOrder = parseInt(maxOrder);
        this.isShowing = true; // TODO : inherit from catalogue

        this.name = options.name || "progressive-cat";
        this.color = options.color || Color.getNextColor();
        this.shape = options.shape || "square";
        this.sourceSize = options.sourceSize || 6;
        this.selectSize = this.sourceSize + 2;
        this.selectionColor = options.selectionColor || '#00ff00'; // TODO: to be merged with Catalog
        this.hoverColor = options.hoverColor || this.color;


        // allows for filtering of sources
        this.filterFn = options.filter || undefined; // TODO: do the same for catalog


        this.onClick = options.onClick || undefined; // TODO: inherit from catalog

        // we cache the list of sources in each healpix tile. Key of the cache is norder+'-'+npix
        this.sourcesCache = new Utils.LRUCache(256);
        this.footprintsCache = new Utils.LRUCache(256);

        //added to allow hips catalogue to also use shape functions
        this.updateShape(options);

        this.maxOrderAllsky = 2;
        this.isReady = false;

        this.tilesInView = [];
    };

    // TODO: to be put higher in the class diagram, in a HiPS generic class
    ProgressiveCat.readProperties = function(rootUrl, successCallback, errorCallback) {
        if (! successCallback) {
            return;
        }

        var propertiesURL = rootUrl + '/properties';
        Utils.fetch({
            url: propertiesURL,
            method: 'GET',
            dataType: 'text',
            success: function(propertiesTxt) {
                var props = {};
                var lines = propertiesTxt.split('\n');
                for (var k=0; k<lines.length; k++) {
                    var line = lines[k];
                    var idx = line.indexOf('=');
                    var propName  = line.substring(0, idx).trim();
                    var propValue = line.substring(idx + 1).trim();
                    
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
        xml.querySelectorAll("FIELD").forEach((field) => {
            var f = {};
            for (var i=0; i<attributes.length; i++) {
                var attribute = attributes[i];
                if (field.hasAttribute(attribute)) {
                    f[attribute] = field.getAttribute(attribute);
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
        var lines = csv.split('\n');
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
            newSource = new Source(ra, dec, mesures);
            sources.push(newSource);
            newSource.setCatalog(instance);
        }

        let footprints = instance.computeFootprints(sources);
        return [sources, footprints];
    };

    ProgressiveCat.prototype = {

        setView: function(view, idx) {
            var self = this;
            this.view = view;

            this.view.catalogs.push(this);
            this.view.insertOverlay(this, idx);

            if (this.maxOrder && this.frameStr) {
                this._loadMetadata();
            }

            else {
                ProgressiveCat.readProperties(self.rootUrl,
                    function (properties) {
                        self.properties = properties;
                        self.maxOrder = parseInt(self.properties['hips_order']);
                        self.frame = CooFrameEnum.fromString(self.properties['hips_frame']);

                        self._loadMetadata();
                    }, function(err) {
                        console.log('Could not find properties for HiPS ' + self.rootUrl);
                    }
                );
            }
        },

        updateShape: Catalog.prototype.updateShape,

        _loadMetadata: function() {
            var self = this;
            let request = new Request(self.rootUrl + '/' + 'Metadata.xml', {
                method: 'GET'
            })
            fetch(request)
                .then((resp) => resp.text())
                .then((text) => {
                    let xml = ProgressiveCat.parser.parseFromString(text, "text/xml")

                    self.fields = getFields(self, xml);
                    self._loadAllskyNewMethod();
                })
                .catch(err => self._loadAllskyOldMethod());
        },

        _loadAllskyNewMethod: function() {
            var self = this;
            Utils.fetch({
                desc: "Loading allsky tiles of: " + self.name,
                url: self.rootUrl + '/' + 'Norder1/Allsky.tsv',
                method: 'GET',
                success: function(tsv) {
                    let [sources, footprints] = getSources(self, tsv, self.fields);

                    self.order1Footprints = footprints;
                    self.order1Sources = sources;

                    if (self.order2Sources) {
                        self.isReady = true;
                        self._finishInitWhenReady();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });

            Utils.fetch({
                desc: "Loading allsky order 2 tiles of: " + self.name,
                url: self.rootUrl + '/' + 'Norder2/Allsky.tsv',
                method: 'GET',
                success: function(tsv) {
                    let [sources, footprints] = getSources(self, tsv, self.fields);

                    self.order2Footprints = footprints;
                    self.order2Sources = sources;

                    if (self.order1Sources) {
                        self.isReady = true;
                        self._finishInitWhenReady();
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
            Utils.fetch({
                desc: "Loading level 2 sources of: " + self.name,
                url: self.rootUrl + '/' + 'Norder2/Allsky.xml',
                method: 'GET',
                success: function(text) {
                    let xml = ProgressiveCat.parser.parseFromString(text, "text/xml")

                    self.fields = getFields(self, xml);

                    let [sources, footprints] = getSources(self, xml.querySelectorAll('CSV').innerText, self.fields);

                    self.order2Footprints = footprints
                    self.order2Sources = sources

                    if (self.order3Sources) {
                        self.isReady = true;
                        self._finishInitWhenReady();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        _loadLevel3Sources: function() {
            var self = this;
            Utils.fetch({
                desc: "Loading level 3 sources of: " + self.name,
                url: self.rootUrl + '/' + 'Norder3/Allsky.xml',
                method: 'GET',
                success: function(text) {
                    let xml = ProgressiveCat.parser.parseFromString(text, "text/xml")
                    let [sources, footprints] = getSources(self, xml.querySelectorAll('CSV').innerText, self.fields);
                    self.order3Footprints = footprints
                    self.order3Sources = sources

                    if (self.order2Sources) {
                        self.isReady = true;
                        self._finishInitWhenReady();
                    }
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        _finishInitWhenReady: function() {
            this.view.requestRedraw();
            this.loadNeededTiles();
        },

        draw: function(ctx, width, height) {
            if (! this.isShowing || ! this.isReady) {
                return;
            }

            if (this._shapeIsFunction) {
                ctx.save();
            }

            // Order must be >= 0
            if (this.order1Footprints) {
                this.order1Footprints.forEach((f) => {
                    f.draw(ctx, this.view);
                    f.source.tooSmallFootprint = f.isTooSmall();
                });
            }
            if (this.order1Sources) {
                this.drawSources(this.order1Sources, ctx, width, height);
            }

            if (this.view.realNorder >= 1) {
                if (this.order2Footprints) {
                    this.order2Footprints.forEach((f) => {
                        f.draw(ctx, this.view)
                        f.source.tooSmallFootprint = f.isTooSmall();
                    });
                }

                if (this.order2Sources) {
                    this.drawSources(this.order2Sources, ctx, width, height);
                }
            }

            // For old allsky, tilesInView refers to tiles at orders 4..
            // For new allsky, tilesInView will contains order3 sources
            if (this.maxOrderAllsky === 3) {
                if (this.view.realNorder >= 2) {
                    if (this.order3Footprints) {
                        this.order3Footprints.forEach((f) => {
                            f.draw(ctx, this.view)
                            f.source.tooSmallFootprint = f.isTooSmall();
                        });
                    }

                    if (this.order3Sources) {
                        this.drawSources(this.order3Sources, ctx, width, height);
                    }
                }
            }

            let key, sources, footprints;
            this.tilesInView.forEach((tile) => {
                key = tile[0] + '-' + tile[1];
                sources = this.sourcesCache.get(key);
                footprints = this.footprintsCache.get(key);

                if (footprints) {
                    footprints.forEach((f) => {
                        f.draw(ctx, this.view)
                        f.source.tooSmallFootprint = f.isTooSmall();
                    });
                }

                if (sources) {
                    this.drawSources(sources, ctx, width, height);
                }
            });

            if (this._shapeIsFunction) {
                ctx.restore();
            }
        },

        drawSources: function(sources, ctx, width, height) {
            let self = this;

            let ra = []
            let dec = [];

            sources.forEach((s) => {
                ra.push(s.ra);
                dec.push(s.dec);
            });

            let xy = this.view.wasm.worldToScreenVec(ra, dec);

            let drawSource = (s, idx) => {
                s.x = xy[2 * idx];
                s.y = xy[2 * idx + 1];
    
                self.drawSource(s, ctx, width, height);
            };

            sources.forEach(function(s, idx) {
                if (xy[2 * idx] && xy[2 * idx + 1]) {
                    if (self.filterFn) {
                        if(!self.filterFn(s)) {
                            s.hide()
                        } else {
                            s.show()
    
                            drawSource(s, idx)
                        }
                    } else {
                        drawSource(s, idx)
                    }
                }
            });
        },

        drawSource: Catalog.prototype.drawSource,

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

        getFootprints: function() {
            var ret = [];
            if (this.order1Footprints) {
                ret = ret.concat(this.order1Footprints);
            }
            if (this.order2Footprints) {
                ret = ret.concat(this.order2Footprints);
            }
            if (this.order3Footprints) {
                ret = ret.concat(this.order3Footprints);
            }
            if (this.tilesInView) {
                var footprints, key, t;
                for (var k=0; k < this.tilesInView.length; k++) {
                    t = this.tilesInView[k];
                    key = t[0] + '-' + t[1];
                    footprints = this.footprintsCache.get(key);

                    if (footprints) {
                        ret = ret.concat(footprints);
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

                var footprints = this.footprintsCache[key];
                for (var k=0; k<footprints.length; k++) {
                    footprints[k].deselect();
                }
            }
        },

        show: function() {
            if (this.isShowing) {
                return;
            }
            this.isShowing = true;
            this.loadNeededTiles();
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
            if ( ! this.isShowing) {
                return;
            }
            this.tilesInView = [];
            
            var norder = this.view.realNorder;
            if (norder > this.maxOrder) {
                norder = this.maxOrder;
            }

            var cells = this.view.getVisibleCells(norder);
            
            // Limit the number of cells to fetch by looking for smaller orders
            let customNorder = norder;
            while (cells.length > 12 && customNorder > this.maxOrderAllsky) {
                customNorder--;
                cells = this.view.getVisibleCells(customNorder);
            }

            norder = customNorder;
            if (norder<=this.maxOrderAllsky) {
                return; // nothing to do, hurrayh !
            }

            var ipixList, ipix;
            for (var curOrder=this.maxOrderAllsky+1; curOrder<=norder; curOrder++) {
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
            for (var k=0; k<this.tilesInView.length; k++) {
                t = this.tilesInView[k];
                key = t[0] + '-' + t[1]; // t[0] is norder, t[1] is ipix
                if (!this.sourcesCache.get(key)) {
                    (function(self, norder, ipix) { // wrapping function is needed to be able to retrieve norder and ipix in ajax success function
                        var key = norder + '-' + ipix;
                        Utils.fetch({
                            /*
                            url: Aladin.JSONP_PROXY,
                            data: {"url": self.getTileURL(norder, ipix)},
                            */
                            // ATTENTION : je passe en JSON direct, car je n'arrive pas a choper les 404 en JSONP
                            url: self.getTileURL(norder, ipix),
                            desc: "Get tile .tsv " + norder + ' ' + ipix + ' of ' + self.name,
                            method: 'GET',
                            //dataType: 'jsonp',
                            success: function(tsv) {
                                let [sources, footprints] = getSources(self, tsv, self.fields);

                                self.sourcesCache.set(key, sources);
                                self.footprintsCache.set(key, footprints);

                                self.view.requestRedraw();
                            },
                            error: function() {
                                // on suppose qu'il s'agit d'une erreur 404
                                self.sourcesCache.set(key, []);
                                self.footprintsCache.set(key, []);
                            }
                        });
                    })(this, t[0], t[1]);
                }
            }
        },

        computeFootprints: Catalog.prototype.computeFootprints,

        reportChange: function() { // TODO: to be shared with Catalog
            this.view && this.view.requestRedraw();
        }
    

    }; // END OF .prototype functions
    
    ProgressiveCat.parser = new DOMParser();
    
    return ProgressiveCat;
})();
    
