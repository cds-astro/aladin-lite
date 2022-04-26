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
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 import { Projection } from "./libs/astro/projection.js";

 export let ProjectionEnum = {
    SIN: Projection.PROJ_SIN,
    AITOFF:  Projection.PROJ_AITOFF,
    MERCATOR:  Projection.PROJ_MERCATOR,
    ARC:  Projection.PROJ_ARC,
    TAN:  Projection.PROJ_TAN,
    MOL: Projection.PROJ_MOLLWEIDE,
    HPX: Projection.PROJ_HEALPIX,
 };
