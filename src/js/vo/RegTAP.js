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

/******************************************************************************
 * Aladin Lite project
 * 
 * File RegTAP
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/
 import { Utils } from '../Utils';

 export let RegTAP = (function() {
    RegTAP.HiPSList = null;

    function RegTAP() {};

    RegTAP._query = async function(adql) {
        const params = new URLSearchParams({
            REQUEST: "doQuery",
            LANG: "ADQL",
            FORMAT: "json",
            QUERY: adql
        });

        const url = `https://dc.g-vo.org/tap/sync?${params}`;

        return await Utils.fetch({
            url,
            method: "GET",
            dataType: "json",
            timeout: 20000,
            desc: "Harvest the registry for HiPS data."
        });
    }

    // Provides a dictionary of all the HiPS fetched from the different
    // HiPSList in the registry.
    // Key is the creator_did unique HiPS identifier and value is a list of mirrors for that HiPS 
    RegTAP.harvestHiPSLists = async function() {
        if (RegTAP.HiPSList) {
            return RegTAP.HiPSList;
        }
    
        console.info('Harvest the registry for the collection of available HiPSes');

        const result = await RegTAP._query(
            "select access_url, standard_id, ivoid from rr.resource \
    natural left outer join rr.capability \
    natural left outer join rr.interface \
    where standard_id like 'ivo://ivoa.net/std/hips%' and ivoid not like '%cds%'"
        );

        RegTAP.HiPSList = {};
        let hipsListFetchingPromises = result.data.filter((record) => {
            let standard_id = record[1];
            if (standard_id.includes("hipslist")) {
                // Keep the HiPSList records
                return true;
            } else {
                // Discard the HiPS records by directly storing them in the HiPSList
                let ivoid = record[2];
                let access_url = record[0];
                let creator_did = ivoid.replace(/^ivo:\/\//, "");

                const hips = {creator_did, hips_service_url: access_url};

                if (creator_did in RegTAP.HiPSList) {
                    RegTAP.HiPSList[creator_did].push(hips);
                } else {
                    RegTAP.HiPSList[creator_did] = [hips];
                }

                // Discard the hips record
                return false;
            }
        })
        .map((record) => {
            let url = record[0];
            return Utils.fetch({
                url,
                dataType: 'text',
                timeout: 1000,
            });
        })

        const fileContents = await Promise.allSettled(hipsListFetchingPromises)
    
        for (const fileContent of fileContents) {
            if (fileContent.status === "fulfilled") {
                const HiPSList = parseHiPSList(fileContent.value);

                for (const [ID, hips] of Object.entries(HiPSList)) {
                    if (ID in RegTAP.HiPSList) {
                        RegTAP.HiPSList[ID].push(hips);
                    } else {
                        RegTAP.HiPSList[ID] = [hips];
                    }
                }
            }
        }

        return RegTAP.HiPSList;
    }
 
    return RegTAP;
})();

function parseHiPSList(text) {
    const result = {};

    const entries = text.trim().split(/\r?\n\s*\r?\n/);

    for (const entry of entries) {
        const hips = {};

        for (const line of entry.split(/\r?\n/)) {
            const separator = line.indexOf("=");

            if (separator === -1 || line.trim().startsWith("#")) {
                continue;
            }

            const key = line.substring(0, separator).trim();
            const value = line.substring(separator + 1).trim();

            if (key) {
                hips[key] = value;
            }
        }

        const did = hips.creator_did || hips.publisher_did;

        if (did && hips.hips_service_url) {
            const key = did.replace(/^ivo:\/\//, "");
            hips["creator_did"] = key;
            result[key] = hips;
        }
    }

    return result;
}
