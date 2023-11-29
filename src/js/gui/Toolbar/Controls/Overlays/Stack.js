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
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

 import { ALEvent } from "../../../../events/ALEvent.js";
 import { Layout } from "../../../Layout.js";
 import { ContextMenu } from "../../../Widgets/ContextMenu.js";
 import { ActionButton } from "../../../Widgets/ActionButton.js";
 import showIconUrl from '../../../../../../assets/icons/show.svg';
 import hideIconUrl from '../../../../../../assets/icons/hide.svg';
 import removeIconUrl from '../../../../../../assets/icons/remove.svg';
 
 export class OverlayStack extends ContextMenu {
     // Constructor
     constructor(aladin, menu) {
         super(aladin);
         this.aladin = aladin;
         this.anchor = menu.controls["OverlayStack"];
 
         this._addListeners();
     }
 
     _addListeners() {
        let self = this;
 
        let updateImageList = () => {
            const overlays = Array.from(self.aladin.getOverlays()).reverse().map((overlay) => {
                return overlay;
            });


            self.attach({overlays});
            // If it is shown, update it
            if (!self.isHidden) {
                self.show();
            }
        };
 
         updateImageList();
         
         ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.listenedBy(this.aladin.aladinDiv, function (e) {
             updateImageList();
         });
 
         ALEvent.GRAPHIC_OVERLAY_LAYER_REMOVED.listenedBy(this.aladin.aladinDiv, function (e) {
             updateImageList();
         });
 
         ALEvent.GRAPHIC_OVERLAY_LAYER_CHANGED.listenedBy(this.aladin.aladinDiv, function (e) {
             updateImageList();
         });
     }
 
     attach(options) {
         const overlays = options && options.overlays || [];
 
         /*let layout = [{
             label: Layout.horizontal({
                 layout: [
                     ActionButton.createIconBtn({
                         iconURL: searchIconUrl,
                         tooltip: {content: 'Add a survey <br /> from our database...', position: { direction: 'bottom' }},
                         cssStyle: {
                             backgroundPosition: 'center center',
                             backgroundColor: '#bababa',
                             border: '1px solid rgb(72, 72, 72)',
                             cursor: 'help',
                         },
                     }),
                     'Add a survey/FITS image'
                 ]
             }),
             action(o) {
                 const hipsSelector = HiPSSelectorBox.getInstance(self.aladin);
                 hipsSelector._hide();
                 hipsSelector._show();
 
                 self.fsm.dispatch('hide');
             }
         }];*/
         let layout = [];
 
         let self = this;
         for(const overlay of overlays) {
             console.log(overlay)
             const name = overlay.name;
             let cssStyle = {
                height: 'fit-content',
            };
             let showBtn = ActionButton.createIconBtn({
                 iconURL: showIconUrl,
                 cssStyle: {
                     backgroundColor: '#bababa',
                     borderColor: '#484848',
                     color: 'black',
                     visibility: 'hidden',
                     width: '18px',
                     height: '18px',
                     verticalAlign: 'middle',
                     marginRight: '2px',
                 },
                 tooltip: {content: 'Hide', position: {direction: 'bottom'}},
                 action(e, btn) {
                     if (overlay.isShowing) {
                         overlay.hide()
                         btn.update({iconURL: showIconUrl, tooltip: {content: 'Hide'}});
                     } else {
                         overlay.show()

                         btn.update({iconURL: hideIconUrl, tooltip: {content: 'Show'}});
                     }
                 }
             });
 
             let deleteBtn = ActionButton.createIconBtn({
                 iconURL: removeIconUrl,
                 cssStyle: {
                     backgroundColor: '#bababa',
                     borderColor: '#484848',
                     color: 'black',
                     visibility: 'hidden',
                     width: '18px',
                     height: '18px',
                     verticalAlign: 'middle'
                 },
                 tooltip: {content: 'Remove', position: {direction: 'left'}},
                 action(e) {
                    self.aladin.removeLayer(overlay)
                 }
             });

             let item = Layout.horizontal({
                 layout: [
                     '<div style="background-color: rgba(0, 0, 0, 0.6); padding: 3px; border-radius: 3px; word-break: break-word;">' + name + '</div>',
                     Layout.horizontal({layout: [showBtn, deleteBtn]})
                 ],
                 cssStyle: {
                     display: 'flex',
                     alignItems: 'center',
                     listStyle: 'none',
                     justifyContent: 'space-between',
                     width: '100%',
                 }
             });
             layout.push({
                 label: item,
                 cssStyle: cssStyle,
                 hover(e) {
                     showBtn.el.style.visibility = 'visible'
                     deleteBtn.el.style.visibility = 'visible'
                 },
                 unhover(e) {
                     showBtn.el.style.visibility = 'hidden'
                     deleteBtn.el.style.visibility = 'hidden'
                 },
             })
         }
 
         super.attach(layout);
     }

 
     show() {
         console.log("show")
         super.show({
             position: {
                 anchor: this.anchor,
                 direction: 'bottom',
             },
             cssStyle: {
                 width: '15em',
                 //overflowY: 'scroll',
                 //maxHeight: '500px',
                 color: 'white',
                 backgroundColor: 'black',
                 border: '1px solid white',
             }
         })
     }
 
     hide() {
         super._hide();
     }
     
     static singleton;
 
     static getInstance(aladin, menu, fsm) {
         if (!OverlayStack.singleton) {
            OverlayStack.singleton = new OverlayStack(aladin, menu, fsm);
         }
 
         return OverlayStack.singleton;
     }
 }
    