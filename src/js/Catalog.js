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

import { Source } from "./Source.js"
import { Color } from "./Color.js"
import { Utils } from "./Utils.js";
import { AladinUtils } from "./AladinUtils.js";
import { Coo } from "./libs/astro/coo.js";
import { ALEvent } from "./events/ALEvent.js";
import { Obscore } from "./vo/Obscore.js";
import { VOTable } from "./vo/VOTable.js";

import $ from 'jquery';

// TODO : harmoniser parsing avec classe ProgressiveCat
export let Catalog = (function() {

   function Catalog(options) {
        let self = this;
        options = options || {};

        this.uuid = Utils.uuidv4();
        this.type = 'catalog';
        this.name = options.name || "catalog";
    	this.color = options.color || Color.getNextColor();
    	this.sourceSize = options.sourceSize || 8;
    	this.markerSize = options.sourceSize || 12;
        this.selectSize = this.sourceSize;
    	this.shape = options.shape || "square";
        this.maxNbSources = options.limit || undefined;
        this.onClick = options.onClick || undefined;

        this.raField = options.raField || undefined; // ID or name of the field holding RA
        this.decField = options.decField || undefined; // ID or name of the field holding dec

        this.fieldsClickCallbacks = {}; // callbacks when the user clicks on a cell in the measurement table associated

    	this.indexationNorder = 5; // à quel niveau indexe-t-on les sources
    	this.sources = [];
        this.footprints = [];

        this.displayLabel = options.displayLabel || false;
        this.labelColor = options.labelColor || this.color;
        this.labelFont = options.labelFont || '10px sans-serif';
        if (this.displayLabel) {
            this.labelColumn = options.labelColumn;
            if (!this.labelColumn) {
                this.displayLabel = false;
            }
        }
    	
        if (this.shape instanceof Image || this.shape instanceof HTMLCanvasElement) {
            this.sourceSize = this.shape.width;
        }
        this._shapeIsFunction = false; // if true, the shape is a function drawing on the canvas
        if ($.isFunction(this.shape)) {
            this._shapeIsFunction = true;
        }
        
    	this.selectionColor = '#00ff00';
    	

        // create this.cacheCanvas    	
    	// cacheCanvas permet de ne créer le path de la source qu'une fois, et de le réutiliser (cf. http://simonsarris.com/blog/427-increasing-performance-by-caching-paths-on-canvas)
        this.updateShape(options);

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

        this.isShowing = true;
    };
    
    Catalog.createShape = function(shapeName, color, sourceSize) {
        if (shapeName instanceof Image || shapeName instanceof HTMLCanvasElement) { // in this case, the shape is already created
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
        else if (shapeName=="circle") {
            ctx.arc(sourceSize/2, sourceSize/2, sourceSize/2 - 1, 0, 2*Math.PI, true);
            ctx.stroke();
        }
        else { // default shape: square
            ctx.moveTo(1, 0);
            ctx.lineTo(1,  sourceSize-1);
            ctx.lineTo( sourceSize-1,  sourceSize-1);
            ctx.lineTo( sourceSize-1, 1);
            ctx.lineTo(1, 1);
            ctx.stroke();
        }
        
        return c;
    };
    

        // find RA, Dec fields among the given fields
        //
        // @param fields: list of objects with ucd, unit, ID, name attributes
        // @param raField:  index or name of right ascension column (might be undefined)
        // @param decField: index or name of declination column (might be undefined)
        //
        function findRADecFields(fields, raField, decField) {
            var raFieldIdx,  decFieldIdx;
            raFieldIdx = decFieldIdx = null;

            // first, look if RA/DEC fields have been already given
            if (raField) { // ID or name of RA field given at catalogue creation
                for (var l=0, len=fields.length; l<len; l++) {
                    var field = fields[l];
                    if (Utils.isInt(raField) && raField<fields.length) { // raField can be given as an index
                        raFieldIdx = raField;
                        break;
                    } 
                    if ( (field.ID && field.ID===raField) || (field.name && field.name===raField)) {
                        raFieldIdx = l;
                        break;
                    }
                }
            }
            if (decField) { // ID or name of dec field given at catalogue creation
                for (var l=0, len=fields.length; l<len; l++) {
                    var field = fields[l];
                    if (Utils.isInt(decField) && decField<fields.length) { // decField can be given as an index
                        decFieldIdx = decField;
                        break;
                    } 
                    if ( (field.ID && field.ID===decField) || (field.name && field.name===decField)) {
                        decFieldIdx = l;
                        break;
                    }
                }
            }
            // if not already given, let's guess position columns on the basis of UCDs
            for (var l=0, len=fields.length; l<len; l++) {
                if (raFieldIdx!=null && decFieldIdx!=null) {
                    break;
                }

                var field = fields[l];
                if ( ! raFieldIdx) {
                    if (field.ucd) {
                        var ucd = $.trim(field.ucd.toLowerCase());
                        if (ucd.indexOf('pos.eq.ra')==0 || ucd.indexOf('pos_eq_ra')==0) {
                            raFieldIdx = l;
                            continue;
                        }
                    }
                }
                    
                if ( ! decFieldIdx) {
                    if (field.ucd) {
                        var ucd = $.trim(field.ucd.toLowerCase());
                        if (ucd.indexOf('pos.eq.dec')==0 || ucd.indexOf('pos_eq_dec')==0) {
                            decFieldIdx = l;
                            continue;
                        }
                    }
                }
            }

            // still not found ? try some common names for RA and Dec columns
            if (raFieldIdx==null && decFieldIdx==null) {
                for (var l=0, len=fields.length; l<len; l++) {
                    var field = fields[l];
                    var name = field.name || field.ID || '';
                    name = name.toLowerCase();
                    
                    if ( ! raFieldIdx) {
                        if (name.indexOf('ra')==0 || name.indexOf('_ra')==0 || name.indexOf('ra(icrs)')==0 || name.indexOf('_ra')==0 || name.indexOf('alpha')==0) {
                            raFieldIdx = l;
                            continue;
                        }
                    }

                    if ( ! decFieldIdx) {
                        if (name.indexOf('dej2000')==0 || name.indexOf('_dej2000')==0 || name.indexOf('de')==0 || name.indexOf('de(icrs)')==0 || name.indexOf('_de')==0 || name.indexOf('delta')==0) {
                            decFieldIdx = l;
                            continue;
                        }
                    }
                    
                }
            }

            // last resort: take two first fieds
            if (raFieldIdx==null || decFieldIdx==null) {
                raFieldIdx  = 0;
                decFieldIdx = 1
            }

            return [raFieldIdx, decFieldIdx];
        };
        
    
    Catalog.parseFields = function(fields, raField, decField) {
        // This votable is not an obscore one
        let [raFieldIdx, decFieldIdx] = findRADecFields(fields, raField, decField);

        let parsedFields = {};
        let fieldIdx = 0;
        fields.forEach((field) => {
            let key = field.name ? field.name : field.id;

            let nameField;
            if (fieldIdx == raFieldIdx) {
                nameField = 'ra';
            } else if (fieldIdx == decFieldIdx) {
                nameField = 'dec';
            } else {
                nameField = key;
            }

            parsedFields[nameField] = {
                name: key,
                idx: fieldIdx,
            };

            fieldIdx++;
        })

        return parsedFields;
    };

    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed
    Catalog.parseVOTable = function(url, callback, maxNbSources, useProxy, raField, decField) {
        VOTable.parse(
            url,
            (fields, rows) => {
                let sources = [];
                let footprints = [];

                rows.every(row => {
                    let ra, dec, region;
                    var mesures = {};

                    for (const [fieldName, field] of Object.entries(fields)) {
                        if (fieldName === 's_region') {
                            // Obscore s_region param
                            region = row[field.idx];
                        } else if (fieldName === 'ra' || fieldName === 's_ra') {
                            ra = row[field.idx]
                        } else if (fieldName === 'dec' || fieldName === 's_dec') {
                            dec = row[field.idx]
                        }

                        var key = field.name;
                        mesures[key] = row[field.idx];
                    }

                    if (ra && dec) {
                        const source = new Source(ra, dec, mesures);
                        
                        sources.push(source);
                        if (maxNbSources && sources.length == maxNbSources) {
                            return false;
                        }
                    }

                    if (region) {
                        let footprint = A.footprintsFromSTCS(region, {lineWidth: 2});
                        footprints = footprints.concat(footprint);
                    }

                    return true;
                })

                if (callback) {
                    callback(sources, footprints, fields);
                }
            },
            raField,
            decField
        )
    };

    Catalog.prototype.addFieldClickCallback = function(field, callback) {
        this.fieldsClickCallbacks[field] = callback;
    }

    // API
    Catalog.prototype.updateShape = function(options) {
        options = options || {};
    	this.color = options.color || this.color || Color.getNextColor();
    	this.sourceSize = options.sourceSize || this.sourceSize || 6;
    	this.shape = options.shape || this.shape || "square";

        this.selectSize = this.sourceSize + 2;

        this.cacheCanvas = Catalog.createShape(this.shape, this.color, this.sourceSize); 
        this.cacheSelectCanvas = Catalog.createShape('square', this.selectionColor, this.selectSize);

        this.reportChange();
    };
    
    // API
    Catalog.prototype.addSources = function(sourcesToAdd) {
        sourcesToAdd = [].concat(sourcesToAdd); // make sure we have an array and not an individual source
    	this.sources = this.sources.concat(sourcesToAdd);
    	for (var k=0, len=sourcesToAdd.length; k<len; k++) {
    	    sourcesToAdd[k].setCatalog(this);
    	}
        this.reportChange();
    };

    Catalog.prototype.addFootprints = function(footprintsToAdd) {
        footprintsToAdd = [].concat(footprintsToAdd); // make sure we have an array and not an individual footprints
    	this.footprints = this.footprints.concat(footprintsToAdd);
    	for (var k=0, len=footprintsToAdd.length; k<len; k++) {
    	    footprintsToAdd[k].setOverlay(this);
    	}
        this.reportChange();
    };

    // API
    //
    // create sources from a 2d array and add them to the catalog
    //
    // @param columnNames: array with names of the columns
    // @array: 2D-array, each item being a 1d-array with the same number of items as columnNames
    Catalog.prototype.addSourcesAsArray = function(columnNames, array) {
        var fields = [];
        for (var colIdx=0 ; colIdx<columnNames.length; colIdx++) {
            fields.push({name: columnNames[colIdx]});
        }
        var raDecFieldIdxes = findRADecFields(fields, this.raField, this.decField);
        var raFieldIdx,  decFieldIdx;
        raFieldIdx = raDecFieldIdxes[0];
        decFieldIdx = raDecFieldIdxes[1];


        var newSources = [];
        var coo = new Coo();
        var ra, dec, row, dataDict;
        for (var rowIdx=0 ; rowIdx<array.length ; rowIdx++) {
            row = array[rowIdx];
            if (Utils.isNumber(row[raFieldIdx]) && Utils.isNumber(row[decFieldIdx])) {
                   ra = parseFloat(row[raFieldIdx]);
                   dec = parseFloat(row[decFieldIdx]);
            }
               else {
                   coo.parse(row[raFieldIdx] + " " + row[decFieldIdx]);
                   ra = coo.lon;
                   dec = coo.lat;
               }

            dataDict = {};
            for (var colIdx=0 ; colIdx<columnNames.length; colIdx++) {
                dataDict[columnNames[colIdx]] = row[colIdx];
            }

            newSources.push(A.source(ra, dec, dataDict));
        }

        this.addSources(newSources);
    };
    
    // return the current list of Source objects
    Catalog.prototype.getSources = function() {
        return this.sources;
    };
    
    // TODO : fonction générique traversant la liste des sources
    Catalog.prototype.selectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].select();
        }
    };
    
    Catalog.prototype.deselectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].deselect();
        }
    };
    
    // return a source by index
    Catalog.prototype.getSource = function(idx) {
        if (idx<this.sources.length) {
            return this.sources[idx];
        }
        else {
            return null;
        }
    };
    
    Catalog.prototype.setView = function(view) {
        this.view = view;
        this.reportChange();
    };

    // remove a source
    Catalog.prototype.remove = function(source) {
        var idx = this.sources.indexOf(source);
        if (idx<0) {
            return;
        }

        this.sources[idx].deselect();
        this.sources.splice(idx, 1);

        this.reportChange();
    };
    
    Catalog.prototype.removeAll = Catalog.prototype.clear = function() {
        // TODO : RAZ de l'index
        this.sources = [];
    };
    
    Catalog.prototype.draw = function(ctx, frame, width, height, largestDim, zoomFactor) {
        if (! this.isShowing) {
            return;
        }
        // tracé simple
        ctx.strokeStyle= this.color;

        //ctx.lineWidth = 1;
    	//ctx.beginPath();
        if (this._shapeIsFunction) {
            ctx.save();
        }

        const sourcesInView = this.drawSources(ctx, width, height);

        if (this._shapeIsFunction) {
            ctx.restore();
        }

        // tracé label
        if (this.displayLabel) {
            ctx.fillStyle = this.labelColor;
            ctx.font = this.labelFont;
            sourcesInView.forEach((s) => {
                Catalog.drawSourceLabel(this, s, ctx);
            })
        }

        // Draw the footprints
        this.drawFootprints(ctx);
    };
    
    Catalog.prototype.drawSources = function(ctx, width, height) {
        let sourcesInView = [];

        this.sources.forEach((s) => {
            if (this.drawSource(s, ctx, width, height)) {
                sourcesInView.push(s);
            }
        });

        return sourcesInView;
    };

    Catalog.prototype.drawSource = function(s, ctx, width, height) {
        if (!s.isShowing) {
            return false;
        }

        var xy = AladinUtils.radecToViewXy(s.ra, s.dec, this.view);

        if (xy) {
            var max = s.popup ? 100 : this.sourceSize;
            // TODO : index sources by HEALPix cells at level 3, 4 ?

            // check if source is visible in view
            if (xy[0]>(width+max) || xy[0]<(0-max) ||
                xy[1]>(height+max) || xy[1]<(0-max)) {
                s.x = s.y = undefined;

                return false;
            } else {
                s.x = xy[0];
                s.y = xy[1];
                if (this._shapeIsFunction) {
                    this.shape(s, ctx, this.view.getViewParams());
                }
                else if (s.marker && s.useMarkerDefaultIcon) {
                    ctx.drawImage(this.cacheMarkerCanvas, s.x-this.sourceSize/2, s.y-this.sourceSize/2);
                }
                else if (s.isSelected) {
                    ctx.drawImage(this.cacheSelectCanvas, s.x-this.selectSize/2, s.y-this.selectSize/2);
                }
                else {
                    ctx.drawImage(this.cacheCanvas, s.x-this.cacheCanvas.width/2, s.y-this.cacheCanvas.height/2);
                }

                // has associated popup ?
                if (s.popup) {
                    s.popup.setPosition(s.x, s.y);
                }

                return true;
            }
        }

        return false;
    };

    Catalog.prototype.drawSourceLabel = function(s, ctx) {
        if (!s || !s.isShowing || !s.x || !s.y) {
            return;
        }

        var label = s.data[this.labelColumn];
        if (!label) {
            return;
        }

        ctx.fillText(label, s.x, s.y);
    };

    Catalog.prototype.drawFootprints = function(ctx) {
        this.footprints.forEach((f) => {
            f.draw(ctx, this.view)
        });
    };


    // callback function to be called when the status of one of the sources has changed
    Catalog.prototype.reportChange = function() {
        this.view && this.view.requestRedraw();
    };
    
    Catalog.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };
    
    Catalog.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.view && this.view.popup && this.view.popup.source && this.view.popup.source.catalog==this) {
            this.view.popup.hide();
        }

        this.reportChange();
    };

    return Catalog;
})();
