// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
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

import { Icon } from "./gui/Widgets/Icon.js";
import { Tabs } from "./gui/Widgets/Tab.js";
import { Table } from "./gui/Widgets/Table.js";
import { ActionButton } from "./gui/Widgets/ActionButton.js";


export let MeasurementTable = (function() {

    // constructor
    function MeasurementTable(aladin) {
        this.aladin = aladin;
        this.tables = {};
    }

    MeasurementTable.prototype.addTab = function(table) {
        let tables = [].concat(table);

        for (let table of tables) {
            this.tables[table.name] = table;
        }

        this.show();
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.show = function() {
        let self = this;

        let i = 0;
        let layout = [];
        for (let tabName in this.tables) {
            let table = this.tables[tabName];
            let content = new Table(table);

            let textContent = '<div style="overflow: hidden; text-overflow: ellipsis;white-space: nowrap;max-width: 20em;">' +
            table.name + '</div>';

            let label = new ActionButton({
                icon: {
                    size: 'small',
                    url: table.icon || Icon.dataURLFromSVG({svg: Icon.SVG_ICONS.CATALOG, color: table.color}),
                },
                content: [
                    textContent,
                    new ActionButton({
                        size: 'small',
                        content: '❌',
                        action(_) {
                            self.hideTab(table.name)
                        },
                        cssStyle: {
                            padding: 0,
                            border: 0,
                        },
                    })
                ],
            })

            i++;

            layout.push({
                title: table.name,
                label,
                content,
            })
        }

        let scrollLeftMemorized = null;
        if (this.table) {
            scrollLeftMemorized = this.table.scrollLeftPosition;
            this.table.remove();
        }

        this.table = new Tabs({
            tooltip: {
                global: true,
                aladin: this.aladin,
                content: 'Scroll to see more...'
            },
            aladin: this.aladin,
            layout,
        }, this.aladin.aladinDiv);

        if (scrollLeftMemorized !== null) {
            this.table.setScrollPosition(scrollLeftMemorized)
        }
    };

    MeasurementTable.prototype.hideAll = function() {
        this.tables = {}

        this.show();
    };

    MeasurementTable.prototype.hideTab = function(tabName) {
        delete this.tables[tabName];

        this.show();
    };

    return MeasurementTable;
})();

