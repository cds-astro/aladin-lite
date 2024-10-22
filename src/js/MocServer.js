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
                expr: "dataproduct_type=image||dataproduct_type=cube",
                //expr: "dataproduct_type=image",
                get: "record",
                fmt: "json",
                fields: "ID,hips_creator,hips_copyright,hips_order,hips_tile_width,hips_frame,hips_tile_format,obs_title,obs_description,obs_copyright,obs_regime",
                //fields: "ID,hips_initial_fov,hips_initial_ra,hips_initial_dec,hips_pixel_bitpix,hips_creator,hips_copyright,hips_frame,hips_order,hips_order_min,hips_tile_width,hips_tile_format,hips_pixel_cut,obs_title,obs_description,obs_copyright,obs_regime,hips_data_range,hips_service_url",
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
            //expr: "dataproduct_type=image||dataproduct_type=cube",
            expr: "dataproduct_type=image",
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
            fields: "ID,hips_creator,hips_copyright,hips_frame,hips_tile_format,obs_title,obs_description,obs_copyright,obs_regime",
            //fields: "ID,hips_initial_fov,hips_initial_ra,hips_initial_dec,hips_pixel_bitpix,hips_creator,hips_copyright,hips_frame,hips_order,hips_order_min,hips_tile_width,hips_tile_format,hips_pixel_cut,obs_title,obs_description,obs_copyright,obs_regime,hips_data_range,hips_service_url",
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
                //fields: "ID,hips_copyright,hips_order,hips_order_min,obs_title,obs_description,obs_copyright,obs_regime,cs_service_url,hips_service_url",
            };

            this._allCatalogHiPSes = Utils.loadFromUrls(MocServer.MIRRORS_HTTPS, {data: params, dataType: 'json'})
        }

        return this._allCatalogHiPSes;
    }


}
