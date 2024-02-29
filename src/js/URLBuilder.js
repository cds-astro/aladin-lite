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
 * File URLBuilder
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
import { Coo } from './libs/astro/coo.js';
import { Utils } from './Utils';
export let URLBuilder = (function() {    

    let URLBuilder = {
        buildSimbadCSURL: function(target, radiusDegrees) {
            if (target && (typeof target  === "object")) {
                if ('ra' in target && 'dec' in target) {
                    var coo = new Coo(target.ra, target.dec, 7);
                    target = coo.format('s');
                }
            }
            return 'https://alasky.unistra.fr/cgi/simbad-flat/simbad-cs.py?target=' + encodeURIComponent(target) + '&SR=' + radiusDegrees + '&format=votable&SRUNIT=deg&SORTBY=nbref';
        },

        buildNEDPositionCSURL: function(ra, dec, radiusDegrees) {
                return 'https://ned.ipac.caltech.edu/cgi-bin/nph-objsearch?search_type=Near+Position+Search&of=xml_main&RA=' + ra + '&DEC=' + dec + '&SR=' + radiusDegrees;
        },

        buildNEDObjectCSURL: function(object, radiusDegrees) {
                return 'https://ned.ipac.caltech.edu/cgi-bin/nph-objsearch?search_type=Near+Name+Search&radius=' + (60 * radiusDegrees) + '&of=xml_main&objname=' + object;
        },

        buildSKAORucioCSURL: function(target, radiusDegrees) {
            let posParam;
            if (target && (typeof target === "object")) {
                if ('ra' in target && 'dec' in target) {
                    posParam = encodeURIComponent(target.ra) + '%20' + encodeURIComponent(target.dec);
                }
            }
            else {
                var isObjectName = /[a-zA-Z]/.test(target);
                if (isObjectName) {
                    posParam = encodeURIComponent(target);
                }
                else {
                    var coo = new Coo();
                    coo.parse(target);
                    posParam = encodeURIComponent(coo.lon) + '%20' + encodeURIComponent(coo.lat);
                }
            }

            if (posParam) {
                return 'https://ivoa.dachs.srcdev.skao.int/rucio/rucio/cone/form?__nevow_form__=genForm&hscs_pos=' + posParam + '&hscs_sr=' + encodeURIComponent(radiusDegrees * 60) + '&_DBOPTIONS_ORDER=_r&_DBOPTIONS_DIR=ASC&MAXREC=100&_FORMAT=VOTable&submit=Go';
            }
        },

        buildVizieRCSURL: function(vizCatId, target, radiusDegrees, options) {
            if (target && (typeof target  === "object")) {
                if ('ra' in target && 'dec' in target) {
                    var coo = new Coo(target.ra, target.dec, 7);

                    target = coo.format('s');
                }
            }

            var maxNbSources = 1e5;
            if (options && options.hasOwnProperty('limit') && Utils.isNumber(options.limit)) {
                maxNbSources = parseInt(options.limit);
            }

            let url = 'https://vizier.unistra.fr/viz-bin/votable?-source=' + vizCatId + '&-c=' + encodeURIComponent(target)+ '&-out.max=' + maxNbSources + '&-c.rd=' + radiusDegrees;

            // request the `s_region` column usually found in ObsCore tables
            url = url + '&-out.add=s_region';
            // request the `s_fov` column usually found in ObsCore tables
            url = url + '&-out.add=s_fov';

            return url;
            //return 'https://vizier.unistra.fr/viz-bin/conesearch/' + vizCatId + '?ra=' + target.ra + '&dec=' + target.dec + '&sr=' + radiusDegrees;
        },

        buildSkyBotCSURL: function(ra, dec, radius, epoch, queryOptions) {
            var url = 'http://vo.imcce.fr/webservices/skybot/skybotconesearch_query.php?-from=AladinLite';
            url += '&RA=' + encodeURIComponent(ra);
            url += '&DEC=' + encodeURIComponent(dec);
            url += '&SR=' + encodeURIComponent(radius);
            url += '&EPOCH=' + encodeURIComponent(epoch);

            if (queryOptions) {
                for (var key in queryOptions) {
                    if (queryOptions.hasOwnProperty(key)) {
                            url += '&' + key + '=' + encodeURIComponent(queryOptions[key]);
                    }
                }
            }

            return url;
        }
    };

    return URLBuilder;
    
})();

