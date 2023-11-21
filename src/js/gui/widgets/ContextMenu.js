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

import { Coo } from '../../libs/astro/coo.js';
import { CooFrameEnum } from '../../CooFrameEnum.js';
import { Utils } from '../../Utils';
import { DOMElement } from './Widget.js';

export class ContextMenu extends DOMElement {

    constructor(aladin, options) {
        let el = document.createElement('ul');
        el.className = 'aladin-context-menu';

        super(el);

        this.aladin = aladin;

        this.cssStyleDefault = el.style;

        if (!options || options.hideOnClick === undefined || options.hideOnClick === true) {
            document.addEventListener('click', () => this._hide());
        }

        if (!options || options.hideOnResize === undefined || options.hideOnResize === true) {
            window.addEventListener('resize', () => this._hide());
        }
    }

    _hide() {
        super._hide()
        super.remove()
    }

    _attachOption(target, opt, xymouse) {
        const item = document.createElement('li');
        item.classList.add('aladin-context-menu-item');

        if (opt.cssStyle) {
            // add the css style to the item
            // copied from widgets.js
            for (const property in opt.cssStyle) {
                item.style[property] = opt.cssStyle[property];
            }
        }

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
            if (opt.label instanceof DOMElement) {
                // And add it to the DOM
                opt.label.attachTo(item);
            } else if (opt.label instanceof Element) {                
                item.insertAdjacentElement('beforeend', opt.label);
            } else {
                let wrapEl = document.createElement('div');
                wrapEl.innerHTML = opt.label;
                item.insertAdjacentElement('beforeend', wrapEl);
            }
        }

        if (opt.subMenu && opt.subMenu.length > 0) {
            item.innerHTML += '<span style="position: absolute; right: 0px">▶</span>';
        }

        const self = this;
        if (opt.disabled && opt.disabled === true) {
            item.classList.add('aladin-context-menu-item-disabled');
        }

        if (opt.selected && opt.selected === true) {
            item.classList.add('aladin-context-menu-item-selected');
        }

        if (opt.action) {
            item.addEventListener('click', e => {
                e.stopPropagation();
    
                if (!opt.disabled || opt.disabled === false) {
                    if (!opt.subMenu || opt.subMenu.length === 0) {
                        opt.action(e);
                        self._hide();
                    }
                }
            });
        }
        
        if (opt.hover) {
            item.addEventListener('mouseover', e => {
                e.stopPropagation();
                opt.hover(e, item);
            })
        }
        if (opt.unhover) {
            item.addEventListener('mouseout', e => {
                e.stopPropagation();
                opt.unhover(e, item);
            })
        }
        
        target.appendChild(item);

        if (opt.subMenu && opt.subMenu.length) {
            const subMenu = document.createElement('ul');
            subMenu.className = 'aladin-context-sub-menu';
            item.appendChild(subMenu);
            opt.subMenu.forEach(subOpt => this._attachOption(subMenu, subOpt));
        }
    }

    _subMenuDisplay(parent) {
        for (let item of parent.children) {
            // Display the submenu to evaluate its size
            item.style.display = "block";

            if (item.className === 'aladin-context-sub-menu') {
                let r = item.getBoundingClientRect();

                if (r.x + r.width >= innerWidth) {
                    this.el.classList.add('left');
                }

                if (r.y + r.height >= innerHeight) {
                    this.el.classList.add('top');
                }
            }

            this._subMenuDisplay(item)

            // Hide the submenu
            item.style.display = "";
        }
    }

    show(options) {
        this.remove();

        this.el.innerHTML = '';
        this.el.style = this.cssStyleDefault
        let xymouse;
        if (options && options.e) {
            xymouse = Utils.relMouseCoords(options.e);
        }

        this.menuOptions.forEach(opt => this._attachOption(this.el, opt, xymouse));

        // Add it to the dom
        this.attachTo(this.aladin.aladinDiv)

        if (options && options.cssStyle) {
            this.setCss(options.cssStyle);
        }

        // Set position
        const position = options && options.position || (options && options.e && { left: options.e.clientX, top: options.e.clientY });
        this.setPosition(position)

        this.el.classList.remove('left')
        this.el.classList.remove('top')
        this._subMenuDisplay(this.el)

        super._show()
    }

    attach(options) {
        this.menuOptions = options;
    }

    static menu = undefined;

    static getInstance(aladin, options) {
        if (!ContextMenu.menu) {
            ContextMenu.menu = new ContextMenu(aladin, options);
        }

        return ContextMenu.menu;
    }
}








