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

MeasurementTable = (function() {


    // constructor
    MeasurementTable = function(aladinLiteDiv) {
        this.isShowing = false;

        this.divEl = $('<div class="aladin-measurement-div"></div>');
        
        $(aladinLiteDiv).append(this.divEl);
    }

    // show measurement associated with a given source
    MeasurementTable.prototype.showMeasurement = function(source) {
        this.divEl.empty();
        var header = '<thead><tr>';
        var content = '<tr>';
        for (key in source.data) {
            header += '<th>' + key + '</th>';
            content += '<td>' + source.data[key] + '</td>';
        }
        header += '</tr></thead>';
        content += '</tr>';
        this.divEl.append('<table>' + header + content + '</table>');
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

