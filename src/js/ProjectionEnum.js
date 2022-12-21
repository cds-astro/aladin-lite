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
export let ProjectionEnum = {
   // Zenithal
   TAN: {id: 1, fov: 180},	  /* Gnomonic projection      */
   STG: {id: 2, fov: 360},	  /* Stereographic projection */
   SIN: {id: 3, fov: 180},	  /* Orthographic		         */
   ZEA: {id: 4, fov: 360},	  /* Equal-area 		         */
   FEYE: {id: 5, fov: 190},
   AIR: {id: 6, fov: 360},
   //AZP: {fov: 180},
   ARC: {id: 7, fov: 360},
   NCP: {id: 8, fov: 180},
   // Cylindrical
   MER: {id: 9, fov: 360},
   CAR: {id: 10, fov: 360},
   CEA: {id: 11, fov: 360},
   CYP: {id: 12, fov: 360},
   // Pseudo-cylindrical
   AIT: {id: 13, fov: 360},
   PAR: {id: 14, fov: 360},
   SFL: {id: 15, fov: 360},
   MOL: {id: 16, fov: 360},
   // Conic
   COD: {id: 17, fov: 360},
   // Hybrid
   HPX: {id: 19, fov: 360},
};

export let projectionNames = [
   // Zenithal
   "TAN",	  /* Gnomonic projection      */
   "STG",	  /* Stereographic projection */
   "SIN",	  /* Orthographic		         */
   "ZEA",	  /* Equal-area 		         */
   "FEYE",
   "AIR",
   //"AZP",
   "ARC",
   "NCP",
   // Cylindrical
   "MER",
   "CAR",
   "CEA",
   "CYP",
   // Pseudo-cylindrical
   "AIT",
   "PAR",
   "SFL",
   "MOL",
   // Conic
   "COD",
   // Hybrid
   "HPX"
]
