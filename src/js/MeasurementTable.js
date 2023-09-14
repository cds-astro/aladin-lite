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
import { ActionButton } from "./gui/widgets/ActionButton.js";

export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(aladinLiteDiv) {
        this.isShowing = false;

        let mainDiv = document.createElement('div');
        mainDiv.setAttribute("class", "aladin-measurement-div");
        this.element = mainDiv;

        aladinLiteDiv.appendChild(this.element);
    }

    MeasurementTable.prototype.updateTableBody = function() {
        let tbody = this.element.querySelector('tbody');
        tbody.innerHTML = '';

        let table = this.tables[this.curTableIdx];

        table["rows"].forEach((row) => {
            let trEl = document.createElement('tr');

            for (let key in row.data) {
                // check the type here

                let tdEl = document.createElement('td');
                tdEl.classList.add(key);

                if (table.showCallback && table.showCallback[key]) {
                    let showFieldCallback = table.showCallback[key];

                    let el = showFieldCallback(row.data);
                    if (el instanceof Element) {
                        tdEl.appendChild(el);
                    } else {
                        tdEl.innerHTML = el;
                    }
                } else {
                    let val = row.data[key] || '--';
                    tdEl.innerText = val;
                }

                trEl.appendChild(tdEl);
            }

            tbody.appendChild(trEl);
        });
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(tables) {
        if (tables.length === 0) {
            return;
        }

        this.update(tables);
    };

    MeasurementTable.prototype.update = function(tables) {
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
        this.updateTableBody();

        this.show();
    }

    MeasurementTable.prototype.createTabs = function() {
        let tabsElement = document.createElement('div')
        tabsElement.setAttribute('class', 'tabs');

        /// Create catalog tabs
        let tabsButtonElement = [];

        let self = this;
        this.tables.forEach(function(table, index) {
            let tabButtonElement = document.createElement("button");
            tabButtonElement.setAttribute('title', table["name"])

            tabButtonElement.innerText = table["name"];
            tabButtonElement.style.overflow = 'hidden';
            tabButtonElement.style.textOverflow = 'ellipsis';
            tabButtonElement.style.whiteSpace = 'nowrap';
            tabButtonElement.style.maxWidth = '20%';

            tabButtonElement.addEventListener(
                'click',
                () => {
                    self.curTableIdx = index;

                    let tableElement = self.element.querySelector('table');
                    tableElement.style.borderColor = table["color"]

                    let thead = self.element.querySelector("thead");
                    // replace the old header with the one of the current table
                    thead.parentNode.replaceChild(MeasurementTable.createTableHeader(table), thead);

                    self.updateTableBody()
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
            if (field.name) {
                content += '<th>' + field.name + '</th>';
            }
        }
        content += '</tr>';

        theadElement.innerHTML = content;

        return theadElement;
    }

    MeasurementTable.prototype.show = function() {
        this.element.style.visibility = "visible";
    };

    MeasurementTable.prototype.hide = function() {
        this.curTableIdx = 0;

        this.element.style.visibility = "hidden";
    };

    return MeasurementTable;
})();

