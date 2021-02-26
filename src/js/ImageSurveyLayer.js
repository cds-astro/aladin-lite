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
 * File ImageSurveyLayer
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
import { Utils } from "./Utils.js";
import { HpxImageSurvey } from "./HpxImageSurvey.js";

export let ImageSurveyLayer = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    /** Constructor
     *  
     */
    let ImageSurveyLayer = function(name) {
        this.surveys = new Map();
        this.name = name;
    }

    ImageSurveyLayer.prototype.addImageSurvey = async function(rootUrlOrId) {
        const survey = await HpxImageSurvey.create(rootUrlOrId);
        this.surveys.set(rootUrlOrId, survey);
    };

    ImageSurveyLayer.prototype.removeImageSurvey = function(rootUrlOrId) {
        this.surveys.remove(rootUrlOrId);
    };

    ImageSurveyLayer.prototype.clear = function() {
        this.surveys.clear();
    };

    ImageSurveyLayer.prototype.getSurveys = function() {
        return this.surveys.values();
    };

    return ImageSurveyLayer;
})();