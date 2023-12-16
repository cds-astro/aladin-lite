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
import { SurveyCtxMenu } from "../CtxMenu/SurveyCtxMenu";
import { MainSurveyActionButton } from "../Button/MainSurvey";
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

        window.addEventListener('resize', () => {
            self.closeAll()
        })

        // Add the fullscreen control
        let self = this;

        this.controls = {
            survey: new MainSurveyActionButton(
                aladin,
                {
                    action(o) {
                        let toolWasShown = !survey.isHidden;

                        self.closeAll();

                        if (!toolWasShown) {
                            survey._show({
                                position: {
                                    nextTo: self.controls['survey'],
                                    direction: 'bottom',
                                }
                            });
                        }
                    }
                }
            ),
            stack: ActionButton.createIconBtn({
                iconURL: stackImageIconUrl,
                tooltip: {
                    content: 'Open the stack layer menu',
                    position: { direction: 'left'},
                },
                action(o) {
                    let toolWasShown = !stack.isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        stack._show({
                            position: {
                                nextTo: self.controls['stack'],
                                direction: 'bottom',
                            }
                        });
                    }
                }
            }),
            overlay: ActionButton.createIconBtn({
                iconURL: stackOverlayIconUrl,
                tooltip: {
                    content: 'Open the overlays menu',
                    position: { direction: 'left'},
                },
                action(o) {
                    let toolWasShown = !overlay.isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        overlay._show({
                            position: {
                                nextTo: self.controls['overlay'],
                                direction: 'bottom',
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
                    position: { direction: 'left'},
                },
                action(o) {
                    let toolWasShown = !goto.isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        goto._show({
                            position: {
                                nextTo: self.controls['goto'],
                                direction: 'bottom',
                            }
                        });
                    }
                }
            }),
            grid: ActionButton.createIconBtn({
                iconURL: gridIcon,
                tooltip: {
                    content: 'Open the grid layer menu',
                    position: { direction: 'left'},
                },
                action(o) {
                    let toolWasShown = !grid.isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        grid._show({
                            position: {
                                nextTo: self.controls['grid'],
                                direction: 'bottom',
                            }
                        });
                    }
                }
            }),
            settings: ActionButton.createIconBtn({
                iconURL: settingsIcon,
                tooltip: {
                    content: 'Some general settings e.g. background color, reticle, windows to show',
                    position: { direction: 'left' },
                },
                action(o) {
                    let toolWasShown = !settings.isHidden;

                    self.closeAll();

                    if (!toolWasShown) {
                        settings._show({
                            position: {
                                nextTo: self.controls["settings"],
                                direction: 'bottom',
                            }
                        });
                    }
                }
            }),
            fullscreen: ActionButton.createIconBtn({
                iconURL: aladin.isInFullscreen ? restoreIcon : maximizeIcon,
                tooltip: {
                    content: aladin.isInFullscreen ? 'Restore original size' : 'Full-screen',
                    position: { direction: 'left'},
                },
                action(o) {
                    aladin.toggleFullscreen(aladin.options.realFullscreen);
    
                    let btn = self.controls['fullscreen'];
                    if (aladin.isInFullscreen) {
                        btn.update({
                            iconURL: restoreIcon,
                            tooltip: {
                                content: 'Restore original size',
                                position: { direction: 'left'}
                            }
                        });
                    } else {
                        btn.update({
                            iconURL: maximizeIcon,
                            tooltip: {
                                content: 'Fullscreen',
                                position: { direction: 'left'}
                            }
                        });
                    }

                    // hide all the controls
                    self.closeAll()
                }
            }),
        };

        // tools
        let stack = new Stack(aladin, self);
        let overlay = new OverlayStack(aladin);
        let goto = new GotoBox(aladin);
        let grid = new GridBox(aladin);
        let settings = new SettingsCtxMenu(aladin, self);
        let survey = new SurveyCtxMenu(aladin, self);

        this.panels = {
            stack, overlay, goto, grid, settings, survey
        };

        this.indices = [];

        this.aladin = aladin;
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
 
