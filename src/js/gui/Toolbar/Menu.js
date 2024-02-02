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
import { GridBox } from "../Box/GridBox";

import settingsIcon from './../../../../assets/icons/settings.svg';
import stackOverlayIconUrl from './../../../../assets/icons/stack.svg';
import stackImageIconUrl from './../../../../assets/icons/telescope.svg';
import gridIcon from './../../../../assets/icons/grid.svg';
import searchIcon from './../../../../assets/icons/search.svg';
import restoreIcon from './../../../../assets/icons/restore.svg';
import maximizeIcon from './../../../../assets/icons/maximize.svg';

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
        // For smarthphone, we only make the menu close when the orientation is changing
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

        // Add the fullscreen control
        // tools
        let stack = new Stack(aladin, self);
        let overlay = new OverlayStack(aladin);
        let goto = new GotoBox(aladin);
        let grid = new GridBox(aladin);
        let settings = new SettingsCtxMenu(aladin, self);

        this.panels = {
            stack, overlay, goto, grid, settings
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
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right' },
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
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right'},
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
            simbad: new SimbadPointer(aladin),
            goto: ActionButton.createIconBtn({
                iconURL: searchIcon,
                tooltip: {
                    content: 'Search for where a celestial object is',
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right'},
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
            grid: ActionButton.createIconBtn({
                iconURL: gridIcon,
                tooltip: {
                    content: 'Open the grid layer menu',
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right'},
                },
                action(o) {
                    let toolWasShown = !self.panels["grid"].isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        self.panels["grid"]._show({
                            position: {
                                nextTo: self.controls['grid'],
                                direction: self.options.direction === 'right' ? 'left' : 'right',
                            }
                        });
                    }
                }
            }),
            settings: ActionButton.createIconBtn({
                iconURL: settingsIcon,
                tooltip: {
                    content: 'Some general settings e.g. background color, reticle, windows to show',
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right' },
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
            }),
            fullscreen: ActionButton.createIconBtn({
                iconURL: aladin.isInFullscreen ? restoreIcon : maximizeIcon,
                tooltip: {
                    content: aladin.isInFullscreen ? 'Restore original size' : 'Full-screen',
                    position: { direction: self.options.direction === 'right' ? 'left' : 'right'},
                },
                action(o) {
                    aladin.toggleFullscreen(aladin.options.realFullscreen);    
                    let btn = self.controls['fullscreen'];

                    if (aladin.isInFullscreen) {
                        // make that div above other aladin lite divs (if there are...)
                        aladin.aladinDiv.style.zIndex = 1
                        btn.update({
                            iconURL: restoreIcon,
                            tooltip: {
                                content: 'Restore original size',
                                position: { direction: self.options.direction === 'right' ? 'left' : 'right'}
                            }
                        });
                    } else {
                        aladin.aladinDiv.style.removeProperty('z-index')

                        btn.update({
                            iconURL: maximizeIcon,
                            tooltip: {
                                content: 'Fullscreen',
                                position: { direction: self.options.direction === 'right' ? 'left' : 'right'}
                            }
                        });
                    }

                    // hide all the controls
                    self.closeAll()
                }
            }),
        };
    }

    closeAll() {
        for (const name in this.tools) {
            let panel = this.panels[name];
            panel && panel._hide();
        }
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
}
 
