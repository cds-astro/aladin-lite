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
   TAN: {id: 1, fov: 150, label: "Tangential"},	         /* Gnomonic projection      */
   STG: {id: 2, fov: 240, label: "Stereographic"},	      /* Stereographic projection */
   SIN: {id: 3, fov: 180, label: "Spheric"},	      /* Orthographic		         */
   // TODO: fix why the projection disappears at fov = 360.0
   ZEA: {id: 4, fov: 359.999, label: "Zenital equal-area"},	/* Equal-area 		         */
   //FEYE: {id: 5, fov: 190, label: "fish eye"},
   //AIR: {id: 6, fov: 360, label: "airy"},
   //AZP: {fov: 180},
   //ARC: {id: 7, fov: 360, label: "zenital equidistant"},
   //NCP: {id: 8, fov: 180, label: "north celestial pole"},
   // Cylindrical
   MER: {id: 9, fov: 360, label: "Mercator"},
   //CAR: {id: 10, fov: 360, label: "plate carrée"},
   //CEA: {id: 11, fov: 360, label: "cylindrical equal area"},
   //CYP: {id: 12, fov: 360, label: "cylindrical perspective"},
   // Pseudo-cylindrical
   AIT: {id: 13, fov: 360, label: "Hammer-Aïtoff"},
   //PAR: {id: 14, fov: 360, label: "parabolic"},
   //SFL: {id: 15, fov: 360, label: "sanson-flamsteed"},
   MOL: {id: 16, fov: 360, label: "Mollweide"},
   // Conic
   //COD: {id: 17, fov: 360, label: "conic equidistant"},
   // Hybrid
   //HPX: {id: 19, fov: 360, label: "healpix"},
};

/*
export let projectionNames = [
   // Zenithal
   "SIN",	  // Orthographic		         
   "TAN",	  // Gnomonic projection      
   "STG",	  // Stereographic projection 
   "ZEA",	  // Equal-area 		         
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
   "MOL",
   "PAR",
   "SFL",
   // Conic
   "COD",
   // Hybrid
   "HPX"
]
*/
