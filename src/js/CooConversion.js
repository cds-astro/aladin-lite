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



export let CooConversion = (function() {

    let CooConversion = {};
    
    CooConversion.GALACTIC_TO_ICRS = [
        -0.05487565771261968232908806948676,  0.49410943719710765017955928850141, -0.86766613755716255824577781583414,
        -0.87343705195577915249273984034980, -0.44482972122205372312012370920248, -0.19807633727507056817237662907031,
        -0.48383507361641838378786914298189,  0.74698218398450941835110635824212,  0.45598381369115237931077906137440 ];
    
    CooConversion.ICRS_TO_GALACTIC = [
        -0.05487565771261968232908806948676, -0.87343705195577915249273984034980, -0.48383507361641838378786914298189,
         0.49410943719710765017955928850141, -0.44482972122205372312012370920248,  0.74698218398450941835110635824212,
        -0.86766613755716255824577781583414, -0.19807633727507056817237662907031,  0.45598381369115237931077906137440 ];
    
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
    CooConversion.GalacticToICRS = function(coo) {
        return CooConversion.Transform(coo, CooConversion.GALACTIC_TO_ICRS);
    };
    // coo : array of lon, lat in degrees
    CooConversion.ICRSToGalactic = function(coo) {
        return CooConversion.Transform(coo, CooConversion.ICRS_TO_GALACTIC);
    };
    return CooConversion;
})();
