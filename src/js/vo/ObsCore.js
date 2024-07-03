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
 * File ObsCore
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/
 import { Datalink } from "./Datalink.js";
 import { Utils } from '../Utils';

 import { ActionButton } from "../gui/Widgets/ActionButton.js";

 export let ObsCore = (function() {

    // dict of mandatory ObsCore fields
    ObsCore.MANDATORY_FIELDS = {
        'dataproduct_type': { name: 'dataproduct_type', ucd: 'meta.id', utype: 'ObsDataset.dataProductType', units: null },
        'calib_level': { name: 'calib_level', ucd: 'meta.code;obs.calib', utype: 'ObsDataset.calibLevel', units: null },
        'obs_collection': { name: 'obs_collection', ucd: 'meta.id', utype: 'DataID.collection', units: null },
        'obs_id': { name: 'obs_id', ucd: 'meta.id', utype: 'DataID.observationID', units: null },
        'obs_publisher_did': { name: 'obs_publisher_did', ucd: 'meta.ref.uri;meta.curation', utype: 'Curation.publisherDID', units: null },
        'access_url': { name: 'access_url', ucd: 'meta.ref.url', utype: 'Access.reference', units: null },
        'access_format': { name: 'access_format', ucd: 'meta.code.mime', utype: 'Access.format', units: null },
        'access_estsize': { name: 'access_estsize', ucd: 'phys.size;meta.file', utype: 'Access.size', units: 'kbyte' },
        'target_name': { name: 'target_name', ucd: 'meta.id;src', utype: 'Target.name', units: null },
        's_ra': { name: 's_ra', ucd: 'pos.eq.ra', utype: 'Char.SpatialAxis.Coverage.Location.Coord.Position2D.Value2.C1', units: 'deg' },
        's_dec': { name: 's_dec', ucd: 'pos.eq.dec', utype: 'Char.SpatialAxis.Coverage.Location.Coord.Position2D.Value2.C2', units: 'deg' },
        's_fov': { name: 's_fov', ucd: 'phys.angSize;instr.fov', utype: 'Char.SpatialAxis.Coverage.Bounds.Extent.diameter', units: 'deg' },
        's_region': { name: 's_region', ucd: 'pos.outline;obs.field', utype: 'Char.SpatialAxis.Coverage.Support.Area', units: null },
        's_resolution': { name: 's_resolution', ucd: 'pos.angResolution', utype: 'Char.SpatialAxis.Resolution.Refval.value', units: 'arcsec' },
        's_xel1': { name: 's_xel1', ucd: 'meta.number', utype: 'Char.SpatialAxis.numBins1', units: null },
        's_xel2': { name: 's_xel2', ucd: 'meta.number', utype: 'Char.SpatialAxis.numBins2', units: null },
        
        't_min': { name: 't_min', ucd: 'time.start;obs.exposure', utype: 'Char.TimeAxis.Coverage.Bounds.Limits.StartTime', units: 'd' },
        't_max': { name: 't_max', ucd: 'time.end;obs.exposure', utype: 'Char.TimeAxis.Coverage.Bounds.Limits.StopTime', units: 'd' },
        't_exptime': { name: 't_exptime', ucd: 'time.duration;obs.exposure', utype: 'Char.TimeAxis.Coverage.Support.Extent', units: 's' },
        't_resolution': { name: 't_resolution', ucd: 'time.resolution', utype: 'Char.TimeAxis.Resolution.Refval.value', units: 's' },
        't_xel': { name: 't_xel', ucd: 'meta.number', utype: 'Char.TimeAxis.numBins', units: null },
        
        'em_min': { name: 'em_min', ucd: 'em.wl;stat.min', utype: 'Char.SpectralAxis.Coverage.Bounds.Limits.LoLimit', units: 'm' },
        'em_max': { name: 'em_max', ucd: 'em.wl;stat.max', utype: 'Char.SpectralAxis.Coverage.Bounds.Limits.HiLimit', units: 'm' },
        'em_res_power': { name: 'em_res_power', ucd: 'spect.resolution', utype: 'Char.SpectralAxis.Resolution.ResolPower.refVal', units: null },
        'em_xel': { name: 'em_xel', ucd: 'meta.number', utype: 'Char.SpectralAxis.numBins', units: null },

        'o_ucd': { name: 'o_ucd', ucd: 'meta.ucd', utype: 'Char.ObservableAxis.ucd', units: null },
        'pol_states': { name: 'pol_states', ucd: 'meta.code;phys.polarization', utype: 'Char.PolarizationAxis.stateList', units: null },
        'pol_xel': { name: 'pol_xel', ucd: 'meta.number', utype: 'Char.PolarizationAxis.numBins', units: null },
        'facility_name': { name: 'facility_name', ucd: 'meta.id;instr.tel', utype: 'Provenance.ObsConfig.Facility.name', units: null },
        'instrument_name': { name: 'instrument_name', ucd: 'meta.id;instr', utype: 'Provenance.ObsConfig.Instrument.name', units: null },
    }

    ObsCore.COLOR = '#004500'

    function ObsCore() {};

    ObsCore.parseFields = function(fields) {
        let parsedFields = {};

        const raField = ObsCore.MANDATORY_FIELDS['s_ra'];
        const decField = ObsCore.MANDATORY_FIELDS['s_dec'];
        const regionField = ObsCore.MANDATORY_FIELDS['s_region'];
        const accessUrlField = ObsCore.MANDATORY_FIELDS['access_url'];
        const accessFormat = ObsCore.MANDATORY_FIELDS['access_format'];

        let raFieldIdx = ObsCore.findMandatoryField(fields, raField.name, raField.ucd, raField.utype);
        let decFieldIdx = ObsCore.findMandatoryField(fields, decField.name, decField.ucd, decField.utype);
        let regionFieldIdx = ObsCore.findMandatoryField(fields, regionField.name, regionField.ucd, regionField.utype);
        let accessUrlFieldIdx = ObsCore.findMandatoryField(fields, accessUrlField.name, accessUrlField.ucd, accessUrlField.utype);
        let accessFormatFieldIdx = ObsCore.findMandatoryField(fields, accessFormat.name, accessFormat.ucd, accessFormat.utype);

        let fieldIdx = 0;
        fields.forEach((field) => {
            let key = field.name ? field.name : field.id;

            let nameField;
            if (fieldIdx == raFieldIdx) {
                nameField = 's_ra';
            } else if (fieldIdx == decFieldIdx) {
                nameField = 's_dec';
            } else if (fieldIdx == regionFieldIdx) {
                nameField = 's_region';
            } else if (fieldIdx == accessUrlFieldIdx) {
                nameField = 'access_url';
            } else if (fieldIdx == accessFormatFieldIdx) {
                nameField = 'access_format';
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


    // Find a specific field idx amond the VOTable fields
    //
    // @param fields: list of objects with ucd, unit, ID, name attributes
    // @param nameField:  index or name of the targeted column (might be undefined)
    // @param ucdField:  ucd of the targeted column (might be undefined)
    // @param possibleNames:  possible names of the targeted columns (might be undefined)
    //
    ObsCore.findMandatoryField = function(fields, nameField = null, ucdField = null, utypeField = null) {
        if (Utils.isInt(nameField) && nameField < fields.length) {
            // nameField can be given as an index
            return nameField;
        }

        // First, look if the name has been already given
        // ID or name of the field given
        if (nameField) { 
            for (var l=0, len=fields.length; l<len; l++) {
                var field = fields[l];
                
                if ( (field.ID && field.ID===nameField) || (field.name && field.name===nameField)) {
                    return l;
                }
            }
        }

        // If not already given, let's guess position column on the basis of UCDs
        if (ucdField) {
            var ucdFieldOld = ucdField.replace('.', '_');

            for (var l = 0, len = fields.length; l < len; l++) {
                var field = fields[l];

                if (field.ucd) {
                    var ucd = field.ucd.toLowerCase().trim();

                    if (ucd.indexOf(ucdField) == 0 || ucd.indexOf(ucdFieldOld) == 0) {
                        return l;
                    }
                }
            }
        }

        // Still not found ? guess the position from the utype
        if (utypeField) {
            for (var l = 0, len = fields.length; l < len; l++) {
                var field = fields[l];

                if (field.utype) {
                    var utype = field.utype.toLowerCase().trim();

                    if (utype === utypeField) {
                        return l;
                    }
                }
            }
        }

        throw 'Mandatory field ' + nameField + ' not found';
    };

    ObsCore.SHOW_CALLBACKS = function(aladinInstance) {
        return {
            "access_url": (data) => {
                let url = data['access_url'];
                let format = data['access_format'];

                let accessUrlEl = document.createElement('div');
                try {
                    let _ = new URL(url);
                    accessUrlEl.classList.add('aladin-href-link');

                    accessUrlEl.innerHTML = '<a href=' + url + ' target="_blank">' + url + '</a>';

                    accessUrlEl.addEventListener('click', (e) => {
                        e.preventDefault();
                        let processImageFitsClick = () => {
                            var name = data['obs_id'] || url;
                            var successCallback = ((ra, dec, fov, _) => {
                                aladinInstance.gotoRaDec(ra, dec);
                                aladinInstance.setFoV(fov);
                            });
            
                            let image = aladinInstance.createImageFITS(url, {name}, successCallback);

                            aladinInstance.setOverlayImageLayer(image, Utils.uuidv4())
                        };
        
                        switch (format) {
                            // A datalink response containing links to datasets or services attached to the current dataset
                            case 'application/x-votable+xml;content=datalink':
                                new Datalink().handleActions(url, data, aladinInstance);
                            break;
                            // Any multidimensional regularly sampled FITS image or cube
                            case 'image/fits':
                                processImageFitsClick();
                                break;
                            // Any generic FITS file
                            case 'application/fits':
                                processImageFitsClick();
                                break;
                            // A FITS multi-extension file (multiple extensions)
                            case 'application/x-fits-mef':
                                processImageFitsClick();
                                break;
                            default:
                                console.warn("Access format ", format, " not yet implemented or not recognized. Download the file triggered")
                                Utils.download(url)
                                break;
                        }
                    });
                } catch(e) {
                    accessUrlEl.innerText = '--';
                }

                return accessUrlEl;
            },
            'access_format': (data) => {
                let accessFormat = data['access_format'];

                if (accessFormat && accessFormat.includes('datalink')) {
                    return new ActionButton({
                        size: 'small',
                        content: '🔗',
                        tooltip: {content: 'Datalink VOTable', aladin: aladinInstance, global: true},
                        action(e) {}
                    }).element();
                } else {
                    return accessFormat;
                }
            }
        }
    };
 
    return ObsCore;
})();
