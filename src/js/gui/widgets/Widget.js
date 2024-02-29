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

export class DOMElement {
    constructor(el, options) {
        let element;
        if (el instanceof DOMElement) {
            element = el.element();
        } else if (el instanceof Element) {
            element = el;
        } else {
            element = document.createElement('div');
            element.innerHTML = el;
        }

        this.el = element;
        this.options = options;
        this.isHidden = true;
    }

    element() {
        if (this.tooltip) {
            return this.tooltip.el;
        }

        return this.el;
    }

    addClass(className) {
        this.el.classList.add(className);
    }
    removeClass(className) {
        this.el.classList.remove(className);
    }

    remove() {
        let el = this.element();
        if (el) {
            let target = el.parentNode;

            let index = 0;
            if (target && target.children) {
                index = Array.prototype.indexOf.call(target.children, el);
            }
            el.remove()

            return {target, position: index};
        }
    }

    setCss(options) {
        // CSS style elements
        if (options) {
            let el = this.el
            for (const property in options) {
                el.style[property] = options[property];
            }
        }
    }

    appendContent(content) {
        DOMElement.appendTo(content, this.el)
    }

    static appendTo(elmt, parent) {
        if(elmt) {
            // Append the updated content
            if (elmt instanceof DOMElement) {
                elmt.attachTo(parent)
            } else if (elmt instanceof Element) {                
                parent.insertAdjacentElement('beforeend', elmt);
            } else {
                let wrapEl = document.createElement('div');
                wrapEl.innerHTML = elmt;
                parent.insertAdjacentElement('beforeend', wrapEl);
            }
        }
    }

    setPosition(options) {
        let el = this.element();

        if (options && options.anchor) {
            el.style.position = 'absolute';

            const [lr, tb] = options.anchor.split(' ').filter(s => s !== '');
            if (lr === 'left') {
                el.classList.add('aladin-anchor-left')
            } else if (lr === 'right') {
                el.classList.add('aladin-anchor-right')
            } else if (lr === 'center') {
                el.classList.add('aladin-anchor-center')
            }

            if (tb === 'top') {
                el.classList.add('aladin-anchor-top')
            } else if (tb === 'bottom') {
                el.classList.add('aladin-anchor-bottom')
            } else if (tb === 'center') {
                el.classList.add('aladin-anchor-middle')
            }

            return;
        }

        let left = 0, top = 0, bottom, right;
        let x = 0, y = 0;

        // handle the anchor/dir case with higher priority
        const {offsetWidth, offsetHeight} = el;
        const aladinDiv = document.querySelector('.aladin-container');

        const innerWidth = aladinDiv.offsetWidth;
        const innerHeight = aladinDiv.offsetHeight;

        // take on less priority the left and top
        if (options && (options.left || options.top || options.right || options.bottom)) {
            el.style.position = 'absolute';

            if (options.top) {
                top = options.top;
            }
            if (options.left) {
                left = options.left;
            }
            if (options.bottom) {
                bottom = options.bottom;
            }
            if (options.right) {
                right = options.right;
            }
        } else if (options && options.nextTo && options.direction) {
            let dir = options.direction || 'right';
            let nextTo = options.nextTo;

            if (nextTo instanceof DOMElement) {
                nextTo = nextTo.element();
            }

            let rect = nextTo.getBoundingClientRect();
            let aDivRect = aladinDiv.getBoundingClientRect();

            const offViewX = aDivRect.x;
            const offViewY = aDivRect.y;

            switch (dir) {
                case 'left':
                    left = rect.x - offsetWidth - offViewX;
                    top = rect.y - offViewY;
                    break;
                case 'right':
                    left = rect.x + rect.width - offViewX;
                    top = rect.y - offViewY;
                    break;
                case 'top':
                    left = rect.x - offViewX;
                    top = rect.y - offsetHeight - offViewY;
                    break;
                case 'bottom':
                    left = rect.x - offViewX;
                    top = rect.y + rect.height - offViewY;
                    break;
                default:
                    left = 0;
                    top = 0;
                    break;
            }   
        }

        // Translate if the div in 
        if (left + offsetWidth > innerWidth) {
            x = '-' + (left + offsetWidth - innerWidth) + 'px';
        }   

        if (top + offsetHeight >= innerHeight) {
            y = '-' + (top + offsetHeight - innerHeight) + 'px';
        }

        if (left < 0) {
            x = Math.abs(left) + 'px';
        }

        if (top < 0) {
            y = Math.abs(top) + 'px';
        }

        if (top !== undefined) {
            el.style.top = top + 'px';
        }
        if (left !== undefined) {
            el.style.left = left + 'px';
        }
        if (bottom !== undefined) {
            el.style.bottom = bottom + 'px';
        }
        if (right !== undefined) {
            el.style.right = right + 'px';
        }

        el.style.transform = `translate(${x}, ${y})`;
    }

    _show() {
        this.el.style.display = 'block';
        this.isHidden = false;

        // recursively
        //this._updateTooltipAfterInsertion();
    }

    _hide() {
        this.isHidden = true;
        this.el.style.display = 'none';
    }

    attachTo(target, position = 'beforeend') {
        if(target) {
            if (typeof position === 'number') {
                target.insertChildAtIndex(this.element(), position)
            } else {
                target.insertAdjacentElement(position, this.element());
            }
        }
    }

    update(options) {
        // if no options given, use the previous one set
        if (options) {
            this.options = {...this.options, ...options};
        }

        this._show();
    }
};
