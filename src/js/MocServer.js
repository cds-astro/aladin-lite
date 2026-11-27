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
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

export class MocServer {
    static MIRRORS_HTTP = [
        'http://alasky.cds.unistra.fr/MocServer/query',
        'http://alaskybis.cds.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTP
    static MIRRORS_HTTPS = [
        'https://alasky.cds.unistra.fr/MocServer/query',
        'https://alaskybis.cds.unistra.fr/MocServer/query'
    ]; // list of base URL for MocServer mirrors, available in HTTPS

    static _allHiPSes = undefined;
    static _allCatalogHiPSes = undefined;
    static _notAvailable = false;

    static async fetchPropertiesFromID(ID) {
        // Check if we have already it in cache
        if (MocServer._allHiPSes) {
            let properties = MocServer._allHiPSes.filter((hips) => hips.ID.includes(ID));

            if (properties.length > 0) {

                console.log("properties found for ", ID, properties)
                // HiPS have been found in the cache
                return properties;
            }
        }

        if (MocServer._notAvailable) {
            throw new Error('MocServer mirrors not availables');
        }

        // Make the Query
        const params = {
            get: "record",
            fmt: "json",
            ID: "*" + decodeURI(ID) + "*",
        };

        try {
            return await Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {
                data: params,
                dataType: 'json'
            });
        } catch (_) {
            MocServer._notAvailable = true;

            // Propagate the error 
            throw new Error('MocServer mirrors not availables');
        }
    }

    static async getAllHiPSes() {
        if (MocServer._allHiPSes) {
            return MocServer._allHiPSes;
        }

        if (MocServer._notAvailable) {
            throw new Error('MocServer mirrors not available');
        }

        // Query the MocServer
        const params = {
            expr: "dataproduct_type=image||dataproduct_type=spectral-cube",
            //expr: "dataproduct_type=image",
            get: "record",
            fmt: "json",
            fields: "*",
        };

        try {
            const allHiPSes = await Utils.fetchFromMirrorUrls(
                MocServer.MIRRORS_HTTPS,
                {
                    data: params,
                    dataType: 'json',
                    desc: 'MOCServer query to get all the HiPS metadata'
                }
            );

            MocServer._allHiPSes = allHiPSes;
            return MocServer._allHiPSes;
        } catch (_) {
            MocServer._notAvailable = true;
            throw new Error('MocServer mirrors not available');
        }
    }

    static async getAllHiPSesInsideView(aladin) {
        if (MocServer._notAvailable) {
            console.warn('MocServer mirrors not available');
            return [];
        }

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

        try {
            return await Utils.fetchFromMirrorUrls(MocServer.MIRRORS_HTTPS, {
                data: params,
                dataType: 'json',
                desc: 'MOCServer: Retrieve HiPS inside FoV'
            })
        } catch(e) {
            MocServer._notAvailable = true;

            throw new Error('MocServer is not available');
        }
    }

    static async getAllCatalogHiPSes() {
        // Check if it has been cached
        if (MocServer._allCatalogHiPSes) {
            return MocServer._allCatalogHiPSes;
        }

        // Not cached, check the MocServer status
        if (MocServer._notAvailable) {
            throw new Error('MocServer mirrors not availables');
        }

        // Query the MocServer
        const params = {
            expr: "dataproduct_type=catalog",
            get: "record",
            fmt: "json",
            fields: "ID,hips_copyright,obs_title,obs_description,obs_copyright,cs_service_url,hips_service_url*",
        };

        try {
            const allCatalogHiPSes = await Utils.fetchFromMirrorUrls(
                MocServer.MIRRORS_HTTPS,
                {
                    data: params,
                    dataType: 'json',
                    desc: 'MOCServer query to get all the HiPS metadata'
                }
            );

            MocServer._allCatalogHiPSes = allCatalogHiPSes;
            return MocServer._allCatalogHiPSes;

        } catch (_) {
            MocServer._notAvailable = true;
            throw new Error('MocServer mirrors not availables');
        }
    }
}
