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



CooConversion = (function() {

    var CooConversion = {};
    
    CooConversion.GALACTIC_TO_J2000 = [
       -0.0548755604024359,  0.4941094279435681, -0.8676661489811610,
       -0.8734370902479237, -0.4448296299195045, -0.1980763734646737,
       -0.4838350155267381,  0.7469822444763707,  0.4559837762325372 ];
    
    CooConversion.J2000_TO_GALACTIC = [
        -0.0548755604024359, -0.873437090247923, -0.4838350155267381,
         0.4941094279435681, -0.4448296299195045, 0.7469822444763707,
        -0.8676661489811610, -0.1980763734646737, 0.4559837762325372 ];
    
    // adapted from www.robertmartinayers.org/tools/coordinates.html
    // radec : array of ra, dec in degrees
    // return coo in degrees
    CooConversion.Transform = function( radec, matrix ) {// returns a radec array of two elements
        radec[0] = radec[0]*Math.PI/180;
        radec[1] = radec[1]*Math.PI/180;
      var r0 = new Array ( 
       Math.cos(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[1]) );
        
     var s0 = new Array (
       r0[0]*matrix[0] + r0[1]*matrix[1] + r0[2]*matrix[2], 
       r0[0]*matrix[3] + r0[1]*matrix[4] + r0[2]*matrix[5], 
       r0[0]*matrix[6] + r0[1]*matrix[7] + r0[2]*matrix[8] ); 
     
      var r = Math.sqrt ( s0[0]*s0[0] + s0[1]*s0[1] + s0[2]*s0[2] ); 
    
      var result = new Array ( 0.0, 0.0 );
      result[1] = Math.asin ( s0[2]/r ); // New dec in range -90.0 -- +90.0 
      // or use sin^2 + cos^2 = 1.0  
      var cosaa = ( (s0[0]/r) / Math.cos(result[1] ) );
      var sinaa = ( (s0[1]/r) / Math.cos(result[1] ) );
      result[0] = Math.atan2 (sinaa,cosaa);
      if ( result[0] < 0.0 ) result[0] = result[0] + 2*Math.PI;
    
        result[0] = result[0]*180/Math.PI;
        result[1] = result[1]*180/Math.PI;
      return result;
    };
    
    // coo : array of lon, lat in degrees
    CooConversion.GalacticToJ2000 = function(coo) {
        return CooConversion.Transform(coo, CooConversion.GALACTIC_TO_J2000);
    };
    // coo : array of lon, lat in degrees
    CooConversion.J2000ToGalactic = function(coo) {
        return CooConversion.Transform(coo, CooConversion.J2000_TO_GALACTIC);
    };
    return CooConversion;
})();
