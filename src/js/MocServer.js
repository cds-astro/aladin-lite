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
 * File MocServer
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

export class MocServer {
    static _allHiPSes = undefined;
    static _allCatalogHiPSes = undefined;

    static getAllHiPSes() {
        if (this._allHiPSes === undefined) {
            (async () => {
                this._allHiPSes = await fetch('https://alasky.cds.unistra.fr/MocServer/query?expr=dataproduct_type%3Dimage+%7C%7C%A0dataproduct_type%3Dcube&get=record&fmt=json&fields=ID,hips_initial_fov,hips_initial_ra,hips_initial_dec,hips_pixel_bitpix,hips_creator,hips_copyright,hips_frame,hips_order,hips_order_min,hips_tile_width,hips_tile_format,hips_pixel_cut,obs_title,obs_description,obs_copyright,obs_regime,hips_data_range,hips_service_url')
                                         .then(response => {return response.json();});
            })();
        }

        return this._allHiPSes;
    }

    static getAllCatalogHiPSes() {
        if (this._allCatalogHiPSes === undefined) {
            (async () => {
                this._allCatalogHiPSes = await fetch('https://alasky.cds.unistra.fr/MocServer/query?expr=dataproduct_type%3Dcatalog&get=record&fmt=json&fields=ID,hips_copyright,hips_order,hips_order_min,obs_title,obs_description,obs_copyright,obs_regime,cs_service_url,hips_service_url')
                                         .then(response => {return response.json();});
            })();
        }

        return this._allCatalogHiPSes;
    }


}
