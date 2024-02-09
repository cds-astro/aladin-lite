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
import { Layout } from "./gui/Layout.js";
import { Tabs } from "./gui/Widgets/Tab.js";
import { Table } from "./gui/Widgets/Table.js";

export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(target) {
        //this.isShowing = false;
        this.target = target;
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(tables) {
        if (tables.length === 0) {
            return;
        }

        let layout = tables.map((table) => {
            let content = new Table(table);

            //let backgroundColor = table["color"];
            let hexStdColor = Color.standardizeColor(table.color);
            let rgbColor = Color.hexToRgb(hexStdColor);
            rgbColor = 'rgb(' + rgbColor.r + ', ' + rgbColor.g + ', ' + rgbColor.b + ')';
            let labelColor = Color.getLabelColorForBackground(rgbColor);

            let textContent = '<div style="overflow: hidden; text-overflow: ellipsis;white-space: nowrap;max-width: 20em;">' +
            table.name + '</div>';

            let label = Layout.horizontal({
                layout: [
                    '<div class="aladin-stack-icon" style="background-image: url(&quot;data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwb2x5Z29uIHBvaW50cz0iMSwwLDUsMCw1LDMsMSwzIiAgZmlsbD0iIzk5Y2MwMCIgLz48cG9seWdvbiBwb2ludHM9IjcsMCw5LDAsOSwzLDcsMyIgIGZpbGw9IiM5OWNjMDAiIC8+PHBvbHlnb24gcG9pbnRzPSIxMCwwLDEyLDAsMTIsMywxMCwzIiAgZmlsbD0iIzk5Y2MwMCIgLz48cG9seWdvbiBwb2ludHM9IjEzLDAsMTUsMCwxNSwzLDEzLDMiICBmaWxsPSIjOTljYzAwIiAvPjxwb2x5bGluZSBwb2ludHM9IjEsNSw1LDkiICBzdHJva2U9IiM5OWNjMDAiIC8+PHBvbHlsaW5lIHBvaW50cz0iMSw5LDUsNSIgc3Ryb2tlPSIjOTljYzAwIiAvPjxsaW5lIHgxPSI3IiB5MT0iNyIgeDI9IjE1IiB5Mj0iNyIgc3Ryb2tlPSIjOTljYzAwIiBzdHJva2Utd2lkdGg9IjIiIC8+PHBvbHlsaW5lIHBvaW50cz0iMSwxMSw1LDE1IiAgc3Ryb2tlPSIjOTljYzAwIiAvPjxwb2x5bGluZSBwb2ludHM9IjEsMTUsNSwxMSIgIHN0cm9rZT0iIzk5Y2MwMCIgLz48bGluZSB4MT0iNyIgeTE9IjEzIiB4Mj0iMTUiIHkyPSIxMyIgc3Ryb2tlPSIjOTljYzAwIiBzdHJva2Utd2lkdGg9IjIiIC8+PC9zdmc+&quot;);"></div>',
                    textContent
                ]
            });

            return {
                title: table.name,
                label: label,
                content: content,
                cssStyle: {
                    backgroundColor: rgbColor,
                    color: labelColor,
                    padding: '2px',
                }
            }
        });

        if (this.table) {
            this.table.remove();
        }

        this.table = new Tabs({
            layout: layout,
            cssStyle: {
                position: 'absolute',
                bottom: '27px',
                maxWidth: '100%',
            }
        }, this.target);
    };

    MeasurementTable.prototype.hide = function() {
        if (this.table) {
            this.table.remove();
        }
    };

    return MeasurementTable;
})();

