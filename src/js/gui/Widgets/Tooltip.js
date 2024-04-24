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
 * File gui/ActionButton.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/


import { DOMElement } from './Widget.js';
import { Utils } from '../../Utils';

/* Add a tooltip on a already added Element on the DOM */
export class Tooltip extends DOMElement {
    constructor(options, target) {
        // Creation of the DOM element
        let el = document.createElement('span');
        el.classList.add('aladin-tooltip');

        let targetParent = target.parentNode;

        // Insert it into the DOM tree
        let wrapperEl = document.createElement('div');
        wrapperEl.classList.add('aladin-tooltip-container');

        if (targetParent) {
            let targetIndex = Array.prototype.indexOf.call(targetParent.children, target);
            targetParent.removeChild(target);

            wrapperEl.appendChild(target);
            wrapperEl.appendChild(el);

            targetParent.insertChildAtIndex(wrapperEl, targetIndex);
        } else {
            wrapperEl.appendChild(target);
            wrapperEl.appendChild(el);
        }

        // Set the anchor to the element on which
        // the tooltip is set
        if (!options.position) {
            options.position = {
                direction: 'right',
            }
        }
        options.position.anchor = target;

        if (!options.delayShowUpTime) {
            options.delayShowUpTime = 500;
        }

        super(wrapperEl, options)

        this.element().classList.add('aladin-dark-theme')

        this._show();
    }

    setPosition(options) {
        // take on less priority the left and top
        if (options && options.left) {
            const left = options.left;
            this.element().style.position = 'absolute';
            this.element().style.left = left;
        }

        if (options && options.top) {
            const top = options.top;
            this.element().style.position = 'absolute';
            this.element().style.top = top;
        }

        if (options && options.bottom) {
            const bottom = options.bottom;
            this.element().style.position = 'absolute';
            this.element().style.bottom = bottom;
        }

        if (options && options.right) {
            const right = options.right;
            this.element().style.position = 'absolute';
            this.element().style.right = right;
        }

        // handle the anchor/dir case with higher priority
        if (options && options.direction) {
            let dir = options.direction || 'right';
            this.removeClass('left');
            this.removeClass('right');
            this.removeClass('bottom');
            this.removeClass('top');

            let self = this;

            dir.split(' ')
                .forEach(d => {
                    self.addClass(d)
                })
        }
    }

    _show() {
        let tooltipEl = this.el.querySelector('.aladin-tooltip');
        tooltipEl.innerHTML = '';

        if (this.options.hoverable) {
            this.element().style.pointerEvents = "auto";
        }

        if (this.options.delayShowUpTime) {
            this.element().style.transitionDelay = this.options.delayShowUpTime;
        }

        if (this.options.content) {
            let content = [].concat(this.options.content);
            for (var c of content) {
                if (c instanceof DOMElement) {
                    c.attachTo(tooltipEl)
                } else if (c instanceof Element) {                
                    tooltipEl.insertAdjacentElement('beforeend', c);
                } else {
                    let wrapEl = document.createElement('div');
                    wrapEl.innerHTML = c;
                    tooltipEl.insertAdjacentElement('beforeend', wrapEl);
                }
            }
        }

        if (this.options.position) {
            this.setPosition(this.options.position)
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle)
        }

        super._show();
    }

    setCss(options) {
        let el = this.element();

        if (options) {
            for (const property in options) {
                el.style[property] = options[property];
            }
        }
    }

    element() {
        return this.el.querySelector('.aladin-tooltip');
    }
    
    static add(options, target) {
        if (target) {
            if (target.tooltip) {
                target.tooltip.update(options)
            } else {
                // Do not create the tooltip if the device used has touch events
                if ('ontouchstart' in window) {
                    return;
                }

                if (options.global) {
                    let statusBar = options.aladin && options.aladin.statusBar;
                    if (!statusBar) {
                        return;
                    }

                    // handle global tooltip div display
                    Utils.on(target.el, 'mouseover', (e) => {
                        statusBar.removeMessage('tooltip')
                        statusBar.appendMessage({
                            id: 'tooltip',
                            message: options.content,
                            duration: 'unlimited',
                            type: 'tooltip'
                        })
                    });
                    Utils.on(target.el, 'mouseout', (e) => {
                        statusBar.removeMessage('tooltip')
                    });
                    return;
                }

                target.tooltip = new Tooltip(options, target.element())
            }
        }
    }
}