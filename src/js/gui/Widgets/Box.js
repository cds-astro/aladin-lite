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
import { ActionButton } from "./ActionButton";
import moveIconImg from '../../../../assets/icons/move.svg';
import { Layout } from "../Layout";

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
        //el.style.display = "initial";

        super(el, options);
        this._show();

        this.addClass('aladin-dark-theme')

        this.attachTo(target, position);
    }

    _show(options) {
        this.options = {
            ...this.options,
            ...options
        };

        this.el.innerHTML = "";

        let self = this;

        let close = this.options.close === false ? false : true;
        let draggable = false;
        if (close) {
            new ActionButton({
                size: 'small',
                content: '❌',
                //tooltip: {content: 'Close the window', position: {direction: 'bottom'}},
                action(e) {
                    self._hide();
                },
                cssStyle: {
                    position: 'absolute',
                },
                position: {
                    top: 0,
                    right: 0,
                }
            }, this.el);
        }

        if (this.options.onDragged) {
            draggable = true;
        }

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
                draggable = true;
            }

            if (draggable) {
                draggableEl = new ActionButton({
                    icon: {
                        url: moveIconImg,
                        size: "small",
                        monochrome: true,
                    },
                    tooltip: {content: 'Drag the window to move it',  position: {direction: 'top'}},
                    cssStyle: {
                        cursor: 'move',
                    },
                    action(e) {}
                });
            }
    
            let headerEl = Layout.horizontal([draggableEl, titleEl], this.el);
            if (draggable) {
                dragElement(headerEl.element(), this.el, this.options.onDragged);
                headerEl.element().style.cursor = 'move';
            }

            let separatorEl = document.createElement('div')
            separatorEl.classList.add("aladin-box-separator");
            this.el.appendChild(separatorEl);
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        if (this.options.content) {
            let content = this.options.content
            //if (Array.isArray(content)) {
                this.appendContent(content);
            //} else {
            //    this.appendContent(content);
            //}
        }

        if (this.options.position) {
            this.setPosition(this.options.position)
        }

        if (this.options.classList) {
            this.addClass(this.options.classList)
        }

        super._show();
    }
}

// Heavily inspired from https://www.w3schools.com/howto/howto_js_draggable.asp
function dragElement(triggerElt, elmnt, onDragged) {
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

        if (onDragged) {
            onDragged();
        }
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
