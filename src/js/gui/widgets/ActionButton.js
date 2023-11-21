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
import { Tooltip } from "./Tooltip";
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

export class ActionButton extends DOMElement {
    constructor(opt, target, position = "beforeend") {
        let el = document.createElement('button');
        el.classList.add('aladin-btn');

        // add it to the dom
        super(el, opt);
        this._show();

        this.attachTo(target, position)
    }

    _show() {
        this.el.innerHTML = '';
        this.el.removeEventListener('click', this.action);

        if (this.options.toggled === true) {
            this.addClass('toggled');
        } else if (this.options.toggled === false) {
            this.removeClass('toggled');
        }

        if (this.options.action) {
            this.action = (e) => {
                e.stopPropagation();
                e.preventDefault();
    
                this.options.action(e, this);
            };

            this.el.addEventListener('click', this.action);
        }

        if (this.options.title) {
            this.el.setAttribute('title', this.options.title);
        }

        if (this.options.iconURL) {
            let img = document.createElement('img');
            img.src = this.options.iconURL;
            img.style.objectFit = 'contain';
            img.style.verticalAlign = 'middle';
            img.style.width = '100%';

            this.el.appendChild(img);
        }

        if (this.options.disable) {
            this.el.disabled = true;
            this.el.style.cursor = "not-allowed";
            this.el.style.filter = 'brightness(70%)';
        } else {
            this.el.disabled = false;
            this.el.style.cursor = 'pointer';
        }

        // Add the content to the dom
        // Content can be a DOM element, just plain text or another Widget instance
        if (this.options.content) {
            this.appendContent(this.options.content);
        }

        if (this.options.cssStyle) {
            this.setCss(this.options.cssStyle);
        }

        // trigger el added
        if (this.options.tooltip) {
            Tooltip.add(this.options.tooltip, this)
        }

        super._show();
    }

    static createIconBtn(opt, target, position = 'beforeend') {
        let btn = new ActionButton(opt, target, position);
        btn.addClass('aladin-24px-icon');

        return btn;
    }

    static create(opt, info, target, position = 'beforeend') {
        opt['info'] = info || undefined;

        return new ActionButton(opt, target, position);
    }

    static DEFAULT_BTN = {
        'loading': {
            content: '⏳',
            width: '28px',
            height: '28px',
            cssStyle: {
                backgroundColor: '#bababa',
                borderColor: '#484848',
            },
            action(e) {}
        },
    }
}
