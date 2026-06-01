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
 * File events/ALEvent
 * 
 * List of events emitted by Aladin Lite
 * 
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/

export class ALEvent {
  static AL_USE_WASM = new ALEvent("AL:Wasm");

  static LOADING_START = new ALEvent("AL:loading.started");
  static LOADING_STOP = new ALEvent("AL:loading.stopped");

  static BACKGROUND_COLOR_CHANGED = new ALEvent("AL:BackgroundColor.changed")

  static COO_GRID_ENABLED  = new ALEvent("AL:cooGrid.enabled");
  static COO_GRID_DISABLED = new ALEvent("AL:cooGrid.disabled");
  static COO_GRID_UPDATED  = new ALEvent("AL:cooGrid.updated");

  static PROJECTION_CHANGED  = new ALEvent("AL:projection.changed");
  static FRAME_CHANGED  = new ALEvent("AL:frame.changed");

  static UPDATE_CMAP_LIST  = new ALEvent("AL:cmap.updated");

  // Gives the center position in ICRS
  static POSITION_CHANGED  = new ALEvent("AL:position.changed");
  static ZOOM_CHANGED  = new ALEvent("AL:zoom.changed");

  static LAYER_ADDED   = new ALEvent("AL:Layer.added");
  static LAYER_REMOVED = new ALEvent("AL:Layer.removed");
  static LAYER_SWAPPED = new ALEvent("AL:Layer.swapped");
  static LAYER_CHANGED  = new ALEvent("AL:Layer.changed");

  static HIPS_CACHE_UPDATED = new ALEvent("AL:HiPSCache.updated");
  static FAVORITE_LAYERS_LIST_UPDATED = new ALEvent("AL:HiPSFavorites.updated");

  static GRAPHIC_OVERLAY_LAYER_ADDED  = new ALEvent("AL:GraphicOverlayLayer.added");
  static GRAPHIC_OVERLAY_LAYER_REMOVED  = new ALEvent("AL:GraphicOverlayLayer.removed");

  static GRAPHIC_OVERLAY_LAYER_CHANGED  = new ALEvent("AL:GraphicOverlayLayer.changed");

  static SAMP_HUB_RUNNING  = new ALEvent("AL:samp.hub");
  static SAMP_CONNECTED  = new ALEvent("AL:samp.connected");
  static SAMP_DISCONNECTED  = new ALEvent("AL:samp.disconnected");

  static CANVAS_EVENT = new ALEvent("AL:Event");

  static RETICLE_CHANGED = new ALEvent("AL:Reticle.changed")

  static RESOURCE_FETCHED = new ALEvent("AL:Resource.fetched")
  static FETCH = new ALEvent("AL:fetch")

  static MODE = new ALEvent("AL:mode")

  static FULLSCREEN_TOGGLED = new ALEvent("AL:fullscreen.toggled")

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

  remove(domEl, fn) {
    domEl.removeEventListener(this.name, fn);
  }
}
  
