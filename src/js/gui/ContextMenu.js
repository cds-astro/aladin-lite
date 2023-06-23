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

export class ContextMenu {

    constructor(aladin) {
        this.aladin = aladin;
        this.isShowing = false;
    }

    _hideMenu(e) {
        //if (e === true || !this.contextMenuUl.contains(e.target)) {
        this.contextMenuUl.remove();
        document.removeEventListener('click', this._hideMenu);
        window.removeEventListener('resize', this._hideOnResize);

        this.isShowing = false;
        //}
    }

    _hideOnResize() {
        this._hideMenu(true);
    }

    _attachOption(target, opt, xymouse) {
        const item = document.createElement('li');
        item.className = 'aladin-context-menu-item';
        if (opt.label == 'Copy position') {
            try {
                const pos = this.aladin.pix2world(xymouse.x, xymouse.y);
                const coo = new Coo(pos[0], pos[1], 6);
                let posStr;
                if (this.aladin.view.cooFrame == CooFrameEnum.J2000) {
                    posStr = coo.format('s/');
                } else if (this.aladin.view.cooFrame == CooFrameEnum.J2000d) {
                    posStr = coo.format('d/');
                } else {
                    posStr = coo.format('d/');
                }
                item.innerHTML = '<span>' + posStr + '</span>';
            } catch (e) {
                item.innerHTML = '<span></span>';
            }
        } else {
            item.innerHTML = '<span>' + opt.label + '</span>';
        }

        if (opt.subMenu && opt.subMenu.length > 0) {
            item.innerHTML += '<span style="position: absolute; right: 4px;">▶</span>';
        }

        const self = this;
        item.addEventListener('click', e => {
            e.stopPropagation();
            if (!opt.subMenu || opt.subMenu.length === 0) {
                if (opt.label == 'Copy position') {
                    opt.action(e);
                } else {
                    opt.action(this.event);
                }
                self._hideMenu(true);
            }
        });

        target.appendChild(item);

        if (opt.subMenu && opt.subMenu.length) {
            const subMenu = document.createElement('ul');
            subMenu.className = 'aladin-context-sub-menu';
            item.appendChild(subMenu);
            opt.subMenu.forEach(subOpt => this._attachOption(subMenu, subOpt));
        }
    }

    _showMenu(e) {


        this.contextMenuUl.className = 'aladin-context-menu';
        this.contextMenuUl.innerHTML = '';

        const xymouse = Utils.relMouseCoords(e);

        this.menuOptions.forEach(opt => this._attachOption(this.contextMenuUl, opt, xymouse));
        document.body.appendChild(this.contextMenuUl);

        const { innerWidth, innerHeight } = window;
        const { offsetWidth, offsetHeight } = this.contextMenuUl;
        let x = 0;
        let y = 0;

        this.event = e;


        if (e.clientX >= (innerWidth / 2)) {
            this.contextMenuUl.classList.add('left');
        }

        if (e.clientY >= (innerHeight / 2)) {
            this.contextMenuUl.classList.add('top');
        }

        if (e.clientX >= (innerWidth - offsetWidth)) {
            x = '-100%';
        }

        if (e.clientY >= (innerHeight - offsetHeight)) {
            y = '-100%';
        }

        this.contextMenuUl.style.left = e.clientX + 'px';
        this.contextMenuUl.style.top = e.clientY + 'px';
        this.contextMenuUl.style.transform = `translate(${x}, ${y})`;
        document.addEventListener('click', () => this._hideMenu(true));
        window.addEventListener('resize', this._hideOnResize);

        this.isShowing = true;
    }


    attachTo(el, options) {
        this.contextMenuUl = document.createElement('ul');
        this.menuOptions = options;

        const self = this;
        /*
        el.addEventListener('contextmenu', function (e) {
            e.preventDefault();
            self._showMenu(e, options, el);

            e.stopPropagation();
        });
        */

    }

}








