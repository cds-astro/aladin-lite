// Copyright 2023 - UDS/CNRS
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
 * File gui/ContextMenu.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import { Coo } from '../libs/astro/coo.js';
import { CooFrameEnum } from '../CooFrameEnum.js';
import { Utils } from '../Utils';
import { ActionButton } from './widgets/ActionButton.js';
import targetIconImg from '../../../assets/icons/target.svg';

export class SODAQueryWindow {
    constructor(aladin) {
        this.aladin = aladin;
        this.isShowing = false;
    }

    _attachParam(target, input) {
        if (input.type === "text" || input.type === "number") {
            let inputEl = document.createElement('input');
            inputEl.type = input.type;
            inputEl.classList.add('aladin-input');
            if (input.type === "number") {
                inputEl.step = "any";
            }
            inputEl.value = input.value;
            inputEl.name = input.name;

            let labelEl = document.createElement('label');
            labelEl.textContent = input.name;

            if (input.utype) {
                labelEl.textContent = labelEl.textContent + "[" + input.utype + "]";
            }
            labelEl.for = input.id;

            let divEl = document.createElement("div");
            divEl.classList.add(labelEl.textContent, "aladin-form-input");

            divEl.appendChild(labelEl);
            divEl.appendChild(inputEl);

            target.appendChild(divEl);
        } else if (input.type === "group") {
            let groupEl = document.createElement('div');
            groupEl.classList.add(input.name, "aladin-form-input-group");
            groupEl.innerHTML = '<div class="aladin-form-group-header">' + input.name + '</div>';

            if (input.name === 'CIRCLE') {
                let self = this;
                new ActionButton(groupEl.querySelector(".aladin-form-group-header"), {
                    iconURL: targetIconImg,
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    info: 'Circular selection\n<i><font size="-2">Click, drag and release to define the circle</font></i>',
                    action(e) {
                        self.aladin.select('circle', (s) => {
                            const {x, y, r} = s;
    
                            const [ra, dec] = self.aladin.pix2world(x, y);
                            const dist = self.aladin.angularDist(x, y, x + r, y);
                            // find the children
                            let [raInputEl, decInputEl, radiusInputEl] = groupEl.querySelectorAll(".aladin-form-input input");
    
                            raInputEl.value = ra;
                            decInputEl.value = dec;
                            radiusInputEl.value = dist;
                        });
                    }
                });
            }

            input.value.forEach((subInput) => this._attachParam(groupEl, subInput));

            target.appendChild(groupEl);
        }
    }

    hide() {
        if(this.mainEl) {
            this.mainEl.remove();
        }

        this.isShowing = false;
    }

    show(callbackValid) {
        this.isShowing = true;

        this.formEl = document.createElement('form');
        this.formEl.className = "aladin-form";
        // Add the form inputs
        this.formParams["inputParams"].forEach((param) => this._attachParam(this.formEl, param));

        let submitFormDiv = document.createElement('div');
        submitFormDiv.className = 'submit';
        submitFormDiv.innerHTML = '<button class="aladin-btn aladin-validBtn" type="submit">Submit</button>' + 
        '   <button class="aladin-btn aladin-cancelBtn">Cancel</button>';
        this.formEl.appendChild(submitFormDiv);

        this.mainEl = document.createElement('div');
        this.mainEl.classList.add('aladin-box', 'aladin-anchor-left');
        this.mainEl.style.display = 'initial';

        this.mainEl.innerHTML = '<a class="aladin-closeBtn">&times;</a><div class="aladin-horizontal-list"></div>';

        const listOfInputParams = this.formParams["inputParams"].map((param) => param.name).join(', ');


        let aladinTitleListEl = this.mainEl.querySelector('div');
        new ActionButton(aladinTitleListEl, {
            content: '📡',
            backgroundColor: 'white',
            borderColor: '#484848',
            info: 'This is the form to request the SODA server located at: <a target="_blank" href="' + this.formParams["baseUrl"]  + '">' + this.formParams["baseUrl"] + '</a>\nThe list of input params is:\n' + listOfInputParams,
            action(e) {}
        });

        let titleEl = document.createElement('div');
        titleEl.textContent = 'Cutouts query form';
        titleEl.classList.add('aladin-box-title');

        aladinTitleListEl.appendChild(titleEl)

        this.mainEl.appendChild(this.formEl);

        this.aladin.aladinDiv.appendChild(this.mainEl);
        this.mainEl.querySelector(".aladin-closeBtn")
            .addEventListener(
                "click",
                () => { this.hide(); }
            );
        this.mainEl.querySelector(".submit .aladin-cancelBtn")
            .addEventListener(
                "click",
                () => { this.hide(); }
            );

        this.mainEl.querySelector(".submit .aladin-validBtn")
            .addEventListener("click", (e) => {
                e.preventDefault();
                let params = [];

                for (let child of this.formEl.children) {
                    let param;
                    if (child.classList.contains("aladin-form-input")) {
                        // get the input
                        let input = child.querySelector("input");
                        param = {
                            name: input.name,
                            value: input.value
                        };
                    } else if (child.classList.contains("aladin-form-input-group")) {
                        let values = [];
                        for (let formParam of child.children) {
                            if (formParam.classList.contains("aladin-form-input")) {
                                // get the input
                                let input = formParam.querySelector("input");
                                values.push(input.value);
                            }
                        }

                        param = {
                            name: child.classList[0],
                            value: values
                        };
                    }

                    if (param) {
                        params.push(param);
                    }
                }

                if (callbackValid) {
                    callbackValid(this.formParams["baseUrl"], params);
                }
            });
    }

    setParams(params) {
        this.formParams = params;
    }


}








