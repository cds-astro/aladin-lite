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
 * File Ellipse
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { Utils } from "./Utils.js";
import { AladinUtils } from "./AladinUtils.js";
import { CooFrameEnum } from "./CooFrameEnum.js";
import { Aladin } from "./Aladin.js";

// TODO : Ellipse, Circle and Footprint should inherit from the same root object
export let Ellipse = (function() {
    // constructor
    let Ellipse = function(centerRaDec, rayonXDegrees, rayonYDegrees, rotationDegrees, options) {
        options = options || {};

        this.color = options['color'] || undefined;

        // TODO : all graphic overlays should have an id
        this.id = 'ellipse-' + Utils.uuidv4();

        this.setCenter(centerRaDec);
        this.setRadiuses(rayonXDegrees, rayonYDegrees);
        this.setRotation(rotationDegrees);
    	this.overlay = null;
    	
    	this.isShowing = true;
        this.isSelected = false;
    };

    Ellipse.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };
    
    Ellipse.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Ellipse.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Ellipse.prototype.dispatchClickEvent = function() {
        if (this.overlay) {
            // footprint selection code adapted from Fabrizio Giordano dev. from Serco for ESA/ESDC
            //window.dispatchEvent(new CustomEvent("footprintClicked", {
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
        }
    };
    
    Ellipse.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
/*
            this.overlay.view.aladinDiv.dispatchEvent(new CustomEvent("footprintClicked", {
                detail: {
                    footprintId: this.id,
                    overlayName: this.overlay.name
                }
            }));
*/

            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };


    
    Ellipse.prototype.setCenter = function(centerRaDec) {
        this.centerRaDec = centerRaDec;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setRotation = function(rotationDegrees) {
        // radians
        let theta = rotationDegrees * Math.PI / 180;
        this.rotation = theta;
        // rotation in clockwise in the 2d canvas
        // we must transform it so that it is a north to east rotation
        //this.rotation = -theta - Math.PI/2;

        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    Ellipse.prototype.setRadiuses = function(radiusXDegrees, radiusYDegrees) {
        this.radiusXDegrees = radiusXDegrees;
        this.radiusYDegrees = radiusYDegrees;

        if (this.overlay) {
            this.overlay.reportChange();
        }
    };

    // TODO
    Ellipse.prototype.draw = function(ctx, view, frame, width, height, largestDim, zoomFactor, noStroke) {
        if (! this.isShowing) {
            return;
        }
        noStroke = noStroke===true || false;

        var centerXyview = AladinUtils.radecToViewXy(this.centerRaDec[0], this.centerRaDec[1], view);
        if (!centerXyview) {
            // the center goes out of the projection
            // we do not draw it
            return;
        }

        let circlePtXyViewRa = AladinUtils.radecToViewXy(this.centerRaDec[0] + this.radiusXDegrees, this.centerRaDec[1], view);
        let circlePtXyViewDec = AladinUtils.radecToViewXy(this.centerRaDec[0], this.centerRaDec[1] + this.radiusYDegrees, view);

        if (!circlePtXyViewRa || !circlePtXyViewDec) {
            // the circle border goes out of the projection
            // we do not draw it
            return;
        }

        var dxRa = circlePtXyViewRa[0] - centerXyview[0];
        var dyRa = circlePtXyViewRa[1] - centerXyview[1];
        var radiusInPixX = Math.sqrt(dxRa*dxRa + dyRa*dyRa);

        var dxDec = circlePtXyViewDec[0] - centerXyview[0];
        var dyDec = circlePtXyViewDec[1] - centerXyview[1];
        var radiusInPixY = Math.sqrt(dxDec*dxDec + dyDec*dyDec);

        // Ellipse crossing the projection
        if ((dxRa*dyDec - dxDec*dyRa) <= 0.0) {
            // We do not draw it
            return;
        }
        // TODO : check each 4 point until show
        var baseColor = this.color;
        if (! baseColor && this.overlay) {
            baseColor = this.overlay.color;
        }
        if (! baseColor) {
            baseColor = '#ff0000';
        }
        
        if (this.isSelected) {
            ctx.strokeStyle= Overlay.increaseBrightness(baseColor, 50);
        }
        else {
            ctx.strokeStyle= baseColor;
        }

        // 1. Find the spherical tangent vector going to the north
        let origin = this.centerRaDec;
        let toNorth = [this.centerRaDec[0], this.centerRaDec[1] + 1e-3];

        // 2. Project it to the screen
        let originScreen = this.overlay.view.wasm.worldToScreen(origin[0], origin[1]);
        let toNorthScreen = this.overlay.view.wasm.worldToScreen(toNorth[0], toNorth[1]);

        // 3. normalize this vector
        let toNorthVec = [toNorthScreen[0] - originScreen[0], toNorthScreen[1] - originScreen[1]];
        let norm = Math.sqrt(toNorthVec[0]*toNorthVec[0] + toNorthVec[1]*toNorthVec[1]);
        
        toNorthVec = [toNorthVec[0] / norm, toNorthVec[1] / norm];
        let toWestVec = [1.0, 0.0];

        let x1 = toWestVec[0];
        let y1 = toWestVec[1];
        let x2 = toNorthVec[0];
        let y2 = toNorthVec[1];
        // 4. Compute the west to north angle
        let westToNorthAngle = Math.atan2(x1*y2-y1*x2, x1*x2+y1*y2);

        // 5. Get the correct ellipse angle
        let theta = -this.rotation + westToNorthAngle;

        ctx.beginPath();
        ctx.ellipse(centerXyview[0], centerXyview[1], radiusInPixX, radiusInPixY, theta, 0, 2*Math.PI, false);
        if (!noStroke) {
            ctx.stroke();
        }
    }; 
    
    return Ellipse;
})();
