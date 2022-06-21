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

    constructor(parentDiv, aladin, fnIdSelected) {
        this.parentDiv = parentDiv;
        this.aladin = aladin;

        this.fnIdSelected  = fnIdSelected;

        this._createComponent();
    }

    _createComponent() {
        const self = this;

        this.mainDiv = document.createElement('div');
        this.mainDiv.classList.add('aladin-dialog', 'aladin-cb-list');
        this.mainDiv.style.display = 'block';

        const autocompleteId = 'autocomplete-' + Utils.uuidv4();
        this.mainDiv.insertAdjacentHTML('afterbegin',
        '<a class="aladin-closeBtn">&times;</a>' +
        '<div class="aladin-box-title">Select Catalogue:</div>' +
        '<div class="aladin-box-content">' +
            '<div class="aladin-label" for="' + autocompleteId + '">By ID, title, keyword</div>' +
            '<input style="width:100%;" name="' + autocompleteId + '" id="' + autocompleteId + '" type="text" placeholder="Type keyword or VOTable URL" />' +
            '<div class="aladin-row" style="font-size: 12px;">' +
                '<div class="cone-search aladin-col">' +
                    '<div><input type="number" value="1.0" style="width: 4em;" maxlength="5" size="5"> <select style="padding: 4px 0!important;"><option>deg<option>arcmin<option>arcsec</select> around view center</div>' +
                    '<div>Limit to <input type="number" min="1" max="10000" value="1000" style="width: 5em;"> sources</div>' +
                '</div>' +
            '</div>' +
            '<div class="aladin-row">' +
                '<div class="cone-search aladin-col">' +
                    '<div><button class="aladin-btn">Load cone</button></div>' +
                '</div>' +
                '<div class="hips aladin-col"><button class="aladin-btn">Load catalogue HiPS</button></div>' +
            '</div>' +
            '<div class="aladin-box-separator"></div>' +
            '<div class="aladin-label" for="' + autocompleteId + '">By VOTable URL</div>' +
            '<input style="width:100%;" name="' + autocompleteId + '" id="' + autocompleteId + '" type="text" placeholder="Enter VOTable URL" />' +
            '<div class="votable"><button class="aladin-btn">Load VOTable</button></div>' +
        '</div>'
        );

        this.parentDiv.appendChild(this.mainDiv);

        this.idInput = self.mainDiv.querySelectorAll('input')[0];
        this.votInput = self.mainDiv.querySelectorAll('input')[3];

        let [loadCSBtn, loadHiPSBtn, loadVOTableBtn]  = this.mainDiv.querySelectorAll('.aladin-btn');
        this.divCS = this.mainDiv.querySelector('.cone-search');
        this.divLoadHiPS = this.mainDiv.querySelector('.hips');
        this.divLoadHiPS.style.display = "none";


        // retrieve cone search div and load HiPS div
        //this.divCS = this.mainDiv.querySelector('div > div:nth-child(5) > div:nth-child(1) > div > div.col-start-1');
        //this.divLoadHiPS = this.mainDiv.querySelector('div > div:nth-child(5) > div:nth-child(1) > div > div.col-start-9');
        //$(this.divCS).hide();
        //$(this.divLoadHiPS).hide();

        // listener to load CS data
        //const loadCSBtn = this.divCS.querySelector('div:nth-child(1) > button');
        $(loadCSBtn).click(function() {
            const radius = parseFloat(self.divCS.querySelector('div:nth-child(1) > input').value);
            const radiusUnit = self.divCS.querySelector('div:nth-child(1) > select').value;
            let radiusDeg = radius;
            if (radiusUnit=='arcmin') {
                radiusDeg /= 60.0;
            }
            else if (radiusUnit=='arcsec') {
                radiusDeg /= 3600.0;
            }
            const maxNbSources = parseInt(self.divCS.querySelector('div:nth-child(2) > input').value);
            const baseURL = self.selectedItem.cs_service_url;

            const [ra, dec] = self.aladin.getRaDec();

            self.fnIdSelected && self.fnIdSelected('coneSearch', {id: self.idInput.value, baseURL: baseURL, limit: maxNbSources, radiusDeg: radiusDeg, ra: ra, dec: dec});
        });

        // listener to load HiPS catalogue
        //const loadHiPSBtn = this.divLoadHiPS.querySelector('button');
        $(loadHiPSBtn).click(function() {
            self.fnIdSelected && self.fnIdSelected('hips', {id: self.idInput.value, hipsURL: self.selectedItem.hips_service_url});
        });

        // listener to load catalogue from VOTable URL
        //const loadVOTableBtn = document.querySelector('div > div:nth-child(5) > div:nth-child(2) > div > button');
        $(loadVOTableBtn).click(function() {
            self.fnIdSelected && self.fnIdSelected('votable', {url: self.votInput.value});
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

        // this modal is closed when clicking on the cross at the top right, or on the Cancel button
        let [closeBtn]  = this.mainDiv.querySelectorAll('.aladin-closeBtn');

        $(closeBtn).click(function() {
            self.hide();
        });

    }

    show() {
        this.mainDiv.style.display = 'block';
        /*
        // focus on text field
        let byIdSelected = $(this.mainDiv.querySelectorAll('div div a')[0]).hasClass('tab-active');
        if (byIdSelected) {
            let idInput = this.mainDiv.querySelectorAll('div div .p-4')[0].querySelector('input');
            idInput.focus();
        }
        else {
            let urlInput = this.mainDiv.querySelectorAll('div div .p-4')[1].querySelector('input');
            urlInput.focus();
        }*/
    }

    hide() {
        this.mainDiv.style.display = 'none';
    }

}
