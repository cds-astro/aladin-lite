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

import $ from 'jquery';

export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(aladinLiteDiv) {
        this.isShowing = false;
        this.divEl = $('<div class="aladin-measurement-div"></div>');

        this.curPage = 1;
        this.numPages = 1;
        this.numRowsByPage = 5;

        this.columnClickAction = {};

        $(aladinLiteDiv).append(this.divEl);
    }

    MeasurementTable.updateBodyTable = function(rows) {
        let tbody = '<tbody class="content">';
        rows.forEach(row => {
            tbody += '<tr>'
            for (let key in row.data) {
                // check the type here
                const val = row.data[key];
                tbody += '<td class="' + key + '">'
                if (typeof(val) === "string") {
                    try {
                        let url = new URL(val);
                        let link = '<a href=' + url + '>' + url + '</a>';
                        tbody += link;
                    } catch(e) {
                        tbody += val
                    }
                } else {
                    tbody += val
                }
                tbody += '</td>'
            }
            tbody += '</tr>';
        });
        tbody += '</tbody>';

        return tbody;
    }

    MeasurementTable.prototype.renderTable = function(fullRows) {
        // idx of the first row to draw
        let rowIdxStart = (this.curPage - 1) * this.numRowsByPage;
        // update the content
        let tbody = this.divEl[0].querySelector(".content");
        tbody.innerHTML = MeasurementTable.updateBodyTable(fullRows.slice(rowIdxStart, rowIdxStart + this.numRowsByPage));
    
        if (this.fieldsClickCallbacks) {
            Object.entries(this.fieldsClickCallbacks)
            .forEach(([key, callback]) => {
                this.divEl[0].querySelectorAll("." + key).forEach((e) => {
                    e.addEventListener('click', (e) => {
                        callback(e.target.innerText)

                        e.preventDefault();
                    }, false)
                })
            });
        }

        // recompute page idx
        let pageIdxElt = this.divEl[0].querySelector("#pageIdx");
        pageIdxElt.innerHTML = '<p id="pageIdx" style="display: inline-block; margin: 0">' + this.curPage + '/' + this.numPages + '</p>';
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(rows, table) {
        this.fieldsClickCallbacks = table.fieldsClickCallbacks;
        // compute the number of pages
        this.numPages = Math.floor(rows.length / this.numRowsByPage);
        if (rows.length % this.numRowsByPage > 0) {
             // handles rows leftovers in a last page
            this.numPages++;
        }
        
        this.divEl.empty();
        var thead = '<thead><tr>';
        for (let key in rows[0].data) {
            thead += '<th>' + key + '</th>';
        }
        thead += '</tr></thead>';

        let tbody = MeasurementTable.updateBodyTable(rows.slice(0, this.numRowsByPage));
        this.divEl.append('<table>' + thead + tbody + '</table>');
        // Add the callbacks to the cells
        if (this.fieldsClickCallbacks) {
            Object.entries(this.fieldsClickCallbacks)
            .forEach(([key, callback]) => {
                this.divEl[0].querySelectorAll("." + key).forEach((e) => {
                    e.addEventListener('click', (e) => {
                        callback(e.target.innerText)

                        e.preventDefault();
                    }, false)
                })
            });
        }

        if (this.numPages > 1) {
            this.divEl.append('<div class="footer"><button id="prevButton" style="display: inline-block">Previous</button><button id="nextButton" style="display: inline-block">Next</button><p id="pageIdx" style="display: inline-block; margin: 0">' + this.curPage + '/' + this.numPages + '</p></div>');

            this.divEl[0].querySelector('#nextButton').addEventListener(
                'click',
                () => {
                    this.curPage++;
                    if (this.curPage >= this.numPages) {
                        this.curPage = this.numPages;
                    }
    
                    this.renderTable(rows)
                }
                ,false
            );

            this.divEl[0].querySelector('#prevButton').addEventListener(
                'click',
                () => {
                    this.curPage--;
                    if (this.curPage < 1) {
                        this.curPage = 1;
                    }

                    this.renderTable(rows)
                }
                ,false
            );
        }

        this.show();
    };

    MeasurementTable.prototype.show = function() {
        this.divEl.show();
    };

    MeasurementTable.prototype.hide = function() {
        this.divEl.hide();
    };

    return MeasurementTable;
})();

