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
 * File Catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// TODO : harmoniser parsing avec classe ProgressiveCat
cds.Catalog = (function() {
   cds.Catalog = function(options) {
        options = options || {};
        this.type = 'catalog';
    	this.name = options.name || "catalog";
    	this.color = options.color || Color.getNextColor();
    	this.sourceSize = options.sourceSize || 6;
    	this.markerSize = options.sourceSize || 12;
    	this.shape = options.shape || "square";
        this.maxNbSources = options.limit || undefined;
        this.onClick = options.onClick || undefined;

        this.displayLabel = options.displayLabel || false;
        this.labelColor = options.labelColor || this.color;
        this.labelFont = options.labelFont || '10px sans-serif';
        if (this.displayLabel) {
            this.labelColumn = options.labelColumn;
            if (!this.labelColumn) {
                this.displayLabel = false;
            }
        }
    	
        if (this.shape instanceof Image) {
            this.sourceSize = this.shape.width;
        }
        this.selectSize = this.sourceSize + 2;
        
        this.isShowing = true;

    	
    	this.indexationNorder = 5; // à quel niveau indexe-t-on les sources
    	this.sources = [];
    	this.hpxIdx = new HealpixIndex(this.indexationNorder);
    	this.hpxIdx.init();
    	this.selectionColor = '#00ff00';
    	
    	
    	// cacheCanvas permet de ne créer le path de la source qu'une fois, et de le réutiliser (cf. http://simonsarris.com/blog/427-increasing-performance-by-caching-paths-on-canvas)
        this.cacheCanvas = cds.Catalog.createShape(this.shape, this.color, this.sourceSize); 

        this.cacheMarkerCanvas = document.createElement('canvas');
        this.cacheMarkerCanvas.width = this.markerSize;
        this.cacheMarkerCanvas.height = this.markerSize;
        var cacheMarkerCtx = this.cacheMarkerCanvas.getContext('2d');
        cacheMarkerCtx.fillStyle = this.color;
        cacheMarkerCtx.beginPath();
        var half = (this.markerSize)/2.;
        cacheMarkerCtx.arc(half, half, half-2, 0, 2 * Math.PI, false);
        cacheMarkerCtx.fill();
        cacheMarkerCtx.lineWidth = 2;
        cacheMarkerCtx.strokeStyle = '#ccc';
        cacheMarkerCtx.stroke();
        
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
        cacheSelectCtx.stroke();

    };
    
    cds.Catalog.createShape = function(shapeName, color, sourceSize) {
        if (shapeName instanceof Image) { // 
            return shapeName;
        }
        var c = document.createElement('canvas');
        c.width = c.height = sourceSize;
        var ctx= c.getContext('2d');
        ctx.beginPath();
        ctx.strokeStyle = color;
        ctx.lineWidth = 2.0;
        if (shapeName=="plus") {
            ctx.moveTo(sourceSize/2., 0);
            ctx.lineTo(sourceSize/2., sourceSize);
            ctx.stroke();
            
            ctx.moveTo(0, sourceSize/2.);
            ctx.lineTo(sourceSize, sourceSize/2.);
            ctx.stroke();
        }
        else if (shapeName=="cross") {
            ctx.moveTo(0, 0);
            ctx.lineTo(sourceSize-1, sourceSize-1);
            ctx.stroke();
            
            ctx.moveTo(sourceSize-1, 0);
            ctx.lineTo(0, sourceSize-1);
            ctx.stroke();
        }
        else if (shapeName=="rhomb") {
            ctx.moveTo(sourceSize/2, 0);
            ctx.lineTo(0, sourceSize/2);
            ctx.lineTo(sourceSize/2, sourceSize);
            ctx.lineTo(sourceSize, sourceSize/2);
            ctx.lineTo(sourceSize/2, 0);
            ctx.stroke();
        }
        else if (shapeName=="triangle") {
            ctx.moveTo(sourceSize/2, 0);
            ctx.lineTo(0, sourceSize-1);
            ctx.lineTo(sourceSize-1, sourceSize-1);
            ctx.lineTo(sourceSize/2, 0);
            ctx.stroke();
        }
        else { // default shape: square
            ctx.moveTo(0, 0);
            ctx.lineTo(0,  sourceSize);
            ctx.lineTo( sourceSize,  sourceSize);
            ctx.lineTo( sourceSize, 0);
            ctx.lineTo(0, 0);
            ctx.stroke();
        }
        
        return c;
        
    };
    

    
    
    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed
    cds.Catalog.parseVOTable = function(url, callback, maxNbSources, useProxy) {
        
        function doParseVOTable(xml, callback) {
            xml = xml.replace(/^\s+/g, ''); // we need to trim whitespaces at start of document
            var attributes = ["name", "ID", "ucd", "utype", "unit", "datatype", "arraysize", "width", "precision"];
            
            var fields = [];
            var k = 0;
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
                fields.push(f);
                k++;
            });
                
            // find RA/DEC fields
            var raFieldIdx,  decFieldIdx;
            raFieldIdx = decFieldIdx = null;
            for (var l=0, len=fields.length; l<len; l++) {
                var field = fields[l];
                if ( ! raFieldIdx) {
                    if (field.ucd) {
                        var ucd = field.ucd.toLowerCase();
                        if (ucd.indexOf('pos.eq.ra')>=0 || ucd.indexOf('pos_eq_ra')>=0) {
                            raFieldIdx = l;
                            continue;
                        }
                    }
                }
                    
                if ( ! decFieldIdx) {
                    if (field.ucd) {
                        var ucd = field.ucd.toLowerCase();
                        if (ucd.indexOf('pos.eq.dec')>=0 || ucd.indexOf('pos_eq_dec')>=0) {
                            decFieldIdx = l;
                            continue;
                        }
                    }
                }
            }
            var sources = [];
            
            var coo = new Coo();
            var ra, dec;
            $(xml).find("TR").each(function() {
               var mesures = {};
               var k = 0;
               $(this).find("TD").each(function() {
                   var key = fields[k].name ? fields[k].name : fields[k].id;
                   mesures[key] = $(this).text();
                   k++;
               });
               var keyRa = fields[raFieldIdx].name ? fields[raFieldIdx].name : fields[raFieldIdx].id;
               var keyDec = fields[decFieldIdx].name ? fields[decFieldIdx].name : fields[decFieldIdx].id;

               if (Utils.isNumber(mesures[keyRa]) && Utils.isNumber(mesures[keyDec])) {
                   ra = parseFloat(mesures[keyRa]);
                   dec = parseFloat(mesures[keyDec]);
               }
               else {
                   coo.parse(mesures[keyRa] + " " + mesures[keyDec]);
                   ra = coo.lon;
                   dec = coo.lat;
               }
               sources.push(new cds.Source(ra, dec, mesures));
               if (maxNbSources && sources.length==maxNbSources) {
                   return false; // break the .each loop
               }
                
            });
            if (callback) {
                callback(sources);
            }
        }
        
        var ajax = Utils.getAjaxObject(url, 'GET', 'text', useProxy);
        ajax.done(function(xml) {
            doParseVOTable(xml, callback);
        });
    };
    
    cds.Catalog.prototype.addSources = function(sourcesToAdd) {
    	this.sources = this.sources.concat(sourcesToAdd);
    	for (var k=0, len=sourcesToAdd.length; k<len; k++) {
    	    sourcesToAdd[k].setCatalog(this);
    	}
        this.view.requestRedraw();
    };
    
    // return the currnet list of Source objects
    cds.Catalog.prototype.getSources = function() {
        return this.sources;
    };
    
    // TODO : fonction générique traversant la liste des sources
    cds.Catalog.prototype.selectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].select();
        }
    };
    
    cds.Catalog.prototype.deselectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].deselect();
        }
    };
    
    // return a source by index
    cds.Catalog.prototype.getSource = function(idx) {
        if (idx<this.sources.length) {
            return this.sources[idx];
        }
        else {
            return null;
        }
    };
    
    cds.Catalog.prototype.setView = function(view) {
        this.view = view;
    };
    
    cds.Catalog.prototype.removeAll = cds.Catalog.prototype.clear = function() {
        // TODO : RAZ de l'index
        this.sources = [];
    };
    
    cds.Catalog.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! this.isShowing) {
            return;
        }
        // tracé simple
        //ctx.strokeStyle= this.color;

        //ctx.lineWidth = 1;
    	//ctx.beginPath();
    	for (var k=0, len = this.sources.length; k<len; k++) {
    		this.drawSource(this.sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor);
    	}
        //ctx.stroke();

    	// tracé sélection
        ctx.strokeStyle= this.selectionColor;
        //ctx.beginPath();
        for (var k=0, len = this.sources.length; k<len; k++) {
            if (! this.sources[k].isSelected) {
                continue;
            }
            this.drawSourceSelection(this.sources[k], ctx);
            
        }
        // NEEDED ?
    	//ctx.stroke();

        // tracé label
        if (this.displayLabel) {
            ctx.fillStyle = this.labelColor;
            ctx.font = this.labelFont;
            for (var k=0, len = this.sources.length; k<len; k++) {
                this.drawSourceLabel(this.sources[k], ctx);
            }
        }
    };
    
    
    
    cds.Catalog.prototype.drawSource = function(s, ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! s.isShowing) {
            return;
        }
        var sourceSize = this.sourceSize;
        // TODO : we could factorize this code with Aladin.world2pix
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
            var max = s.popup ? 100 : s.sourceSize;
            if (xyview) {
                // TODO : index sources
                // check if source is visible in view ?
                if (xyview.vx>(width+max)  || xyview.vx<(0-max) ||
                    xyview.vy>(height+max) || xyview.vy<(0-max)) {
                    s.x = s.y = undefined;
                    return;
                }
                
                s.x = xyview.vx;
                s.y = xyview.vy;

                if (s.marker) {
                    ctx.drawImage(this.cacheMarkerCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
                }
                else {
                    ctx.drawImage(this.cacheCanvas, s.x-this.cacheCanvas.width/2, s.y-this.cacheCanvas.height/2);
                }


                // has associated popup ?
                if (s.popup) {
                    s.popup.setPosition(s.x, s.y);
                }
                
                
            }
        }
    };
    
    cds.Catalog.prototype.drawSourceSelection = function(s, ctx) {
        if (!s || !s.isShowing || !s.x || !s.y) {
            return;
        }
        var sourceSize = this.selectSize;
        
        ctx.drawImage(this.cacheSelectCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
    };

    cds.Catalog.prototype.drawSourceLabel = function(s, ctx) {
        if (!s || !s.isShowing || !s.x || !s.y) {
            return;
        }

        var label = s.data[this.labelColumn];
        if (!label) {
            return;
        }

        ctx.fillText(label, s.x, s.y);
    };

    
    // callback function to be called when the status of one of the sources has changed
    cds.Catalog.prototype.reportChange = function() {
        this.view.requestRedraw();
    };
    
    cds.Catalog.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };
    
    cds.Catalog.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.view && this.view.popup && this.view.popup.source && this.view.popup.source.catalog==this) {
            this.view.popup.hide();
        }
        this.reportChange();
    };

    return cds.Catalog;
})();
