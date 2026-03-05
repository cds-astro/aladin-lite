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
 * File GenericPointer.js
 * 
 ******************************************************************************/

import { SimbadPointer } from "./SimbadPointer.js";
import { PlanetaryFeaturesPointer } from "./PlanetaryFeaturesPointer.js";
import { Utils } from './Utils';

// allow to call either Simbad or Planetary features Pointers
export let GenericPointer = function (view, e) {
    let xymouse;
    if (e instanceof Event) {
        xymouse = Utils.relMouseCoords(e);
    } else {
        xymouse = e;
    }

    let radec = view.aladin.pix2world(xymouse.x, xymouse.y, 'icrs');
    if (radec) {
        // sky case
        if (view.aladin.getBaseImageLayer().isPlanetaryBody() === false) {
            const queryRadius = Math.min(1, 15 * view.fov / view.largestDim);
            SimbadPointer.query(radec[0], radec[1], queryRadius, view.aladin);
        }
        // planetary body case
        else {
            // TODO: replace with actual value
            const body = view.aladin.getBaseImageLayer().hipsBody;
            PlanetaryFeaturesPointer.query(radec[0], radec[1], Math.min(80, view.fov / 20.0), body, view.aladin);
        }
    } else {
        alert("The location you clicked on is out of the view.");
    }
}

