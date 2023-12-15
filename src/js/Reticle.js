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

import iconUrl from './../../assets/icons/reticle.svg'
import { Color } from './Color.js';

/******************************************************************************
 * Aladin Lite project
 * 
 * File Source
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { Aladin } from "./Aladin";
import { ALEvent } from './events/ALEvent';
 
export let Reticle = (function() {
     // constructor
    let Reticle = function(options, aladin) {
        this.el = document.createElement('object');
        this.el.className = 'aladin-reticle';
        this.el.type = "image/svg+xml";
        this.el.data = iconUrl;
        this.aladin = aladin;

        this.color = null;

        aladin.aladinDiv.appendChild(this.el);

        let color = options && options.color || Aladin.DEFAULT_OPTIONS.reticleColor;
        let size = options && options.size || Aladin.DEFAULT_OPTIONS.reticleSize;
        
        let show;
        if (options.showReticle === undefined) {
            show = Aladin.DEFAULT_OPTIONS.showReticle;
        } else {
            show = options && options.showReticle;
        }

        let self = this;
        this.loaded = new Promise((resolve, reject) => {
            function handleLoad(event) {
                /* Removes the listeners */
                self.el.removeEventListener('load', handleLoad);
                resolve(event); // works just fine
            }

            function handleError(event) {
                /* Removes the listeners */
                self.el.removeEventListener('error', handleError);
                reject(event); // works just fine
            }

            self.el.addEventListener('load', handleLoad);
            self.el.addEventListener('error', handleError);
        });

        this.update({color, size, show})
    };

    Reticle.prototype.update = async function(options) {
        options = options || {};
        await this.loaded;

        if (options.size) {
            this._setSize(options.size)
        }

        if (options.color) {
            this._setColor(options.color)
        }

        if (options.show !== undefined) {
            this._show(options.show)
        }

        ALEvent.RETICLE_CHANGED.dispatchedTo(this.aladin.aladinDiv, {
            color: this.color,
            size: this.size,
            show: this.visible
        })
    }

    Reticle.prototype._setColor = function(color) {
        if (!color) {
            return;
        }

        // 1. the user has maybe given some
        let reticleColor = new Color(color);
        // a dynamic way to set the color
        this.color = 'rgb(' + reticleColor.r + ', ' + reticleColor.g + ', ' + reticleColor.b + ')';

        this.el.contentDocument
            .getElementById("reticle")
            .setAttribute('fill', this.color);

    }

    Reticle.prototype._setSize = function(size) {
        if (!size) {
            return;
        }

        this.size = size;
        this.el.style.width = this.size + 'px';
        this.el.style.height = this.size + 'px';
    }

    Reticle.prototype._show = function(show) {
        if (show === undefined) {
            return;
        }

        if (show === true) {
            this.el.style.visibility = 'visible';
        } else {
            this.el.style.visibility = 'hidden';
        }

        this.visible = show;
    }

    Reticle.prototype.getColor = function() {
        return this.color;
    }

    Reticle.prototype.getSize = function() {
        return this.size;
    }

    Reticle.prototype.isVisible = function() {
        return this.visible;
    }

    return Reticle;
 })();
 