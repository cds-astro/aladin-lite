// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//

import { Utils } from "./Utils";


/******************************************************************************
 * Aladin Lite project
 * 
 * File MocServer
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


export class MocServer {
    static MIRRORS_HTTP = [
        'http://alaskybis.cds.unistra.fr/MocServer/query',
        'http://alasky.cds.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTP
    static MIRRORS_HTTPS = [
        'https://alaskybis.cds.unistra.fr/MocServer/query',
        'https://alasky.cds.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTPS

    static _allHiPSes = undefined;
    static _allCatalogHiPSes = undefined;
    static _notAvailable = false;

    static fetchMetadataFromID(ID) {
        const params = {
            get: "record",
            fmt: "json",
            ID: "*" + decodeURI(ID) + "*",
        };

        if (MocServer._notAvailable) {
            console.warn('MocServer mirrors not availables');
        }

        return Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {
            data: params,
            dataType: 'json'
        }).catch((_) => {
            MocServer._notAvailable = true;
        })
    }

    static getAllHiPSes() {
        if (MocServer._notAvailable) {
            console.warn('MocServer mirrors not available');
        }

        if (!this._allHiPSes) {
            const params = {
                expr: "dataproduct_type=image||dataproduct_type=spectral-cube",
                //expr: "dataproduct_type=image",
                get: "record",
                fmt: "json",
                fields: "ID,hips_creator,hips_copyright,hips_order,hips_tile_width,hips_frame,hips_tile_format,obs_title,obs_description,obs_copyright,obs_regime,client_category,dataproduct_subtype,hips_service_url,hips_initial_ra,hips_initial_dec,hips_initial_fov,em_min,em_max",
            };
    
            this._allHiPSes = Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {
                data: params,
                dataType: 'json',
                desc: 'MOCServer query to get all the HiPS metadata'
            }).catch((_) => {
                MocServer._notAvailable = true;
            })
        }

        return this._allHiPSes;
    }

    static getAllHiPSesInsideView(aladin) {
        let params = {
            expr: "dataproduct_type=image||dataproduct_type=spectral-cube",
            //expr: "dataproduct_type=image",
            get: "record",
            fmt: "json",
            fields: "ID",
        };

        try {
            const corners = aladin.getFoVCorners(1, 'icrs');
            let stc = 'Polygon '
            for (var radec of corners) {
                stc += radec[0] + ' ' + radec[1] + ' ';
            }

            params['stc'] = stc;
        } catch (e) {}

        if (MocServer._notAvailable) {
            console.warn('MocServer mirrors not available');
        }

        return Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {
            data: params,
            dataType: 'json',
            desc: 'MOCServer: Retrieve HiPS inside FoV'
        }).catch((_) => {
            MocServer._notAvailable = true;
        })
    }

    static getAllCatalogHiPSes() {
        if (!this._allCatalogHiPSes) {
            const params = {
                expr: "dataproduct_type=catalog",
                get: "record",
                fmt: "json",
                fields: "ID,hips_copyright,obs_title,obs_description,obs_copyright,cs_service_url,hips_service_url",
            };

            if (MocServer._notAvailable) {
                console.warn('MocServer mirrors not available');
            }

            this._allCatalogHiPSes = Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {data: params, dataType: 'json'})
                .catch((_) => {
                    MocServer._notAvailable = true;
                })
        }

        return this._allCatalogHiPSes;
    }
}
