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
import { requestAnimFrame } from "./libs/RequestAnimationFrame.js";

 export let Zoom = (function() {
    // constructor
	function Zoom(view) {
        this.view = view;
	};

    Zoom.prototype.apply = function(options) {
        let startZoom = options['start'] || this.view.fov;
        let finalZoom = options['stop'] || undefined;
        let interpolationDuration = options['duration'] || 1000; // default to 1seconds
        if (!finalZoom)
            return;

        // clamp the zoom to the view params minFov and maxFov and the projection bounds
        finalZoom = Math.min(finalZoom, this.view.projection.fov);

        // then clamp the fov between minFov and maxFov
        const minFoV = this.view.minFoV;
        const maxFoV = this.view.maxFoV;

        if (minFoV) {
            finalZoom = Math.max(finalZoom, minFoV);
        }

        if (maxFoV) {
            finalZoom = Math.min(finalZoom, maxFoV);
        }

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

            // we must delete the current request anim frame
            window.cancelAnimationFrame(this.requestAnimID)
        }

        // Initialize current zoom to the current zoom level
        let interpolatedZoom;
        let self = this;
        // Recursive function to perform interpolation for each frame
        function interpolateFrame() {
            //fps = 1000 / self.dt;
            //totalFrames = interpolationDuration * fps; // Total number of frames
            self.x = ( performance.now() - self.startTime ) / interpolationDuration;
            // Calculate step size for each frame
            //stepSize = (desiredZoom - currentZoom) / totalFrames;
            interpolatedZoom = Zoom.hermiteCubic.f(self.x, self.x1, self.x2, self.y1, self.y2, self.m1, self.m2);
            // Clamp the interpolation in case it is < 0 for a time
            interpolatedZoom = Math.max(0, interpolatedZoom);

            // Apply zoom level to map or perform any necessary rendering
            self.view.setZoom(interpolatedZoom);

            self.fov = interpolatedZoom;
    
            // Check if interpolation is complete
            if (self.stop) {
                self.isZooming = false;
                self.stop = false;
            } else if (self.x >= self.x2 || Math.abs(interpolatedZoom - self.finalZoom) < 1e-4) {
                self.view.setZoom(self.finalZoom);

                self.isZooming = false;
            } else {
                // Request the next frame
                self.requestAnimID = requestAnimFrame(interpolateFrame);
            }
        }
    
        // Start interpolation by requesting the first frame
        self.requestAnimID = requestAnimFrame(interpolateFrame);
    }

    Zoom.prototype.stopAnimation = function() {
        if (this.isZooming) {
            this.stop = true;
        }
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
