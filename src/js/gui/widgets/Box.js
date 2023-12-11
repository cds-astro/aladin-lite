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

import { DOMElement } from "./Widget";
import { Utils } from "../Utils";
import { ActionButton } from "./ActionButton";
import moveIconImg from '../../../../assets/icons/move.svg';
import { Layout } from "../Layout";
import { Form } from "./Form";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Tab.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

/* Example of layout
[{
    content: ''
    title: '',
    color: <label color>,
    backgroundColor: <background tab color>,
    action: () => {}
},]
*/
export class Box extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement("div");
        el.classList.add('aladin-box');
        el.style.display = "initial";

        super(el, options);
        // add it to the dom
        this.attachTo(target, position);
    }

    _hide() {
        super._hide()
    }

    _show() {
        this.el.innerHTML = "";
        super._show();

        let self = this;

        // Check for the title
        if (this.options.header) {
            let header = this.options.header;
            let titleEl;
            if (header.title) {
                titleEl = document.createElement('div')
                titleEl.classList.add("aladin-box-title");

                DOMElement.appendTo(header.title, titleEl);
            }
    
            let draggableEl;
            if (header.draggable) {
                draggableEl = ActionButton.createIconBtn({
                    iconURL: moveIconImg,
                    tooltip: {content: 'Drag the window to move it',  position: {direction: 'bottom'}},
                    cssStyle: {
                        backgroundColor: '#bababa',
                        borderColor: '#484848',
                        cursor: 'move',
                        width: '18px',
                        height: '18px',
                    },
                    action(e) {}
                });
    
                dragElement(draggableEl.element(), this.el)
    
                // Heavily inspired from https://www.w3schools.com/howto/howto_js_draggable.asp
                function dragElement(triggerElt, elmnt) {
                    var pos1 = 0, pos2 = 0, pos3 = 0, pos4 = 0;
                    // otherwise, move the DIV from anywhere inside the DIV:
    
                    triggerElt.onmousedown = dragMouseDown;
                  
                    function dragMouseDown(e) {
                        e = e || window.event;
                        e.preventDefault();
                        // get the mouse cursor position at startup:
                        pos3 = e.clientX;
                        pos4 = e.clientY;
                        document.onmouseup = closeDragElement;
                        // call a function whenever the cursor moves:
                        document.onmousemove = elementDrag;
                    }
                  
                    function elementDrag(e) {
                        e = e || window.event;
                        e.preventDefault();
                        // calculate the new cursor position:
                        pos1 = pos3 - e.clientX;
                        pos2 = pos4 - e.clientY;
                        pos3 = e.clientX;
                        pos4 = e.clientY;
                        // set the element's new position:
                        elmnt.style.top = (elmnt.offsetTop - pos2) + "px";
                        elmnt.style.left = (elmnt.offsetLeft - pos1) + "px";
                    }
                  
                    function closeDragElement() {
                        // stop moving when mouse button is released:
                        document.onmouseup = null;
                        document.onmousemove = null;
                    }
                }
            }
    
            let closedEl = ActionButton.createIconBtn({
                content: '❌',
                tooltip: {content: 'Close the window', position: {direction: 'bottom'}},
                cssStyle: {
                    backgroundColor: '#bababa',
                    borderColor: '#484848',
                    color: 'red',
                    cursor: 'pointer',
                    textAlign: 'center',
                    fontSize: '12px',
                    fontWeight: 'bold',
                    lineHeight: '0px', 
                    width: '18px',
                    height: '18px',
                },
                action(e) {
                    self._hide();
                }
            });
    
            Layout.horizontal({
                cssStyle: {
                    justifyContent: 'space-between',
                },
                layout: [draggableEl, titleEl, closedEl]
            }, this.el);

            let separatorEl = document.createElement('div')
            separatorEl.classList.add("aladin-box-separator");
            this.el.appendChild(separatorEl);
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.content) {
            this.appendContent(this.options.content);
        }

        if (this.options.position) {
            this.setPosition(this.options.position)
        }
    }
}

export class SelectBox extends Box {
    /**
     * Create a Tabs layout
     * @param {box options, possibilities: Array.<{label: String, content: DOMElement}>} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, target, position = "beforeend") {
        let labels = options.possibilities.map((opt) => opt.label);
        
        let value = (options && options.selected) || labels[0];

        let self;
        let settingsSelector = new Form({
            label: "Settings",
            name: 'param',
            type: 'select',
            value: value,
            options: labels,
            actions: {
                'change': (e) => {
                    let labelSelected = e.target.value;

                    let content;
                    for (let opt of options.possibilities) {
                        if (opt.label === labelSelected) {
                            content = opt.content;
                            break;
                        }
                    }

                    self.update({
                        header: options && options.header,
                        content: Layout.vertical({layout: [settingsSelector, content]})
                    })
                }
            }
        })

        let selectedContent = options.possibilities.find((p) => p.label === value);
        options['content'] = Layout.vertical({
            layout: [settingsSelector, selectedContent.content]
        });

        super(options, target, position);
        self = this;
    }
}
