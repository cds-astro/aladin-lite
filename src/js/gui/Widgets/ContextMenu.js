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
import { Layout } from '../Layout.js';
import { Icon } from './Icon.js';

import uploadIconUrl from '../../../../assets/icons/upload.svg';
import copyIconUrl from '../../../../assets/icons/copy.svg';

import nextIconSvg from '../../../../assets/icons/next.svg';

export class ContextMenu extends DOMElement {
    static _menus = [];

    constructor(aladin, options) {
        let el = document.createElement('ul');
        el.className = 'aladin-context-menu';

        super(el, options);

        this.addClass('aladin-dark-theme')

        this.aladin = aladin;

        this.cssStyleDefault = el.style;

        if (!options || options.hideOnClick === undefined || options.hideOnClick === true || typeof options.hideOnClick === 'function') {
            this.aladin.aladinDiv.addEventListener('click', (e) => {
                if (!el.contains(e.target)) {
                    if (options && options.hideOnClick && typeof options.hideOnClick === 'function') {
                        options.hideOnClick(e)
                    } else {
                        this._hide()
                    }
                }
            });
        }

        if (!options || options.hideOnResize === undefined || options.hideOnResize === true) {
            if (Utils.hasTouchScreen()) {
                if (screen && 'orientation' in screen) {
                    screen.orientation.addEventListener("change", (e) => {
                        this._hide()
                    });
                } else {
                    window.addEventListener('orientationchange', (e) => {
                        this._hide()
                    })
                }
            } else {
                new ResizeObserver(() => { 
                    this._hide()
                })
                .observe(this.aladin.aladinDiv)
            }
        }

        ContextMenu._menus.push(this);
    }

    //static lastHoveredItem;

    _attachOption(target, opt, e, cssStyle) {
        let item = document.createElement('li');
        item.classList.add('aladin-context-menu-item');

        if (opt.label == 'Copy position') {
            try {
                // erase the label
                item.innerHTML = '';

                // compute the position string
                const xymouse = Utils.relMouseCoords(e);
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

                // construct the new item
                Layout.horizontal([
                    new Icon({
                        monochrome: true,
                        url: copyIconUrl,
                        size: 'small',
                        cssStyle: {
                            cursor: 'not-allowed',
                        }
                    }),
                    posStr
                ]).attachTo(item)
            } catch (e) {
                item.innerHTML = '<span>Out of projection</span>';
            }
        } else {
            if (opt.label instanceof DOMElement) {
                // And add it to the DOM
                opt.label.attachTo(item);
            } else if (opt.label instanceof Element) {                
                item.insertAdjacentElement('beforeend', opt.label);
            } else if (opt.label instanceof Object) {
                let layout = [];

                if (opt.label.icon) {
                    // add a button with a little bit of margin
                    opt.label.icon.size = opt.label.icon.size || 'small';

                    let icon = new Icon(opt.label.icon);
                    layout.push(icon)
                }

                if (opt.label.content) {
                    if (!Array.isArray(opt.label.content)) {
                        opt.label.content = [opt.label.content]
                    }

                    opt.label.content.forEach(l => layout.push(l))
                }

                for (let l of layout) {
                    let el = l;
                    if (l instanceof DOMElement) {
                        el = l.element()
                    }

                    if (el.style) {
                        el.style.marginRight = '5px';
                    }
                }

                let tooltip;
                if (opt.disabled && opt.disabled.reason) {
                    tooltip = {
                        content: opt.disabled.reason,
                        position: {direction: 'top'}
                    }
                } else if (opt.label.tooltip) {
                    tooltip = opt.label.tooltip
                }

                let labelEl = Layout.horizontal({layout, tooltip});
                labelEl.attachTo(item)
            } else if (opt.disabled && opt.disabled.reason) {
                let tooltip = {
                    content: opt.disabled.reason,
                    position: {direction: 'top'}
                }

                let labelEl = Layout.horizontal({layout: opt.label, tooltip});
                labelEl.attachTo(item)
            } else {
                let wrapEl = document.createElement('div');
                wrapEl.innerHTML = opt.label;
                item.insertAdjacentElement('beforeend', wrapEl);
            }
        }

        if (opt.cssStyle) {
            // add the css style to the item
            // copied from widgets.js
            for (const property in opt.cssStyle) {
                item.style[property] = opt.cssStyle[property];
            }
        }

        if (opt.subMenu && opt.subMenu.length > 0) {
            let iconElt = new Icon({url: nextIconSvg, size: 'small', monochrome: true}).element();
            item.appendChild(iconElt);
            item.style.display = 'flex';
            item.style.alignItems = 'center';
            item.style.justifyContent = 'space-between';
        }

        const self = this;
        if (opt.disabled && opt.disabled !== false) {
            item.classList.add('aladin-context-menu-item-disabled');
        }

        if (opt.selected && opt.selected === true) {
            item.classList.add('aladin-context-menu-item-selected');
        }

        if (opt.subMenu) {
            // User
            item.addEventListener('click', e => {
                e.stopPropagation();

                /* Add the ability to deploy the menu/sub-menus for touch screen devices */
                if (Utils.hasTouchScreen()) {
                    let subMenu = item.querySelector(".aladin-context-sub-menu")
                    let subMenuIsShown = subMenu.style.display === "block";
    
                    if (item.parentNode) {
                        let subMenus = item.parentNode.querySelectorAll(".aladin-context-sub-menu")
                        for (let subMenuChild of subMenus) {
                            subMenuChild.style.display = 'none';
                        }
                    }
    
                    if (!subMenuIsShown) {
                        subMenu.style.setProperty('display', 'block');
                    }
                }

                if (opt.action && (!opt.disabled || opt.disabled === false)) {
                    opt.action(e, self);
                }
            });
        } else if (opt.action) {
            item.addEventListener('click', e => {
                e.stopPropagation();

                if (!opt.disabled || opt.disabled === false) {
                    if (!opt.subMenu || opt.subMenu.length === 0) {
                        let close = opt.action(e, self);

                        close = close !== undefined ? close : true;

                        if (close && ((opt.mustHide === undefined || opt.mustHide === true) && (!self.options || self.options.hideOnClick === undefined || self.options.hideOnClick === true))) {
                            self._hide();
                        }
                    }
                }
            });
        }

        if (opt.subMenu && opt.subMenu.length) {
            const subMenu = document.createElement('ul');
            subMenu.className = 'aladin-context-sub-menu';
            // css is applied to the ul lonely
            if (cssStyle) {
                // add the css style to the item
                // copied from widgets.js
                for (const property in cssStyle) {
                    subMenu.style[property] = cssStyle[property];
                }
            }

            item.appendChild(subMenu);
            opt.subMenu.forEach(subOpt => this._attachOption(subMenu, subOpt, e, cssStyle));
        }

        const areSiblings = (elm1, elm2) => (elm1 !== elm2 && elm1.parentNode == elm2.parentNode);
        item.addEventListener('mouseover', e => {
            e.stopPropagation();
            e.preventDefault();

            if (opt.hover) {
                opt.hover(e, item);
            }

            /*if (ContextMenu.lastHoveredItem) {
                let parent = ContextMenu.lastHoveredItem.parentNode;
                if (parent && (areSiblings(parent, item) || item.contains(parent) || item === parent)) {
                    ContextMenu.lastHoveredItem.style.display = 'none';
                }
            }

            const subMenu = item.querySelector('.aladin-context-sub-menu');
            if (subMenu) {
                subMenu.style.display = 'block';
                ContextMenu.lastHoveredItem = subMenu;
            }*/
        })

        item.addEventListener('mouseout', e => {
            e.stopPropagation();
            e.preventDefault();

            if (opt.unhover) {
                opt.unhover(e, item);
            }
        })

        if (opt.classList) {
            item.classList.add(opt.classList)
        }

        target.appendChild(item);
    }

    _subMenuDisplay(parent, child) {
        parent.style.display = "block";
        child.style.display = "block";
    
        if (child.classList.contains('aladin-context-sub-menu')) {
            const aladinRect = this.aladin.aladinDiv.getBoundingClientRect();

            let o = parent.getBoundingClientRect();
            let c = child.getBoundingClientRect();

            child.classList.remove('left', 'right', 'top', 'bottom');

            // First check if there is place towards the right, which is the desired behaviour
            if (aladinRect.right - (o.x + o.width) >= c.width) {
                // do nothing as it is by default considering this case
            } else if (o.x - aladinRect.left >= c.width) {
                child.classList.add('left');
            } else if (aladinRect.bottom - (o.y + o.height) >= c.height) {
                child.classList.add('bottom');
            } else {
                child.classList.add('top');
            }

            /*if (r.y - aladinRect.top <= offsetHeight / 2.0) {
                topDir -= 1;
            } else {
                topDir += 1;
            }*/
            
        }

        for (let grandChild of child.children) {
            this._subMenuDisplay(child, grandChild);
        }

        child.style.display = "";
        parent.style.display = "";
    }

    show(options) {
        this.el.innerHTML = '';
        this.el.style = this.cssStyleDefault

        this.menuOptions.forEach((opt) => {
            this._attachOption(this.el, opt, options && options.e, options && options.cssStyle)
        });

        // Add it to the dom
        this.attachTo(this.aladin.aladinDiv)

        if (options && options.cssStyle) {
            this.setCss(options.cssStyle);
        }

        let mouseCoords = options && options.e && Utils.relMouseCoords(options.e)
        // Set position
        const position =
        options && options.position ||
        {left: mouseCoords.x, top: mouseCoords.y};

        this.setPosition({...position, aladin: this.aladin})

        for (let childEl of this.el.children) {
            this._subMenuDisplay(this.el, childEl);
        }

        super._show()
    }

    attach(options) {
        this.menuOptions = options;
    }

    /* Hide all the defined menus */
    static hideAll() {
        ContextMenu._menus.forEach((menu) => menu._hide())
    }

    /// Context menu predefined items
    static fileLoaderItem(itemOptions) {
        return {
            ...itemOptions,
            label: {
                icon: {
                    monochrome: true,
                    tooltip: {content: 'Load a local file from your computer.<br \>Accept ' + itemOptions.accept + ' files'},
                    url: uploadIconUrl,
                    cssStyle: {
                        cursor: 'help',
                    }
                },
                content: itemOptions.label
            },
            action(e) {
                let fileLoader = document.createElement('input');
                fileLoader.type = 'file';
                fileLoader.accept = itemOptions.accept || '*';
                // Case: The user is loading a FITS file
        
                fileLoader.addEventListener("change", (e) => {    
                    let file = e.target.files[0];
        
                    if (itemOptions.action) {
                        itemOptions.action(file)
                    }
                });
        
                fileLoader.click();
            }
        }
    }
}