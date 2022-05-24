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
 * File events/ALEvent
 * 
 * List of events emitted by Aladin Lite
 * 
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/

 export class ALEvent {
    static COO_GRID_ENABLED  = new ALEvent("AL:cooGrid.enabled");
    static COO_GRID_DISABLED = new ALEvent("AL:cooGrid.disabled");
    static COO_GRID_UPDATED  = new ALEvent("AL:cooGrid.updated");
  
    constructor(name) {
      this.name = name;
    }

    dispatchedTo(domEl, options) {
        if (options) {
          domEl.dispatchEvent(new CustomEvent(this.name, {detail: options}));
        }
        else {
          domEl.dispatchEvent(new CustomEvent(this.name));
        }
    }

    listenedBy(domEl, fn) {
        domEl.addEventListener(this.name, fn);
    }
  }
  