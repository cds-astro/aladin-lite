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
import { DOMElement } from "./Widget";
import { ActionButton } from "./ActionButton";
import enlargeIconImg from '../../../../assets/icons/enlarge.svg';
import moveIconImg from '../../../../assets/icons/move.svg';
import { Layout } from "../Layout";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Widgets/Box.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/
export class Box extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement("div");
        el.classList.add('aladin-box');

        super(el, options);

        this.attachTo(target, position);
        this._show();
    }

    close() {
        this._hide()
    }

    _show(options) {
        this.options = {
            ...this.options,
            ...options
        };

        this.el.innerHTML = "";

        let close = this.options.close === false ? false : true;
        let draggable = false;
        if (close) {
            this.el.appendChild(
                ActionButton.BUTTONS(null).close(this).element()
            );
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

                DOMElement.appendTo(new Layout(header.title), titleEl);
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
    
            let headerEl = Layout.horizontal([draggableEl, titleEl], {}, this.el);
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
            if (content instanceof Layout) {
                this.appendContent(content);
            } else {
                this.appendContent(Layout.vertical(content));
            }

            this.el.lastChild.classList.add("aladin-box-content");
        }

        if (this.options.sizeable) {
            let sizeableBtn = new ActionButton({
                icon: {
                    url: enlargeIconImg,
                    size: "small",
                    monochrome: true,
                },
                tooltip: {content: 'Enlarge the window',  global: true, aladin: this.aladin},
                cssStyle: {
                    cursor: 'move',
                    position: 'absolute',
                    bottom: 0,
                    right: 0
                },
            });
            this.appendContent(sizeableBtn);

            enlargeElement(sizeableBtn.element(), this.el);
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
    var t, l;
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
        t = elmnt.offsetTop - pos2
        l = elmnt.offsetLeft - pos1
        elmnt.style.top = t + "px";
        elmnt.style.left = l + "px";
    }
  
    function closeDragElement() {
        // stop moving when mouse button is released:
        document.onmouseup = null;
        document.onmousemove = null;

        /*var r = elmnt.getBoundingClientRect();

        if (t < r.height / 2) {
            elmnt.style.top = r.height / 2 + "px";
        }

        if (l < r.width / 2) {
            elmnt.style.left = r.width / 2 + "px";
        }

        const aladinDiv = elmnt.closest('.aladin-container');
        
        if (l + r.width / 2 > aladinDiv.offsetWidth) {
            elmnt.style.left = (aladinDiv.offsetWidth - r.width / 2) + "px";
        }

        if (t + r.height / 2 > aladinDiv.offsetHeight) {
            elmnt.style.top = (aladinDiv.offsetHeight - r.height / 2) + "px";
        }*/
    }
}

function enlargeElement(triggerElt, elmnt) {
    let pos3 = 0, pos4 = 0;

    triggerElt.onmousedown = dragMouseDown;

    function dragMouseDown(e) {
        e.preventDefault();
        pos3 = e.clientX;
        pos4 = e.clientY;
        document.onmouseup = closeDragElement;
        document.onmousemove = elementDrag;
    }

    function elementDrag(e) {
        e.preventDefault();

        const dx = e.clientX - pos3;
        const dy = e.clientY - pos4;

        pos3 = e.clientX;
        pos4 = e.clientY;

        const newWidth  = elmnt.offsetWidth  + 2*dx;
        const newHeight = elmnt.offsetHeight + 2*dy;

        elmnt.style.width  = Math.max(20, newWidth) + "px";
        elmnt.style.height = Math.max(20, newHeight) + "px";
    }

    function closeDragElement() {
        document.onmouseup = null;
        document.onmousemove = null;
    }
}
