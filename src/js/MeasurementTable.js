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
 * File MeasurementTable
 *
 * Graphic object showing measurement of a catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

import { Color } from "./Color.js"

export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(aladinLiteDiv) {
        this.isShowing = false;

        let mainDiv = document.createElement('div');
        mainDiv.setAttribute("class", "aladin-measurement-div");
        this.element = mainDiv;
        aladinLiteDiv.appendChild(this.element);
    }

    MeasurementTable.prototype.updateRows = function() {
        let tbody = this.element.querySelector('tbody');
        
        tbody.innerHTML = "";

        let table = this.tables[this.curTableIdx];

        let result = '';
        table["rows"].forEach((row) => {
            result += '<tr>'
            for (let key in row.data) {
                // check the type here
                const val = row.data[key];
                result += '<td class="' + key + '">'
                if (typeof(val) === "string") {
                    try {
                        let url = new URL(val);
                        let link = '<a href=' + url + ' target="_blank">' + url + '</a>';
                        result += link;
                    } catch(e) {
                        result += val
                    }
                } else {
                    result += val
                }
                result += '</td>'
            }
            result += '</tr>';
        });

        tbody.innerHTML = result;

        if (table["fieldsClickedActions"]) {
            for (let key in table["fieldsClickedActions"]) {
                tbody.querySelectorAll("." + key).forEach(function(e, index) {
                    e.addEventListener('click', (e) => {
                        let callback = table["fieldsClickedActions"][key];
                        callback(table["rows"][index].data)

                        e.preventDefault();
                    }, false)
                })
            }
        }
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(tables) {
        if (tables.length === 0) {
            return;
        }

        this.tables = tables;
        this.curTableIdx = 0;

        let table = tables[this.curTableIdx];
        this.element.innerHTML = "";

        /// Create tabs element
        let tabsElement = this.createTabs();
        this.element.appendChild(tabsElement);

        /// Create table element
        let tableElement = document.createElement('table');
        tableElement.style.borderColor = table['color'];

        // table header creation
        const thead = MeasurementTable.createTableHeader(table);
        // table body creation
        const tbody = document.createElement('tbody');

        tableElement.appendChild(thead);
        tableElement.appendChild(tbody);
        this.element.appendChild(tableElement);

        this.updateRows();

        this.show();
    };

    MeasurementTable.prototype.createTabs = function() {
        let tabsElement = document.createElement('div')
        tabsElement.setAttribute('class', 'tabs');

        let tabsButtonElement = [];

        let self = this;
        this.tables.forEach(function(table, index) {
            let tabButtonElement = document.createElement("button");
            tabButtonElement.innerText = table["name"];

            tabButtonElement.addEventListener(
                'click',
                () => {
                    self.curTableIdx = index;

                    let tableElement = self.element.querySelector('table');
                    tableElement.style.borderColor = table["color"]

                    self.updateRows()
                }
                ,false
            );

            tabButtonElement.style.backgroundColor = table["color"];

            let hexStdColor = Color.standardizeColor(table["color"]);
            let rgbColor = Color.hexToRgb(hexStdColor);
            rgbColor = 'rgb(' + rgbColor.r + ', ' + rgbColor.g + ', ' + rgbColor.b + ')';

            let labelColor = Color.getLabelColorForBackground(rgbColor);
            tabButtonElement.style.color = labelColor

            tabsButtonElement.push(tabButtonElement);
            tabsElement.appendChild(tabButtonElement);
        });

        return tabsElement;
    }

    MeasurementTable.createTableHeader = function(table) {
        let theadElement = document.createElement('thead');
        var content = '<tr>';

        for (let [_, field] of Object.entries(table["fields"])) {
            content += '<th>' + field.name + '</th>';
        }
        content += '</thead>';

        theadElement.innerHTML = content;

        return theadElement;
    }

    MeasurementTable.prototype.show = function() {
        this.element.style.visibility = "visible";
    };

    MeasurementTable.prototype.hide = function() {
        this.element.style.visibility = "hidden";
    };

    return MeasurementTable;
})();

