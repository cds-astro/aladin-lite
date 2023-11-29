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

import { Layout } from "../Layout";
import { ActionButton } from "../Widgets/ActionButton";
import { DOMElement } from "../Widgets/Widget";

/* Control import */
import { Settings } from "./Controls/Settings";
import { StackLayerMenu } from "./Controls/StackLayer/Menu";
import { OverlayStack } from "./Controls/Overlays/Stack";
import { GotoBox } from "./Controls/GotoBox";
import { SimbadPointer } from "./Controls/SimbadPointer";
import { GridBox } from "./Controls/GridBox";

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

/**
 * Class representing a Tabs layout
 * @extends DOMElement
 */
 export class Menu extends DOMElement {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, parent) {
        let layout = Layout.horizontal({
            layout: []
        }, parent.element());

        super(layout)
        /*
        showFullscreenControl,
        showLayersControl,
        showGotoControl,
        showSimbadPointerControl,
        showShareControl,
        showCooGridControl
        */

        // Add the fullscreen control
        const controls = this.defaultControls(aladin);
        this.layout = layout;
        this.controls = controls;
        this.controlIdxAppened = [];

        // Add the layers control
        if (aladin.options && aladin.options.showLayersControl) {
            this.appendControl('StackLayerMenu')
            this.appendControl('OverlayStack')
        }
        // Add the simbad pointer control
        if (aladin.options && aladin.options.showSimbadPointerControl) {
            this.appendControl('SimbadPointer')
        }
        // Add the goto control
        if (aladin.options && aladin.options.showGotoControl) {
            this.appendControl('GotoBox')
        }
        // Add the coo grid control
        if (aladin.options && aladin.options.showCooGridControl) {
            this.appendControl('GridBox')
        }
        // Add the share control
        if (aladin.options && aladin.options.showShareControl) {
            this.appendControl('ShareControl')
        }
        // Settings control
        this.appendControl('Settings')

        if (aladin.options && aladin.options.showFullscreenControl) {
            this.appendControl('FullScreen')
        }

        this.aladin = aladin;
    }

    defaultControls(aladin) {
        let menu = this;
        const controls = {
            StackLayerMenu: new ActionButton({
                iconURL: stackImageIconUrl,
                tooltip: {
                    content: 'Open the stack layer menu',
                    position: { direction: 'left'},
                },
                cssStyle: {
                    padding: 0,
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    width: '28px',
                    height: '28px'

                },
                action(o) {
                    menu.showControl(StackLayerMenu)
                }
            }),
            OverlayStack: new ActionButton({
                iconURL: stackOverlayIconUrl,
                tooltip: {
                    content: 'Open the overlays menu',
                    position: { direction: 'left'},
                },
                cssStyle: {
                    padding: 0,
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    width: '28px',
                    height: '28px'

                },
                action(o) {
                    menu.showControl(OverlayStack)
                }
            }),
            SimbadPointer: new SimbadPointer(aladin),
            GotoBox: new ActionButton({
                iconURL: searchIcon,
                tooltip: {
                    content: 'Search for where a celestial object is',
                    position: { direction: 'left'},
                },
                cssStyle: {
                    padding: 0,
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    width: '28px',
                    height: '28px'
                },
                action(o) {
                    menu.showControl(GotoBox)
                }
            }),
            GridBox: new ActionButton({
                iconURL: gridIcon,
                tooltip: {
                    content: 'Open the grid layer menu',
                    position: { direction: 'left'},
                },
                cssStyle: {
                    padding: 0,
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    width: '28px',
                    height: '28px'
                },
                action(o) {
                    menu.showControl(GridBox)
                }
            }),
            Settings: new ActionButton({
                content: Layout.horizontal({
                    layout: ['Settings', new ActionButton({
                        iconURL: settingsIcon,
                        cssStyle: {
                            padding: 0,
                            color: "black",
                            backgroundColor: '#bababa',
                            backgroundPosition: 'center',
                            borderColor: '#484848',
                            cursor: 'pointer',
                            height: '17px',
                            width: '17px'
                        }
                    })]
                }),
                tooltip: {
                    content: 'Some general settings e.g. background color, reticle, windows to show',
                    position: { direction: 'left' },
                },
                cssStyle: {
                    padding: 0,
                    color: "black",
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    height: '28px'
                },
                action(o) {
                    menu.showControl(Settings)
                }
            }),
            FullScreen: new ActionButton({
                iconURL: aladin.isInFullscreen ? restoreIcon : maximizeIcon,
                tooltip: {
                    content: aladin.isInFullscreen ? 'Restore original size' : 'Full-screen',
                    position: { direction: 'left'},
                },
                cssStyle: {
                    padding: 0,
                    backgroundColor: '#bababa',
                    backgroundPosition: 'center',
                    borderColor: '#484848',
                    cursor: 'pointer',
                    width: '28px',
                    height: '28px'
                },
                action(o, self) {
                    aladin.toggleFullscreen(aladin.options.realFullscreen);
    
                    if (aladin.isInFullscreen) {
                        self.update({iconURL: restoreIcon, info: 'Restore original size'});
                    } else {
                        self.update({iconURL: maximizeIcon, info: 'Full-screen'});
                    }

                    // hide all the controls
                    menu.hideAllControls()
                }
            }),
        }

        return controls;
    }

    hideAllControls() {
        for (const classNameControl in this.controls) {
            try {
                const className = eval(classNameControl)
                this.hideControl(className, this.aladin);
            } catch(e) {}
        }

        // Hide the layer edit box if there is one shown
        //LayerEditBox.getInstance(this.aladin, this.controls["StackLayerMenu"])._hide();
    }

    // show/hide panel
    hideControl(classNameControl) {
        if (classNameControl.getInstance) {
            let tool = classNameControl.getInstance(this.aladin, this);

            tool._hide();
        }
    }

    showControl(toolClass) {
        let tool = toolClass.getInstance(this.aladin, this);
        let toolWasShown = !tool.isHidden;

        console.log('tool hidden', tool.isHidden)
        this.hideAllControls(this.controls, this.aladin);

        if (!toolWasShown) {
            tool._show();
        }
    }

    // remove control and add control
    removeControl(name) {
        const idx = Object.keys(this.controls).indexOf(name);
        if (idx > -1) {
            this._removeIdxControl(idx);

            const ctrl = this.controls[name];
            this.layout.removeItem(ctrl)
        }
    }

    appendControl(name) {
        const idx = Object.keys(this.controls).indexOf(name);

        if (idx > -1) {
            let insertIdx = this._appendIdxControl(idx);

            const ctrl = this.controls[name];
            this.layout.insertItemAtIndex(ctrl, insertIdx)
        }
    }

    _appendIdxControl(idx) {
        const insertIdx = this._getInsertIdxControl(idx);
        this.controlIdxAppened.splice(insertIdx, 0, idx);

        return insertIdx;
    }

    _removeIdxControl(idx) {
        const insertIdx = this._getInsertIdxControl(idx);
        this.controlIdxAppened.splice(insertIdx, 1);

        return insertIdx;
    }

    _getInsertIdxControl(idx) {
        return Utils.binarySearch(this.controlIdxAppened, idx);
    }
}
 
