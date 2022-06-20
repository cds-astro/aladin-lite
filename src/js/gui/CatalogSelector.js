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

import { MocServer } from "../MocServer";
import { Utils } from "../Utils";
import  autocomplete from 'autocompleter';

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/CatalogSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export class CatalogSelector {

    constructor(parentDiv, aladin, fnURLSelected, fnIdSelected) {
        this.parentDiv = parentDiv;
        this.aladin = aladin;

        this.fnURLSelected = fnURLSelected;
        this.fnIdSelected  = fnIdSelected;

        this.#createComponent();
    }

    #createComponent() {
        const self = this;

        this.mainDiv = document.createElement('div');
        this.mainDiv.classList.add('modal', 'modal-open', 'place-items-center', 'h-screen');
        this.mainDiv.style.display = 'none';

        const autocompleteId = 'autocomplete-' + Utils.uuidv4();
        this.mainDiv.insertAdjacentHTML('afterbegin', 
        '<div class="modal-box bg-stone-100">' +
          '<label class="btn btn-sm btn-circle absolute right-2 top-2">✕</label>' +
          '<h3 class="font-bold text-lg">Select catalogue</h3>' +
          '<br>' +
            '<div tabindex="0" class="tabs">' +
              '<a class="tab tab-lifted tab-active">By ID, title, keyword</a> ' +
              '<a class="tab tab-lifted">By URL</a>' +
            '</div>' +
            '<div>' +
            '<div class="p-4">' +
            '<input id="' + autocompleteId + '" type="text" placeholder="Type keyword" class="input input-bordered w-full max-w-xs" />' +
            '<div class="grid grid-cols-12 gap-4 m-3">' +
              '<div class="col-start-1 col-end-7 space-y-2">' +
                '<div><button class="aladin-btn">Load cone</button></div>' +
                '<div><input type="number" value="1.0" class="w-12"><select><option>deg<option>arcmin<option>arcsec</select> around view center</div>' +
                '<div>Limit to <input type="number" min="1" max="10000" value="1000" class="w-20"> sources</div>' +
              '</div>' +
              '<div class="col-start-9 col-span-4">' +
                '<button class="aladin-btn">Load catalogue HiPS</button>' +
              '</div>' +
            '</div>' +
            '</div>' +
            '<div class="hidden p-4">' +
            '<input type="text" placeholder="Type VOTable URL" class="input input-bordered w-full max-w-xs" />' +
            '<div class="flex space-x-2 justify-center pt-6">' +
              '<button class="aladin-btn">Load VOTable</button>' +
            '</div>' +
            '</div>' +
         ' </div>' +
            
        '</div>');

        this.parentDiv.appendChild(this.mainDiv);

        // retrieve cone search div and load HiPS div
        this.divCS = this.mainDiv.querySelector('div > div:nth-child(5) > div:nth-child(1) > div > div.col-start-1');
        this.divLoadHiPS = this.mainDiv.querySelector('div > div:nth-child(5) > div:nth-child(1) > div > div.col-start-9');
        $(this.divCS).hide();
        $(this.divLoadHiPS).hide();

        // listener to load CS data
        const loadCSBtn = this.divCS.querySelector('div:nth-child(1) > button');
        $(loadCSBtn).click(function() {
            const radius = parseFloat(self.divCS.querySelector('div:nth-child(2) > input').value);
            const radiusUnit = self.divCS.querySelector('div:nth-child(2) > select').value;
            let radiusDeg = radius;
            if (radiusUnit=='arcmin') {
                radiusDeg /= 60.0;
            }
            else if (radiusUnit=='arcsec') {
                radiusDeg /= 3600.0;
            }
            const maxNbSources = parseInt(self.divCS.querySelector('div:nth-child(3) > input').value);
            const baseURL = self.selectedItem.cs_service_url;


            const [ra, dec] = self.aladin.getRaDec();
            self.fnIdSelected && self.fnIdSelected('coneSearch', {id: self.idInput.value, baseURL: baseURL, limit: maxNbSources, radiusDeg: radiusDeg, ra: ra, dec: dec});
        });

        // listener to load HiPS catalogue
        const loadHiPSBtn = this.divLoadHiPS.querySelector('button');
        $(loadHiPSBtn).click(function() {
            self.fnIdSelected && self.fnIdSelected('hips', {id: self.idInput.value, hipsURL: self.selectedItem.hips_service_url});
        });

        // listener to load catalogue from VOTable URL
        const loadVOTableBtn = document.querySelector('div > div:nth-child(5) > div:nth-child(2) > div > button');
        $(loadVOTableBtn).click(function() {
            self.fnURLSelected && self.fnURLSelected(self.urlInput.value);
        });



        // setup autocomplete
        let input = document.getElementById(autocompleteId);
        
        // Query the mocserver
        MocServer.getAllCatalogHiPSes();
        autocomplete({
            input: input,
            minLength: 3,
            fetch: function(text, update) {
                text = text.toLowerCase();
                const filterCats = function(item) {
                    const ID = item.ID;
                    const obsTitle = item.obs_title || '';
                    const obsDescription = item.obs_description || '';

                    return ID.toLowerCase().includes(text) || obsTitle.toLowerCase().includes(text) || obsDescription.toLowerCase().includes(text);
                }
                // you can also use AJAX requests instead of preloaded data
                const suggestions = MocServer.getAllCatalogHiPSes().filter(filterCats);
                update(suggestions);
            },
            onSelect: function(item) {
                // adapt UI to selected catalogue
                if (item.cs_service_url) {
                    $(self.divCS).show();
                }
                else {
                    $(self.divCS).hide();
                }
                if (item.hips_service_url) {
                    $(self.divLoadHiPS).show();
                }
                else {
                    $(self.divLoadHiPS).hide();
                }

                input.value = item.ID;
                self.selectedItem = item;
            },
            render: function(item, currentValue) {
                const itemElement = document.createElement("div");
                itemElement.innerHTML = (item.obs_title || '') + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';


                return itemElement;
            }
        });

        // tab management
        this.firstTab = this.mainDiv.querySelectorAll('div div a')[0];
        this.secondTab = this.mainDiv.querySelectorAll('div div a')[1];
        let firstTabContent = this.mainDiv.querySelectorAll('div div .p-4')[0];
        let secondTabContent = this.mainDiv.querySelectorAll('div div .p-4')[1];

        this.idInput = self.mainDiv.querySelectorAll('div div .p-4')[0].querySelector('input');
        this.urlInput = self.mainDiv.querySelectorAll('div div .p-4')[1].querySelector('input');

        $(this.firstTab).click(function() {
            $(self.secondTab).removeClass('tab-active');
            $(self.firstTab).addClass('tab-active');
            $(secondTabContent).hide();
            $(firstTabContent).show();

            self.idInput.focus();
        });
        $(this.secondTab).click(function() {
            $(self.firstTab).removeClass('tab-active');
            $(self.secondTab).addClass('tab-active');
            $(firstTabContent).hide();
            $(secondTabContent).show();

            self.urlInput.focus();
        });

        // this modal is closed when clicking on the cross at the top right, or on the Cancel button
        let circledCloseBtn  = this.mainDiv.querySelectorAll('.btn-circle');
        let closeBtn = this.mainDiv.querySelectorAll('.btn-outline');
        $(closeBtn).add($(circledCloseBtn)).click(function() {
            self.hide();
        });

        // when 'Select' is pressed, call the callbacks
        let selectBtn = this.mainDiv.querySelectorAll('.btn-primary');
        $(selectBtn).click(function() {
            let byIdSelected = $(self.mainDiv.querySelectorAll('div div a')[0]).hasClass('tab-active');

            let idInput = self.mainDiv.querySelectorAll('div div .p-4')[0].querySelector('input');
            let urlInput = self.mainDiv.querySelectorAll('div div .p-4')[1].querySelector('input');

            if (byIdSelected) {
                self.fnIdSelected && self.fnIdSelected(idInput.value, self.selectedItem, {});
            }
            else {
                self.fnURLSelected && self.fnURLSelected(urlInput.value);
            }

            idInput.value = '';
            urlInput.value = '';
        
            self.hide();

        });

    }

    show() {
        this.mainDiv.style.display = 'flex';
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }

}
