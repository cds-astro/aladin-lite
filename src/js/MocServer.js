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
        'http://alaskybis.unistra.fr/MocServer/query',
        'http://alasky.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTP
    static MIRRORS_HTTPS = [
        'https://alaskybis.unistra.fr/MocServer/query',
        'https://alasky.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTPS

    static _allHiPSes = undefined;
    static _allCatalogHiPSes = undefined;

    static getAllHiPSes() {
        if (!this._allHiPSes) {
            const params = {
                expr: "dataproduct_type=image||dataproduct_type=spectral-cube",
                //expr: "dataproduct_type=image",
                get: "record",
                fmt: "json",
                fields: "ID,hips_creator,hips_copyright,hips_order,hips_tile_width,hips_frame,hips_tile_format,obs_title,obs_description,obs_copyright,obs_regime,client_category,dataproduct_subtype,hips_service_url,hips_initial_ra,hips_initial_dec,hips_initial_fov,em_min,em_max",
            };
    
            this._allHiPSes = Utils.loadFromUrls(MocServer.MIRRORS_HTTPS, {
                data: params,
                dataType: 'json',
                desc: 'MOCServer query to get all the HiPS metadata'
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

        return Utils.loadFromUrls(MocServer.MIRRORS_HTTPS, {
            data: params,
            dataType: 'json',
            desc: 'MOCServer: Retrieve HiPS inside FoV'
        })
    }

    static getHiPSesFromIDs(ids) {
        const params = {
            //expr: "dataproduct_type=image||dataproduct_type=cube",
            expr: "dataproduct_type=image&&ID=" + ids.join(','),
            get: "record",
            fmt: "json",
            fields: "ID,hips_creator,hips_copyright,hips_frame,hips_tile_format,obs_title,obs_description,obs_copyright,obs_regime,dataproduct_subtype,hips_service_url,hips_initial_ra,hips_initial_dec,hips_initial_fov,em_min,em_max",
        };

        return Utils.loadFromUrls(MocServer.MIRRORS_HTTPS, {
            data: params,
            dataType: 'json'
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

            this._allCatalogHiPSes = Utils.loadFromUrls(MocServer.MIRRORS_HTTPS, {data: params, dataType: 'json'})
            this._allCatalogHiPSes.then((aa) => console.log(aa))
        }

        return this._allCatalogHiPSes;
    }


}
