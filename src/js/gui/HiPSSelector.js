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

import $ from 'jquery';

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export class HiPSSelector {

    constructor(parentDiv, fnIdSelected, aladin) {
        this.parentDiv = parentDiv;

        this.fnIdSelected  = fnIdSelected;

        this.aladin = aladin;

        this._createComponent();
    }

    _createComponent() {
        const self = this;

        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'block';
        this.mainDiv.classList.add('aladin-dialog', 'aladin-layerBox', 'aladin-cb-list');

        const autocompleteId = 'autocomplete-' + Utils.uuidv4();
        this.mainDiv.insertAdjacentHTML('afterbegin', 
          '<a class="aladin-closeBtn">&times;</a>' +
          '<div class="aladin-box-title">Select image HiPS:</div>' +
          '<div class="aladin-box-content">' +

                '<div class="aladin-label" for="' + autocompleteId + '">By ID, title, keyword or URL</div>' +
                '<input name="' + autocompleteId + '" id="' + autocompleteId + '" type="text" placeholder="Type ID, title, keyword or URL" /><br>' +

            '<div>' +
                '<button class="aladin-btn">Select HiPS</button>' +
                '<button class="aladin-btn">Load coverage</button>' +
            '</div>' +
          '</div>'
        );

        this.parentDiv.appendChild(this.mainDiv);

        // setup autocomplete
        const input = document.getElementById(autocompleteId);

        // Unfocus the keyboard on android devices (maybe it concerns all smartphones) when the user click on enter
        $(input).on("change", function () {
            input.blur();
        });

        // Query the mocserver
        MocServer.getAllHiPSes();

        autocomplete({
            input: input,
            fetch: function(text, update) {
                text = text.toLowerCase();
                // filter suggestions
                const suggestions = MocServer.getAllHiPSes().filter(n => n.ID.toLowerCase().includes(text) || n.obs_title.toLowerCase().includes(text))

                // sort suggestions
                suggestions.sort( function(a , b) {
                    let scoreForA = 0;
                    let scoreForB = 0;

                    if (a.ID.toLowerCase().includes(text)) {
                        scoreForA += 100;
                    }
                    if (b.ID.toLowerCase().includes(text)) {
                        scoreForB += 100;
                    }

                    if (a.obs_title.toLowerCase().includes(text)) {
                        scoreForA += 50;
                    }
                    if (b.obs_title.toLowerCase().includes(text)) {
                        scoreForB += 50;
                    }

                    if (a.obs_description && a.obs_description.toLowerCase().includes(text)) {
                        scoreForA += 10;
                    }
                    if (b.obs_description && b.obs_description.toLowerCase().includes(text)) {
                        scoreForB += 10;
                    }

                    if (scoreForA > scoreForB) {
                        return -1;
                    }
                    if (scoreForB > scoreForA) {
                        return  1;
                    }

                    return 0;
                });

                // limit to 50 first suggestions
                const returnedSuggestions = suggestions.slice(0, 50);

                update(returnedSuggestions);
            },
            onSelect: function(item) {
                self.selectedItem = item;
                input.value = item.ID;

                self.fnIdSelected && self.fnIdSelected(item.ID);
                input.blur();
            },
            // attach container to AL div if needed (to prevent it from being hidden in full screen mode)
            customize: function(input, inputRect, container, maxHeight) {
                // this tests if we are in full screen mode
                if (self.aladin.fullScreenBtn.hasClass('aladin-restore')) {
                    self.parentDiv.appendChild(container);
                }
            },
            render: function(item, currentValue) {
                const itemElement = document.createElement("div");
                itemElement.innerHTML = item.obs_title + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';


                return itemElement;
            }
        });

        // this modal is closed when clicking on the cross at the top right
        let [selectBtn, loadMOCBtn]  = this.mainDiv.querySelectorAll('.aladin-btn');
        let [closeBtn]  = this.mainDiv.querySelectorAll('.aladin-closeBtn');

        $(closeBtn).click(function() {
            self.hide();
        });

        // when 'Select' is pressed, call the callbacks
        $(selectBtn).click(function() {
            let byIdSelected = self.mainDiv.querySelectorAll('input')[0];

            if (byIdSelected) {
                self.fnIdSelected && self.fnIdSelected(byIdSelected.value);
            }

            byIdSelected.value = '';
        
            //self.hide();
        });

        $(loadMOCBtn).click(function() {
            let url;
            let byIdSelected = self.mainDiv.querySelectorAll('input')[0];
            if (byIdSelected.value.startsWith('http')) {
                url = byIdSelected.value + '/Moc.fits';
            }
            else {
                url = self.selectedItem.hips_service_url + '/Moc.fits';
            }
            url = Utils.fixURLForHTTPS(url);

            const moc = A.MOCFromURL(url);
            self.aladin.addMOC(moc);
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
