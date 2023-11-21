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

import { Utils } from "../Utils.js";
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
    }
 
    static _createTableBody = function(opt) {
        const tbody = document.createElement('tbody');

        opt.rows.forEach((row) => {
            let trEl = document.createElement('tr');

            for (let key in row.data) {
                // check the type here

                let tdEl = document.createElement('td');
                tdEl.classList.add(key);

                if (opt.showCallback && opt.showCallback[key]) {
                    let showFieldCallback = opt.showCallback[key];

                    let el = showFieldCallback(row.data);
                    Utils.appendTo(el, tdEl);
                } else {
                    let val = row.data[key] || '--';
                    tdEl.innerHTML = val;
                    tdEl.classList.add("aladin-text-td-container");
                }

                trEl.appendChild(tdEl);
            }

            tbody.appendChild(trEl);
        });

        return tbody;
    }
 
    /*MeasurementTable.prototype.createTabs = function() {
        let self = this;
        let layout = [];
        this.tables.forEach(function(table, index) {
            let backgroundColor = table["color"];
            let hexStdColor = Color.standardizeColor(table["color"]);
            let rgbColor = Color.hexToRgb(hexStdColor);
            rgbColor = 'rgb(' + rgbColor.r + ', ' + rgbColor.g + ', ' + rgbColor.b + ')';
            let labelColor = Color.getLabelColorForBackground(rgbColor);

            let textContent = '<div style="overflow: hidden; text-overflow: ellipsis;white-space: nowrap;max-width: 20em;">' +
            table.name + '</div>';

            let tabContent = Layout.horizontal(['<div class="aladin-stack-icon" style="background-image: url(&quot;data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwb2x5Z29uIHBvaW50cz0iMSwwLDUsMCw1LDMsMSwzIiAgZmlsbD0iIzk5Y2MwMCIgLz48cG9seWdvbiBwb2ludHM9IjcsMCw5LDAsOSwzLDcsMyIgIGZpbGw9IiM5OWNjMDAiIC8+PHBvbHlnb24gcG9pbnRzPSIxMCwwLDEyLDAsMTIsMywxMCwzIiAgZmlsbD0iIzk5Y2MwMCIgLz48cG9seWdvbiBwb2ludHM9IjEzLDAsMTUsMCwxNSwzLDEzLDMiICBmaWxsPSIjOTljYzAwIiAvPjxwb2x5bGluZSBwb2ludHM9IjEsNSw1LDkiICBzdHJva2U9IiM5OWNjMDAiIC8+PHBvbHlsaW5lIHBvaW50cz0iMSw5LDUsNSIgc3Ryb2tlPSIjOTljYzAwIiAvPjxsaW5lIHgxPSI3IiB5MT0iNyIgeDI9IjE1IiB5Mj0iNyIgc3Ryb2tlPSIjOTljYzAwIiBzdHJva2Utd2lkdGg9IjIiIC8+PHBvbHlsaW5lIHBvaW50cz0iMSwxMSw1LDE1IiAgc3Ryb2tlPSIjOTljYzAwIiAvPjxwb2x5bGluZSBwb2ludHM9IjEsMTUsNSwxMSIgIHN0cm9rZT0iIzk5Y2MwMCIgLz48bGluZSB4MT0iNyIgeTE9IjEzIiB4Mj0iMTUiIHkyPSIxMyIgc3Ryb2tlPSIjOTljYzAwIiBzdHJva2Utd2lkdGg9IjIiIC8+PC9zdmc+&quot;);"></div>', textContent]);

            layout.push({
                label: tabContent,
                title: table["name"],
                cssStyle: {
                    backgroundColor: backgroundColor,
                    color: labelColor,
                },
                content: tabContent
                action(e) {
                    self.curTableIdx = index;

                    let tableElement = self.element.querySelector('table');
                    tableElement.style.borderColor = table["color"]

                    let thead = self.element.querySelector("thead");
                    // replace the old header with the one of the current table
                    thead.parentNode.replaceChild(MeasurementTable.createTableHeader(table), thead);
                    
                    self.updateTableBody()
                }
            });
        });

        return new Tabs(layout, self.aladinLiteDiv);
    }*/

    static _createTableHeader = function(opt) {
        let theadElement = document.createElement('thead');
        var content = '<tr>';

        for (let [_, field] of Object.entries(opt.fields)) {
            if (field.name) {
                content += '<th>' + field.name + '</th>';
            }
        }
        content += '</tr>';

        theadElement.innerHTML = content;

        return theadElement;
    }
}
 
 