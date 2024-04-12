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
 * File Popup.js
 *
 * Author: Thomas Boch [CDS]
 *
 *****************************************************************************/
// TODO replace with Box
export let Popup = (function() {

    // constructor
    function Popup(parentDiv, view) {
        let el = document.createElement('div');
        el.className = 'aladin-popup-container';
        el.innerHTML = '<div class="aladin-popup"><a class="aladin-closeBtn">&times;</a><div class="aladin-popupTitle"></div><div class="aladin-popupText"></div></div><div class="aladin-popup-arrow"></div>';
        this.domEl = el;

        parentDiv.appendChild(this.domEl);

        this.view = view;

        var self = this;
        // close popup
        this.domEl.querySelector('.aladin-closeBtn').addEventListener("click", () => {self.hide()});
    };

    Popup.prototype.hide = function() {
        this.isShown = false;
        this.domEl.style.display = "none";

        this.view.mustClearCatalog=true;
        this.view.catalogForPopup.hide();
        this.view.overlayForPopup.hide();
    };

    Popup.prototype.show = function() {
        this.isShown = true;
        this.domEl.style.display = "block";
    };

    Popup.prototype.setTitle = function(title) {
        this.domEl.querySelector('.aladin-popupTitle').innerHTML = title || '';
    };

    Popup.prototype.setText = function(text) {
        this.domEl.querySelector('.aladin-popupText').innerHTML = text || '';
        if (!this.isShown) {
            // offsetWidth and offsetHeight are gettable
            // only if the dom element is displayed
            // so we display it and hide it just after
            // if it is supposed to be hidden
            this.domEl.style.display = "block";

            this.w = this.domEl.offsetWidth;
            this.h = this.domEl.offsetHeight;

            this.domEl.style.display = "none";
        } else {
            this.w = this.domEl.offsetWidth;
            this.h = this.domEl.offsetHeight;
        }
    };

    Popup.prototype.setSource = function(source) {
        // remove reference to popup for previous source
        if (this.source) {
            this.source.popup = null;
        }
        source.popup = this;
        this.source = source;
        this.setPosition(source.x, source.y);
    };

    Popup.prototype.setPosition = function(x, y) {
        var newX = x - this.w/2;
        var newY = y - this.h;

        if (this.source) {
            newY += this.source.catalog.sourceSize/2;
        }

        this.domEl.style.left = newX + 'px';
        this.domEl.style.top  = newY + 'px';
    };

    return Popup;
})();

