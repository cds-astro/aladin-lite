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
 * File Table
 *
 * Graphic object showing measurement of a catalog
 * 
 * Author: Matthieu Baumann[CDS], Thomas Boch[CDS]
 * 
 *****************************************************************************/

import { DOMElement } from "./Widget.js";

export class Table extends DOMElement {
    // constructor
    constructor(options, target, position = 'beforeend') {
        let el = document.createElement('div');
        el.setAttribute("class", "aladin-measurement-div");

        let tableEl = document.createElement('table');
        tableEl.style.borderColor = options.color;

        // table header creation
        const thead = Table._createTableHeader(options);
        const tbody = Table._createTableBody(options);
        // table body creation
        tableEl.appendChild(thead);
        tableEl.appendChild(tbody);

        el.appendChild(tableEl);

        super(el, options);
        this.attachTo(target, position);

        this.addClass("aladin-dark-theme")
    }
 
    static _createTableBody = function(options) {
        const tbody = document.createElement('tbody');

        options.rows.forEach((row) => {
            let trEl = document.createElement('tr');

            for (let key in row.data) {
                // check the type here

                let tdEl = document.createElement('td');
                tdEl.classList.add(key);

                if (options.showCallback && options.showCallback[key]) {
                    let showFieldCallback = options.showCallback[key];

                    let el = showFieldCallback(row.data);
                    if (el instanceof Element) {
                        tdEl.appendChild(el);
                    } else {
                        tdEl.innerHTML = el;
                    }
                } else {
                    let val = row.data[key] || '--';
                    tdEl.innerHTML = val;
                    tdEl.classList.add("aladin-text-td-container");
                    tdEl.title = val;
                }

                trEl.appendChild(tdEl);
            }

            tbody.appendChild(trEl);
        });

        return tbody;
    }
 
    static _createTableHeader = function(options) {
        let theadElement = document.createElement('thead');
        var content = '<tr>';

        for (let [_, field] of Object.entries(options.fields)) {
            if (field.name) {
                content += '<th>' + field.name + '</th>';
            }
        }
        content += '</tr>';

        theadElement.innerHTML = content;

        return theadElement;
    }
}
 
 