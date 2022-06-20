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
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

 export class HiPSSelector {

    constructor(parentDiv, fnIdSelected) {
        this.parentDiv = parentDiv;

        this.fnIdSelected  = fnIdSelected;

        this.#createComponent();
    }

    #createComponent() {
        this.mainDiv = document.createElement('div');
        this.mainDiv.style.display = 'block';
        this.mainDiv.classList.add('aladin-dialog', 'aladin-cb-list');

        const autocompleteId = 'autocomplete-' + Utils.uuidv4();
        this.mainDiv.insertAdjacentHTML('afterbegin', 
          '<a class="aladin-closeBtn">&times;</a>' +
          '<div class="aladin-box-title">Select HiPS:</div>' +
          '<div class="aladin-box-content">' +

                '<div class="aladin-label" for="' + autocompleteId + '">By ID, title, keyword or URL</div>' +
                '<input name="' + autocompleteId + '" id="' + autocompleteId + '" type="text" placeholder="Type ID, title, keyword or URL" />' +

            '<div>' +
                '<button class="aladin-btn">Select</button>' +
                '<button class="aladin-btn aladin-cancelBtn">Cancel</button>' +
            '</div>' +
          '</div>'
        );

        this.parentDiv.appendChild(this.mainDiv);

        // setup autocomplete
        let input = document.getElementById(autocompleteId);
        
        // Query the mocserver
        MocServer.getAllHiPSes();
        autocomplete({
            input: input,
            fetch: function(text, update) {
                text = text.toLowerCase();
                // you can also use AJAX requests instead of preloaded data
                var suggestions = MocServer.getAllHiPSes().filter(n => n.ID.toLowerCase().includes(text) || n.obs_title.toLowerCase().includes(text))
                update(suggestions);
            },
            onSelect: function(item) {
                input.value = item.ID;
            },
            render: function(item, currentValue) {
                const itemElement = document.createElement("div");
                itemElement.innerHTML = item.obs_title + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';


                return itemElement;
            }
        });

        // this modal is closed when clicking on the cross at the top right, or on the Cancel button
        let [selectBtn, cancelBtn]  = this.mainDiv.querySelectorAll('.aladin-btn');
        let [closeBtn]  = this.mainDiv.querySelectorAll('.aladin-closeBtn');

        var self = this;
        $(closeBtn).add($(cancelBtn)).click(function() {
            self.hide();
        });

        // when 'Select' is pressed, call the callbacks
        $(selectBtn).click(function() {
            let byIdSelected = self.mainDiv.querySelectorAll('input')[0];

            if (byIdSelected) {
                self.fnIdSelected(byIdSelected.value);
            }

            byIdSelected.value = '';
        
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
