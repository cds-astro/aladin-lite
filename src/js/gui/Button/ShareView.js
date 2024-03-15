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

import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import shareIconUrl from '../../../../assets/icons/share.svg';
import cameraIconUrl from '../../../../assets/icons/camera.svg';
import linkIconUrl from '../../../../assets/icons/link.svg';
import jupyterIconUrl from '../../../../assets/icons/jupyter.svg';

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
 * @extends CtxMenuActionButtonOpener
 */
 export class ShareActionButton extends CtxMenuActionButtonOpener {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        let layout = [
            {
                label: {
                    content: 'Get view URL',
                    icon: {
                        monochrome: true,
                        url: linkIconUrl,
                        size: 'small',
                    },
                    tooltip: {
                        content: 'View URL will be saved into your clipboard',
                        position: {
                            direction: 'right'
                        }
                    }
                },
                action(o) {
                    var url = aladin.getShareURL();
                    navigator.clipboard.writeText(url);

                    if (aladin.statusBar) {
                        aladin.statusBar.appendMessage({
                            message: 'View URL saved into your clipboard!',
                            duration: 2000,
                            type: 'info'
                        })
                    }
                }
            },
            {
                label: 'HiPS2FITS cutout',
                action(o) {
                    let hips2fitsUrl = 'https://alasky.cds.unistra.fr/hips-image-services/hips2fits#';
                    let radec = aladin.getRaDec();
                    let fov = Math.max.apply(null, aladin.getFov());
                    let hipsId = aladin.getBaseImageLayer().id;
                    let proj = aladin.getProjectionName();
                    hips2fitsUrl += 'ra=' + radec[0] + '&dec=' + radec[1] + '&fov=' + fov + '&projection=' + proj + '&hips=' + encodeURIComponent(hipsId);
                    window.open(hips2fitsUrl, '_blank');
                }
            },
            {
                label: {
                    content: 'Export to notebook',
                    icon: {
                        url: jupyterIconUrl,
                        size: 'medium',
                    },
                    tooltip: {
                        content: '<i><font color="#ff0000">Not implemented</font></i><br/>Launch a notebook with <a href="https://github.com/cds-astro/ipyaladin" target="_blank"><font color="#fff">ipyaladin</font></a> inside.',
                        hoverable: true,
                        position: {
                            direction: 'right'
                        }
                    }
                },
                disabled: true,
            },
            {
                label: {
                    icon: {
                        tooltip: {content: 'Download a PNG image file of the view', position: {direction: 'top'}},
                        monochrome: true,
                        url: cameraIconUrl,
                        size: 'small',
                    },
                    content: 'Take a snapshot'
                },
                action(o) {
                    aladin.exportAsPNG()
                }
            },
        ];

        super({
            size: 'medium',
            ctxMenu: layout,
            classList: ['aladin-share-control'],
            ...options,
            icon: {
                url: shareIconUrl,
                monochrome: true,
            },
            tooltip: {content: 'You can share/export your view into many ways', position: {direction: 'top'}},
        }, aladin);

        self = this;
    }
}
