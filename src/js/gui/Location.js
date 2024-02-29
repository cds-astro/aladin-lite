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
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


import { Coo }            from "../libs/astro/coo.js";
import { CooFrameEnum }   from "../CooFrameEnum.js";

import { DOMElement } from "./Widgets/Widget.js";
import copyIconUrl from '../../../assets/icons/copy.svg';

import { ALEvent } from "../events/ALEvent.js";
import { Layout } from "./Layout.js";
import { ActionButton } from "./Widgets/ActionButton.js";
import { Input } from "./Widgets/Input.js";

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

        let field = Input.text({
            classList: ['search'],
            tooltip: {
                global: true,
                aladin,
                content: 'Search for an object name/position'
            },
            placeholder: "Search for an object...",
            autocomplete: 'off',
            autofocus: true,
            actions: {
                keydown: (e) => {
                    field.removeClass('aladin-unknownObject'); // remove red border

                    if (e.key === 'Enter') {
                        field.el.blur();

                        let object = field.get();

                        field.update({placeholder: 'Resolving ' + object + '...'})
                        field.set('');

                        aladin.gotoObject(
                            object,
                            {
                                error: function () {
                                    field.addClass('aladin-unknownObject');
                                    field.update({placeholder: object + ' not found...'})
                                    field.set('');
                                    field.el.focus();
                                },
                                success: function() {
                                    field.update({placeholder:'Search for an object...', value: object});
                                }
                            }
                        );
                    }
                }
            },
            value: parseCoo(),
            cssStyle: {
                borderRadius: "0px 5px 5px 0px",
            }
        });

        let el = Layout.horizontal({
            layout: [
                new ActionButton({
                    icon: {
                        monochrome: true,
                        size: 'small',
                        url: copyIconUrl,
                    },
                    //tooltip: {content: 'Copy to clipboard!', position: {direction: 'bottom'}},
                    action(e) {
                        self.copyCoordinatesToClipboard()
                    },
                    cssStyle: {
                        height: '1.4rem',
                        width: '1.4rem',
                        marginRight: 0,
                        paddingRight: '0.2rem',
                        borderRadius: "5px 0px 0px 5px",
                        borderRight: 'none',
                    }
                }),
                field
            ]
        })
        el.addClass('aladin-location');

        super(el)

        this.field = field;

        self = this;
        ALEvent.CANVAS_EVENT.listenedBy(aladin.aladinDiv, function (e) {
            let param = e.detail;

            if (param.type === 'mouseout') {
                let [lon, lat] = aladin.getRaDec();
                self.update({
                    lon, lat,
                    frame: aladin.view.cooFrame,
                    isViewCenter: true,
                }, aladin);
            }

            if (param.type === 'mousemove' && param.state.dragging === false) {
                self.update({
                    mouseX: param.xy.x,
                    mouseY: param.xy.y,
                    frame: aladin.view.cooFrame,
                    isViewCenter: false,
                }, aladin);
            }
        });

        ALEvent.POSITION_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            self.update({
                lon: e.detail.lon, 
                lat: e.detail.lat,
                isViewCenter: true,
                frame: aladin.view.cooFrame
            }, aladin);
        });

        ALEvent.FRAME_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let [lon, lat] = aladin.getRaDec();

            self.update({
                lon, lat,
                isViewCenter: true,
                frame: e.detail.cooFrame
            }, aladin);
        });

        this.aladin = aladin;

        let [lon, lat] = aladin.getRaDec();
        this.update({
            lon, lat,
            isViewCenter: true,
            frame: aladin.view.cooFrame
        }, aladin)
    };

    static prec = 7;

    update(options, aladin) {
        let self = this;
        const updateFromLonLatFunc = (lon, lat, cooFrame) => {
            var coo = new Coo(lon, lat, Location.prec);
            if (cooFrame == CooFrameEnum.J2000) {
                self.field.set(coo.format('s/'));
            }
            else if (cooFrame == CooFrameEnum.J2000d) {
                self.field.set(coo.format('d/'))
            }
            else {
                self.field.set(coo.format('d/'))
            }
            self.field.removeClass('aladin-unknownObject');

            self.field.element().style.color = options.isViewCenter ? aladin.getReticle().getColor() : 'white';
            //self.field.el.blur()
        };

        if (options.lon && options.lat) {
            updateFromLonLatFunc(options.lon, options.lat, options.frame, true);
        } else if (options.mouseX && options.mouseY) {
            try {
                let radec = aladin.pix2world(options.mouseX, options.mouseY); // This is given in the frame of the view
                if (radec) {
                    if (radec[0] < 0) {
                        radec = [radec[0] + 360.0, radec[1]];
                    }
    
                    updateFromLonLatFunc(radec[0], radec[1], options.frame, false);
                }
            } catch(e) {}
        }
    }

    copyCoordinatesToClipboard() {
        let msg;
        navigator.clipboard.writeText(this.field.get())
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
