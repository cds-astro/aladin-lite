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
import { Icon } from "./gui/Widgets/Icon.js";
import { Tabs } from "./gui/Widgets/Tab.js";
import { Table } from "./gui/Widgets/Table.js";
import { ActionButton } from "./gui/Widgets/ActionButton.js";


export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(aladin) {
        this.aladin = aladin;
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(tables) {
        if (tables.length === 0) {
            return;
        }

        let layout = tables.map((table) => {
            let content = new Table(table);

            let textContent = '<div style="overflow: hidden; text-overflow: ellipsis;white-space: nowrap;max-width: 20em;">' +
            table.name + '</div>';

            let label = new ActionButton({
                icon: {
                    size: 'small',
                    url: Icon.dataURLFromSVG({svg: Icon.SVG_ICONS.CATALOG, color: table.color}),
                },
                content: textContent,
            })

            return {
                title: table.name,
                label,
                content,
                /*cssStyle: {
                    backgroundColor: rgbColor,
                    color: labelColor,
                    padding: '2px',
                }*/
            }
        });

        this.hide();

        this.table = new Tabs({
            aladin: this.aladin,
            layout,
            cssStyle: {
                position: 'absolute',
                bottom: '2.4rem',
                maxWidth: '100%',
            }
        }, this.aladin.aladinDiv);
    };

    MeasurementTable.prototype.hide = function() {
        if (this.table) {
            this.table.remove();
        }
    };

    return MeasurementTable;
})();

