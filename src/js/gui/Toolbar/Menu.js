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

import { ActionButton } from "../Widgets/ActionButton";
import { DOMElement } from "../Widgets/Widget";

/* Control import */
import { SettingsCtxMenu } from "../CtxMenu/Settings";
import { Stack } from "../CtxMenu/SurveyStack";
import { OverlayStack } from "../CtxMenu/OverlayStack";
import { GotoBox } from "../Box/GotoBox";
import { SimbadPointer } from "../Button/SimbadPointer";
import { ProjectionActionButton } from "../Button/Projection";

import settingsIcon from './../../../../assets/icons/settings.svg';
import stackOverlayIconUrl from './../../../../assets/icons/stack.svg';
import stackImageIconUrl from './../../../../assets/icons/telescope.svg';
import { GridEnabler } from '../Button/GridEnabler';
import searchIcon from './../../../../assets/icons/search.svg';

import { Utils as UtilsExt } from "../../Utils";
import { Utils } from "../Utils";

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
import { Toolbar } from "../Widgets/Toolbar";
import { ALEvent } from "../../events/ALEvent";
import { View } from "../../View";

/**
 * Class representing a Tabs layout
 * @extends DOMElement
 */
 export class Menu extends Toolbar {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(options, aladin) {
        super(options, aladin.aladinDiv)
        let self = this;

        // When the menu resize we close it.
        // For smartphone, we only make the menu close when the orientation is changing
        if (UtilsExt.hasTouchScreen()) {
            if (screen && 'orientation' in screen) {
                screen.orientation.addEventListener("change", (e) => {
                    self.closeAll()
                });
            } else {
                window.addEventListener('orientationchange', (e) => {
                    self.closeAll()
                })
            }
        } else {
            window.addEventListener('resize', () => {
                self.closeAll()
            })
        }

        // tools
        let stack = new Stack(aladin, self);
        let overlay = new OverlayStack(aladin);
        let goto = new GotoBox(aladin);
        let settings = new SettingsCtxMenu(aladin, self);

        this.panels = {
            stack, overlay, goto, settings
        };
        this.indices = [];

        this.aladin = aladin;

        this._initControls();
    }

    _initControls() {
        let self = this;
        let aladin = this.aladin;

        this.controls = {
            stack: ActionButton.createIconBtn({
                iconURL: stackImageIconUrl,
                tooltip: {
                    content: 'Open the stack layer menu',
                    position: {
                        direction: 'top'
                    }
                },
                action(o) {
                    let toolWasShown = !self.panels["stack"].isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        self.panels["stack"]._show({
                            position: {
                                nextTo: self.controls['stack'],
                                direction: self.options.direction === 'right' ? 'left' : 'right',
                            }
                        });
                    }
                }
            }),
            overlay: ActionButton.createIconBtn({
                iconURL: stackOverlayIconUrl,
                tooltip: {
                    content: 'Open the overlays menu',
                    position: {
                        direction: 'top'
                    }
                },
                action(o) {
                    let toolWasShown = !self.panels["overlay"].isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        self.panels["overlay"]._show({
                            position: {
                                nextTo: self.controls['overlay'],
                                direction: self.options.direction === 'right' ? 'left' : 'right',
                            }
                        });
                    }
                }
            }),
            projection: new ProjectionActionButton(aladin, {
                openDirection: self.options.direction === 'right' ? 'left' : 'right',
                action(o) {
                    // executed before opening the ctx menu
                    self.closeAll();
                }
            }),
            simbad: new SimbadPointer(aladin),
            goto: ActionButton.createIconBtn({
                iconURL: searchIcon,
                tooltip: {
                    content: 'Search for where a celestial object is',
                    position: {
                        direction: 'top'
                    }
                },
                action(o) {
                    let toolWasShown = !self.panels["goto"].isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        self.panels["goto"]._show({
                            position: {
                                nextTo: self.controls['goto'],
                                direction: self.options.direction === 'right' ? 'left' : 'right',
                            }
                        });
                    }
                }
            }),
            grid: new GridEnabler(aladin),
            settings: ActionButton.createIconBtn({
                iconURL: settingsIcon,
                tooltip: {
                    content: 'Some general settings e.g. background color, reticle, windows to show',
                    position: {
                        direction: 'top'
                    }
                },
                action(o) {
                    let toolWasShown = !self.panels["settings"].isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        self.panels["settings"]._show({
                            position: {
                                nextTo: self.controls["settings"],
                                direction: self.options.direction === 'right' ? 'left' : 'right',
                            }
                        });
                    }
                }
            })
        };
    }

    closeAll() {
        for (const name in this.tools) {
            let panel = this.panels[name];
            panel && panel._hide();
        }

        this.controls.projection.hideMenu()
    }

    enable(name) {
        if (!this.contains(name)) {
            // add the tool
            const idx = Object.keys(this.controls).indexOf(name);
            let insertIdx = Utils.binarySearch(this.indices, idx);
            this.indices.splice(insertIdx, 0, idx);

            this.appendAtIndex(this.controls[name], name, insertIdx)            
        }

        // show it
        this.show(name);

        // update the settings once a tool has been added
        this.panels["settings"]._attach();
    }

    disable(name) {
        // If it is not even added, do nothing
        if (!this.contains(name)) {
            return;
        }

        this.hide(name);

        // update the settings once a tool has been added
        this.panels["settings"]._attach();
    }

    open(name) {
        this.closeAll();
        this.panels[name]._show({
            position: {
                nextTo: this.controls[name],
                direction: this.options.direction === 'right' ? 'left' : 'right',
            }
        });
    }
}
 
