
// Copyright 2013 - UDS/CNRS
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
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
import { Box } from "../Widgets/Box";
import { ALEvent } from "../../events/ALEvent";
import { Utils } from "../../Utils";
import { Layout } from "../Layout";
import { ActionButton } from "../Widgets/ActionButton";
import infoIconUrl from '../../../../assets/icons/info.svg';


export class StatusBarBox extends Box {
    constructor(aladin) {
        super({
            cssStyle: {
                color: 'white',
                backgroundColor: 'black',
                borderRadius: '3px',
                padding: 0,
            }
        }, aladin.aladinDiv)

        this.addClass("aladin-status-bar");

        this.inProgressTasks = [];

        this._addListeners()
    }

    _addListeners() {
        ALEvent.FETCH.listenedBy(document, (e) => {
            let task = e.detail.task;
            this.appendMessage(task);
        });

        ALEvent.RESOURCE_FETCHED.listenedBy(document, (e) => {
            let task = e.detail.task;
            this.removeMessage(task.id);
        });
    }

    appendMessage(task) {
        task.id = task.id || Utils.uuidv4();
        task.type = task.type || 'loading';

        this.inProgressTasks.push(task);

        if (task.duration && task.duration !== "unlimited") {
            setTimeout(() => {
                this.removeMessage(task.id);
            }, task.duration)
        }

        // display it
        this._displayLastTaskInProgress();
    };

    removeMessage(id) {
        const index = this.inProgressTasks.findIndex((t) => t.id === id);
        if (index >= 0) {
            // task found
            this.inProgressTasks.splice(index, 1);

            // If it was the last element, i.e. the one being displayed
            if (index === this.inProgressTasks.length) {
                // display the "new" last
                this._displayLastTaskInProgress();
            }
        }
    };

    _displayLastTaskInProgress() {
        this._hide();

        if (this.inProgressTasks.length === 0) {
            // no more task to run
            return;
        }

        let task = this.inProgressTasks[this.inProgressTasks.length - 1];

        this.el.title = task.message;

        // create message div
        let messageDiv = document.createElement("div");
        messageDiv.className = "aladin-status-bar-message";
        messageDiv.innerHTML = task.message;
        messageDiv.title = task.message;

        this._show({
            content: [StatusBarBox.icons[task.type], messageDiv],
            position: {
                anchor: 'center bottom'
            }
        })
    }

    static icons = {
        loading: (() => {
            let icon = new ActionButton({
                iconURL: "https://raw.githubusercontent.com/cds-astro/aladin-lite/master/assets/aladin-logo.gif",
                cssStyle: {
                    backgroundColor: 'black',
                    border: "none",
                    margin: "5px",
                    cursor: "help",
                },
                tooltip: {
                    content: "Loading...",
                    position: {
                        direction: 'top'
                    }
                },
            })

            icon.addClass("rotating")
            icon.addClass("medium-sized-icon")

            return icon
        })(),
        info: ActionButton.createIconBtn({
            iconURL: infoIconUrl,
            cssStyle: {
                backgroundColor: 'white',
                border: "none",
                margin: "5px",
                cursor: "help",
            },
        })
    }
}
 