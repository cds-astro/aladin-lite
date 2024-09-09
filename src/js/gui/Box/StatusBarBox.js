
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
import infoIconUrl from '../../../../assets/icons/info.svg';
import tooltipIconUrl from '../../../../assets/icons/tooltip.svg';
import aladinLogoGif from '../../../../assets/aladin-logo.gif';

import { Icon } from "../Widgets/Icon";

export class StatusBarBox extends Box {
    constructor(aladin, options) {
        super({...options, close: false}, aladin.aladinDiv)

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

        let offsetTimeDisplay = task.offsetTimeDisplay;
        let duration = task.duration;
        if (typeof duration === 'number' && typeof offsetTimeDisplay === 'number' && offsetTimeDisplay > duration) {
            // do not add the message the display occurs after the duration of the task
            return;
        }

        if (duration && duration !== "unlimited") {
            setTimeout(() => {
                this.removeMessage(task.id);
            }, duration)
        }

        // display it
        if (offsetTimeDisplay) {
            setTimeout(() => {
                this._displayLastTaskInProgress();
            }, offsetTimeDisplay)
        } else {
            this._displayLastTaskInProgress();
        }
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
        let message = Layout.horizontal({
            layout: task.message,
            tooltip: {
                content: task.message,
                position: {
                    direction: "top",
                },
                hoverable: true,
                delayShowUpTime: '500ms',
                cssStyle: {
                    fontSize: 'x-small',
                    maxWidth: "200px",
                    "overflow-wrap": "break-word",
                }
            },
        });

        message.addClass("aladin-status-bar-message")

        this._show({
            content: new Layout({layout: [StatusBarBox.icons[task.type], message], orientation: 'horizontal'}),
        })
    }

    static icons = {
        loading: (() => {
            let icon = new Icon({
                size: 'medium',
                url: aladinLogoGif,
                cssStyle: {
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

            return icon
        })(),
        info: new Icon({
            size: 'medium',
            monochrome: true,
            url: infoIconUrl,
            cssStyle: {
                cursor: "help",
            },
        }),
        tooltip: new Icon({
            size: 'medium',
            monochrome: true,
            url: tooltipIconUrl,
            cssStyle: {
                cursor: "help",
            },
        })
    }
}
 