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
import { FSM } from "../../FiniteStateMachine";
import { ActionButton } from "./ActionButton";
import { ContextMenu } from "./ContextMenu";
import { Layout } from "../Layout";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Form.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

/* 
Exemple of layout object
{
    {
        label: "ID",
        type: "text",
        value: "the placeholder value...",
    },
*/

/*
options = {id: (btn option), id2: btn option, selected: id}
*/
export class SelectorButton extends DOMElement {
    /**
     * Create a layout
     * @param {{layout: {type: String, name: String, value: Number | String, placeholder: Number | String, change: Function } | {type: String, name: String, checked: Boolean, change: Function } | { type: String, name: String, value: String, options: Array.<String>, change: Function }, cssStyle: Object}} options - Represents the structure of the Tabs
     * @param {DOMElement} target - The parent element.
     * @param {String} position - The position of the tabs layout relative to the target.
     *     For the list of possibilities, see https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML
     */
    constructor(options, aladin, target, position = 'beforeend') {
        let el;
        super(el, options)

        this.aladin = aladin;

        let self = this;
        let ctxMenu = new ContextMenu(this.aladin, {hideOnClick: false});

        let openCtxMenuOnClick = (e) => {
            this.fsm.dispatch('closeCtxMenu')
        };
        const openCtxMenu = () => {
            this.aladin.aladinDiv.addEventListener('click', openCtxMenuOnClick)
            
            let menuOptions = [];
            for (const id in this.options) {
                if (id === 'selected' || this.options.selected === id || id === 'tooltip') {
                    continue;
                }
                let optSelect = this.options[id];

                menuOptions.push({
                    label: new ActionButton(optSelect),
                    action(e) {
                        if(optSelect.change) {
                            optSelect.change(e)
                        }

                        self.update({selected: id});
                        self._show();

                        self.fsm.dispatch('closeCtxMenu')
                    },
                    cssStyle: {
                        padding: "0",
                    }
                })
            }
            ctxMenu.attach(menuOptions);
            ctxMenu.show({
                position: {
                    nextTo: this.el,
                    direction: 'bottom',
                }
            })
        };
        const closeCtxMenu = () => {
            this.aladin.aladinDiv.removeEventListener('click', openCtxMenuOnClick)
            ctxMenu._hide();
        };

        this.fsm = new FSM({
            state: 'init',
            transitions: {
                init: {
                    openCtxMenu
                },
                openCtxMenu: {
                    closeCtxMenu
                },
                closeCtxMenu: {
                    openCtxMenu
                },
            }
        })

        this._show();
    }

    _hide() {
        this.fsm.dispatch('closeCtxMenu');
        super._hide()
    }

    _show() {
        let self = this;

        // remove from the DOM tree
        const selectedId = this.options.selected;

        let {target, position} = this.remove();
        
        this.el = new ActionButton({
            ...this.options[selectedId],
            action: (e) => {
                if (self.fsm.state === 'openCtxMenu') {
                    self.fsm.dispatch('closeCtxMenu');
                } else {
                    self.fsm.dispatch('openCtxMenu');
                }
            }
        }).element();

        // Reattach it at the same position
        this.attachTo(target, position)
    }
}
