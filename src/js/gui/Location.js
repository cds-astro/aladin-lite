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

/******************************************************************************
 * Aladin Lite project
 * 
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

import { CooConversion } from "../CooConversion.js";
import { Coo }            from "../libs/astro/coo.js";
import { CooFrameEnum }   from "../CooFrameEnum.js";

import { DOMElement } from "./Widgets/Widget.js";
import copyIconUrl from '../../../assets/icons/copy.svg';

import { ALEvent } from "../events/ALEvent.js";
import { Layout } from "./Layout.js";
import { ActionButton } from "./Widgets/ActionButton.js";
import { Input } from "./Widgets/Input.js";
import { Utils } from "../Utils.ts";

function radec2Lonlat(radec, frame) {
    // convert to the view frame
    let lonlat = radec;
    if (frame === "GAL") {
        lonlat = CooConversion.ICRSToGalactic(radec)
    }

    return lonlat
}

export class Location extends DOMElement {
    // constructor
    constructor(aladin) {
        let self;

        let parseCoo = () => {
            let [lon, lat] = aladin.getRaDec()
            let coo = new Coo(lon, lat, 5);
            return coo.format('s/')
        };

        aladin.view.catalogCanvas.addEventListener('click', (e) => {
            self.field.el.blur();
        });

        let focused = false;
        let field = Input.text({
            classList: ['search'],
            tooltip: {
                global: true,
                aladin,
                content: 'Edit for typing an object name/position'
            },
            placeholder: "Search for an object...",
            autocomplete: 'off',
            autofocus: true,
            actions: {
                dblclick: (_) => {
                    field.set('')
                },
                focus: (e) => {
                    focused = true;
                },
                blur: (e) => {
                    focused = false;
                },
                keydown: (e) => {
                    e.stopPropagation();

                    field.removeClass('aladin-not-valid'); // remove red border
                    field.removeClass('aladin-valid'); // remove red border

                    if (e.key === 'Enter') {
                        let object = field.get();

                        field.update({placeholder: 'Resolving ' + object + '...'})
                        field.set('');

                        aladin.gotoObject(
                            object,
                            {
                                error: function () {
                                    field.addClass('aladin-not-valid');
                                    field.update({placeholder: object + ' not found...'})
                                    field.set('');
                                    field.el.focus();
                                },
                                success: function() {
                                    field.addClass('aladin-valid');

                                    field.update({placeholder:'Search for an object...', value: object});
                                }
                            }
                        );
                    }
                }
            },
            value: parseCoo(),
        });

        field.addClass("aladin-medium-sized")

        let copyBtn = new ActionButton({
            icon: {
                monochrome: true,
                size: 'medium',
                url: copyIconUrl,
            },
            tooltip: {content: 'Copy to clipboard!', position: {direction: 'bottom'}},
            action(e) {
                self.copyCoordinatesToClipboard()
            },
        })
        copyBtn.el.classList.add("aladin-location-copy");
 
        let el = Layout.horizontal([
            copyBtn,
            field
        ])
        el.addClass('aladin-location');

        super(el)

        this.field = field;

        self = this;
        ALEvent.CANVAS_EVENT.listenedBy(aladin.aladinDiv, function (e) {
            let param = e.detail;

            let frame = aladin.getFrame();

            if (param.type === 'mouseout') {
                let [ra, dec] = aladin.getRaDec();

                self.update({
                    ra, dec,
                    frame,
                    center: true,
                }, aladin);
            }

            if(param.state.dragging) {
                self.field.el.blur()
            }

            if (param.type === 'mousemove' && param.state.dragging === false) {
                if (focused) {
                    return;
                }

                self.update({
                    mouseX: param.xy.x,
                    mouseY: param.xy.y,
                    frame,
                    center: false,
                }, aladin);
            }
        });

        ALEvent.POSITION_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            // center position in ICRS
            let {ra, dec} = e.detail;
            let frame = aladin.getFrame();

            self.update({
                ra, 
                dec,
                center: true,
                frame
            }, aladin);
        });

        ALEvent.FRAME_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let [ra, dec] = aladin.getRaDec();
            let frame = aladin.getFrame();

            self.update({
                ra, dec,
                center: true,
                frame
            }, aladin);
        });

        this.aladin = aladin;

        let [ra, dec] = aladin.getRaDec();
        let frame = aladin.getFrame();

        this.update({
            ra,
            dec,
            frame,
            center: true
        }, aladin)
    };

    static prec = 6;

    update(options, aladin) {
        let self = this;
        // lon and lat must be given in cooFrame
        const updateFromLonLatFunc = (lon, lat, cooFrame) => {
            var coo = new Coo(lon, lat, Location.prec);

            cooFrame = CooFrameEnum.fromString(cooFrame);

            if (cooFrame == CooFrameEnum.ICRS) {
                self.field.set(coo.format('s/'));
            }
            else if (cooFrame == CooFrameEnum.ICRSd) {
                self.field.set(coo.format('d/'))
            }
            else {
                self.field.set(coo.format('d/'))
            }
            self.field.removeClass('aladin-not-valid');
            self.field.removeClass('aladin-valid'); 

            self.field.element().style.color = options.center ? 'var(--aladin-color)' : 'var(--text-color)';
        };

        if (options.ra && options.dec) {
            let [lon, lat] = radec2Lonlat([options.ra, options.dec], options.frame)
            updateFromLonLatFunc(lon, lat, options.frame);
        } else if (options.mouseX && options.mouseY) {
            try {
                let lonlat = aladin.pix2world(options.mouseX, options.mouseY); // This is given in the frame of the view
                if (lonlat) {
                    if (lonlat[0] < 0) {
                        lonlat = [lonlat[0] + 360.0, lonlat[1]];
                    }
    
                    updateFromLonLatFunc(lonlat[0], lonlat[1], options.frame);
                }
            } catch(e) {}
        }
    }

    copyCoordinatesToClipboard() {
        let msg;
        const cooText = this.field.get();
        Utils.copy2Clipboard(cooText)
            .then(() => {
                msg = 'successful'
                if (this.aladin.statusBar) {
                    this.aladin.statusBar.appendMessage({
                        message: 'Reticle location saved!',
                        duration: 2000,
                        type: 'info'
                    })
                }
            })
            .catch((e) => {
                msg = 'unsuccessful'
                console.info('Oops, unable to copy', e);
            })
            .finally(() => {
                console.info('Copying text command was ' + msg);
            })
    }
};

