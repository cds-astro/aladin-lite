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
 * File gui/AladinLogo.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export const AladinLogo = (function () {

    // Constructor
    const AladinLogo = function (parentDiv) {
        const newDiv = document.createElement('div');
        newDiv.classList.add('aladin-logo-container');

        const link = document.createElement('a');
        link.href   ='https://aladin.cds.unistra.fr/' 
        link.title  = 'Powered by Aladin Lite';
        link.target = '_blank';

        const logoDiv = document.createElement('div');
        logoDiv.classList.add('aladin-logo');

        link.appendChild(logoDiv);
        newDiv.appendChild(link);

       parentDiv.appendChild(newDiv);
    };


    return AladinLogo;
})();
