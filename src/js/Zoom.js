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
 * File Tile
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/
import { Utils } from "./Utils";
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";


 export let Zoom = (function() {
    // constructor
	function Zoom(view) {
        this.view = view;
	};
	
    Zoom.LEVELS = [
        360, 330, 300, 275, 250, 225, 200, 190,
        180, 170, 160, 150, 140, 130, 120, 110, 100,
        95, 90, 85, 80, 75, 70, 65, 60, 55, 50, 45, 40, 35, 30, 25, 20, 18, 16, 14, 12, 10,
        9, 8, 7, 6, 5, 4, 3, 2, 1.75, 1.5, 1.25, 1,
        55/60, 50/60, 45/60, 40/60, 35/60, 30/60, 25/60, 20/60, 15/60, 10/60,
        9/60, 8/60, 7/60, 6/60, 5/60, 4/60, 3/60, 2/60, 1/60,
        50/3600, 40/3600, 30/3600, 20/3600, 10/3600,
        9/3600, 8/3600, 7/3600, 6/3600, 5/3600, 4/3600, 3/3600, 2/3600, 1/3600,
        9/36000, 8/36000, 7/36000, 6/36000, 5/36000, 4/36000, 3/36000, 2/36000, 1/36000
    ];
    Zoom.MAX_IDX_DELTA_PER_TROTTLE = 2;

    Zoom.determineNextFov = function(view, amount) {
        if (!view.idx)
            view.idx = Utils.binarySearch(Zoom.LEVELS, view.fov);

        let deltaIdx = amount;
        view.idx += deltaIdx;

        // clamp to the array indices
        if (view.idx < 0) {
            view.idx = 0
        }

        if (view.idx >= Zoom.LEVELS.length) {
            view.idx = Zoom.LEVELS.length - 1
        }

        return Zoom.LEVELS[view.idx];
    }

    Zoom.prototype.apply = function(options) {
        let startZoom = options['start'] || this.view.fov;
        let finalZoom = options['stop'] || undefined;
        let interpolationDuration = options['duration'] || 1000; // default to 1seconds
        if (!finalZoom)
            return;

        this.finalZoom = finalZoom;

        if (!this.isZooming) {
            this.isZooming = true;

            this.startTime = performance.now();

            this.x1 = 0
            this.x2 = 1;
            this.y1 = startZoom;
            this.y2 = finalZoom;
            this.m1 = finalZoom - startZoom;
            this.m2 = 0;

            this.x = this.x1;
        } else {
            // find the startTime
            this.x = (performance.now() - this.startTime) / interpolationDuration;

            let m1 = Zoom.hermiteCubic.fPrime(this.x, this.x1, this.x2, this.y1, this.y2, this.m1, this.m2)
            let y1 = Zoom.hermiteCubic.f(this.x, this.x1, this.x2, this.y1, this.y2, this.m1, this.m2);
            this.y1 = y1;
            this.x1 = this.x;
            this.x2 = this.x1 + 1;
            this.y2 = finalZoom;
            this.m1 = m1;
            this.m2 = 0;
        }

        // Initialize current zoom to the current zoom level
        let interpolatedZoom;
        let self = this;
        // Recursive function to perform interpolation for each frame
        function interpolateFrame() {
            //console.log('zooming')
            //fps = 1000 / self.dt;
            //totalFrames = interpolationDuration * fps; // Total number of frames
            self.x = ( performance.now() - self.startTime ) / interpolationDuration;
            // Calculate step size for each frame
            //stepSize = (desiredZoom - currentZoom) / totalFrames;
            interpolatedZoom = Zoom.hermiteCubic.f(self.x, self.x1, self.x2, self.y1, self.y2, self.m1, self.m2);
            // Clamp the interpolation in case it is < 0 for a time
            if (interpolatedZoom < Zoom.min()) {
                interpolatedZoom = Zoom.min();
            }

            // Apply zoom level to map or perform any necessary rendering
            self.view.setZoom(interpolatedZoom);

            self.fov = interpolatedZoom;
    
            // Check if interpolation is complete
            if (self.x >= self.x2 || Math.abs(interpolatedZoom - self.finalZoom) < 1e-4) {
                self.view.setZoom(self.finalZoom);

                self.isZooming = false;
            } else {
                // Request the next frame
                requestAnimFrame(interpolateFrame);
            }
        }
    
        // Start interpolation by requesting the first frame
        requestAnimFrame(interpolateFrame);
    }

    Zoom.max = function() {
        return Zoom.LEVELS[0];
    }

    Zoom.min = function() {
        return Zoom.LEVELS[Zoom.LEVELS.length - 1];
    }

    Zoom.hermiteCubic = {
        f: function(x, x1, x2, y1, y2, m1, m2) {
            let t = (x - x1) / (x2 - x1)
            let t2 = t*t;
            let t3 = t2*t;
            return (1 - 3*t2 + 2*t3) * y1 + (t - 2*t2 + t3) * m1 + (3*t2 - 2*t3) * y2 + (-t2 + t3) * m2
        },
        fPrime: function(x, x1, x2, y1, y2, m1, m2) {
            let t = (x - x1) / (x2 - x1)
            let t2 = t*t;
            return (1 / (x2 - x1))*((-6*t+6*t2)*y1 + (1 - 4*t + 3*t2)*m1 + (6*t - 6*t2)*y2 + m2*(3*t2 - 2*t))
        }
    }

	return Zoom;
})();
