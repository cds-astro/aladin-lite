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
import { Utils } from "./Utils";
export class FSM {
    // Constructor
    constructor(options) {
        this.state = options && options.state;
        this.transitions = options && options.transitions || {};
        this.view = options && options.view;

        const EVENTS = ["mousedown", "mouseup", "mousemove", "mouseout"];

        let self = this;
        for (const [start, to] of Object.entries(this.transitions)) {
            for (const [trigger, action] of Object.entries(to)) {
                if (EVENTS.includes(trigger)) {
                    Utils.on(self.view.aladin.aladinDiv, trigger, function (e) {
                        e.preventDefault();
                        e.stopPropagation();

                        if (self.state == start) {
                            action(e)
                        }
                    })
                }
            }
        }
    }

    start(params) {
        this.dispatch('start', params)
    }

    // Do nothing if the to is inaccesible
    dispatch(to, params) {
        const action = this.transitions[this.state][to];
        if (action) {
            this.state = to;

            if (params) {
                action(params);
            } else {
                action()
            }
        }
    }
}