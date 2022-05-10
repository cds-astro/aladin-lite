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
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export const HiPSSelector = (function () {

    // Constructor
    const HiPSSelector = function (parentDiv) {
        this.mainDiv = document.createElement('div');
        this.mainDiv.classList.add('modal', 'modal-open');
        //this.mainDiv.style.display = 'none';

        this.mainDiv.insertAdjacentHTML('afterbegin', 
        '<div class="modal-box">' +
          '<label class="btn btn-sm btn-circle absolute right-2 top-2">✕</label>' +
          '<h3 class="font-bold text-lg">Choose HiPS:</h3>' +
          '<br>' +
          '<div class="dropdown">' +
            '<div tabindex="0" class="tabs">' +
              '<a class="tab tab-lifted">By URL</a>' +
              '<a class="tab tab-lifted tab-active">By keyword, title, ID</a> ' +
            '</div>' +
            '  <div id="tab-contents">' +
            '<div id="first" class="p-4">' +
            '  First tab' +
            '</div>' +
            '<div id="second" class="hidden p-4">' +
            '  Second tab' +
            '</div>' +
            '<div id="third" class="hidden p-4">' +
            '  Third tab' +
            '</div>' +
            '<div id="fourth" class="hidden p-4">' +
            ' Fourth tab' +
            '</div>' +
         ' </div>' +
          '</ul>' +
            '</div>' +
            '<div class="flex space-x-4">' +
            '<button class="btn btn-primary btn-sm">OK</button>' +
            '<button class="btn btn-outline btn-primary btn-small">Cancel</button>' +
            '</div>' +
            
        '</div>');

        parentDiv.appendChild(this.mainDiv);

    };

    HiPSSelector.prototype = {
        show: function() {
            this.mainDiv.style.display = 'block';
        },

        hide: function() {
            this.mainDiv.style.display = 'none';
        }
    }


    return HiPSSelector;
})();
