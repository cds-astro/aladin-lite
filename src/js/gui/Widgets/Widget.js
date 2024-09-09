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
            let el = this.el;
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

        el.classList.remove('aladin-anchor-left');
        el.classList.remove('aladin-anchor-right');
        el.classList.remove('aladin-anchor-center');
        el.classList.remove('aladin-anchor-top');
        el.classList.remove('aladin-anchor-bottom');
        el.classList.remove('aladin-anchor-middle');

        delete el.style.removeProperty("left");
        delete el.style.removeProperty("right");
        delete el.style.removeProperty("top");
        delete el.style.removeProperty("bottom");
        delete el.style.removeProperty("transform");

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

        let aladinDiv = options && options.aladin && options.aladin.aladinDiv;
        if (!aladinDiv) {
            aladinDiv = el.closest('.aladin-container');
        }
        let innerWidth = aladinDiv && aladinDiv.offsetWidth;
        let innerHeight = aladinDiv && aladinDiv.offsetHeight;


        let left, top, bottom, right;
        let x, y;

        // handle the anchor/dir case with higher priority
        const {offsetWidth, offsetHeight} = el;
        
        // take on less priority the left and top
        if (options && (options.left !== undefined || options.top !== undefined || options.right !== undefined || options.bottom !== undefined)) {
            el.style.position = 'absolute';

            if (options.top !== undefined) {
                top = options.top;
            }
            if (options.left !== undefined) {
                left = options.left;
            }
            if (options.bottom !== undefined) {
                bottom = options.bottom;
            }
            if (options.right !== undefined) {
                right = options.right;
            }

            if (typeof top === 'number') {
                if (innerHeight && top + offsetHeight >= innerHeight) {
                    y = '-' + (top + offsetHeight - innerHeight) + 'px';
                } else if (top < 0) {
                    y = Math.abs(top) + 'px';
                }

                top = top + 'px';
            }
            if (typeof bottom === 'number') {
                bottom = bottom + 'px';
            }
            if (typeof left === 'number') {
                if (innerWidth && left + offsetWidth > innerWidth) {
                    x = '-' + (left + offsetWidth - innerWidth) + 'px';
                } else if (left < 0) {
                    x = Math.abs(left) + 'px';
                }

                left = left + 'px';
            }
            if (typeof right === 'number') {
                right = right + 'px';
            }
        } else if (options && options.nextTo) {
            let dir = options.direction;
            let nextTo = options.nextTo;
            let aDivRect = aladinDiv.getBoundingClientRect();
            const offViewX = aDivRect.x;
            const offViewY = aDivRect.y;
            if (!dir) {
                // determine the direction with respect to the element given
                let elX = options.nextTo.el.getBoundingClientRect().left + options.nextTo.el.getBoundingClientRect().width * 0.5 - offViewX;
                dir = (elX < innerWidth / 2) ? 'right' : 'left';
            }

            if (nextTo instanceof DOMElement) {
                nextTo = nextTo.element();
            }

            let rect = nextTo.getBoundingClientRect();

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

            // Translate if the div in 
            if (typeof top === 'number') {
                if (top + offsetHeight >= innerHeight) {
                    y = '-' + (top + offsetHeight - innerHeight) + 'px';
                } else if (top < 0) {
                    y = Math.abs(top) + 'px';
                }

                top = top + 'px';
            }
            if (typeof bottom === 'number') {
                bottom = bottom + 'px';
            }
            if (typeof left === 'number') {
                if (left + offsetWidth > innerWidth) {
                    x = '-' + (left + offsetWidth - innerWidth) + 'px';
                } else if (left < 0) {
                    x = Math.abs(left) + 'px';
                }

                left = left + 'px';
            }
            if (typeof right === 'number') {
                right = right + 'px';
            }
        }

        if (bottom !== undefined) {
            el.style.bottom = bottom;
        }
        if (top !== undefined) {
            el.style.top = top;
        }
        if (left !== undefined) {
            el.style.left = left;
        }
        if (right !== undefined) {
            el.style.right = right;
        }

        if (x || y) {
            if (!x)
                x = 0
            if (!y)
                y = 0

            el.style.transform = `translate(${x}, ${y})`;
        }
    }

    _show() {
        this.el.style.display = ""
        this.isHidden = false;
    }

    _hide() {
        this.isHidden = true;
        this.el.style.display = 'none';
    }

    attachTo(target, position = 'beforeend') {
        if(target) {
            if (typeof position === 'number') {
                target.insertBefore(this.element(), target.childNodes[position]);
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

        if (this.isHidden) {
            return;
        }

        this._show();
    }
};
