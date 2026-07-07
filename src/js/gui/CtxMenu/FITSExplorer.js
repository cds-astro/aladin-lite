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
 * File FITSExplorer
 *
 * Author: Matthieu Baumann [CDS]
 *
 *****************************************************************************/

import { Input } from "../Widgets/Input.js";
import cubeRenderingIconUrl from "../../../../assets/icons/cube.svg";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Utils } from "../../Utils";
import uploadIconUrl from '../../../../assets/icons/upload.svg';


export let FITSExplorerCtxMenu = (function () {

    let FITSExplorerCtxMenu = {};

    FITSExplorerCtxMenu.getLayout = async function (fits, aladin) {

        let i = 0;
        let ctxMenu = []
        for(const header of fits.headers) {
            let name = await fits.getHDUName(i)

            let j = i;
            let showCheckbox = Input.checkbox({
                name,
                checked: aladin.view.wasm.isHDUVisible(fits.layer, j),
                click: (e) => {
                    e.stopPropagation();

                    const checked = e.target.checked;
                    showCheckbox.update({checked});

                    aladin.view.wasm.makeHDUVisible(fits.layer, j, checked);
                },
                tooltip: {
                    content: 'Show/hide HDU',
                }
            })
            let label = {
                content: [
                    name,
                    showCheckbox
                ]
            };
            const naxis = header.get('NAXIS')?.Integer?.value;
            const naxis1 = header.get('NAXIS1')?.Integer?.value;
            const naxis2 = header.get('NAXIS2')?.Integer?.value;
            const naxis3 = header.get('NAXIS3')?.Integer?.value;

            if (naxis && naxis >= 3 && naxis1 <= 512 && naxis2 <= 512 && naxis3 <= 512) {
                // FITS cube HDU
                label.content.splice(0, 0, new ActionButton({
                    icon: {
                        url: cubeRenderingIconUrl,
                        monochrome: true,
                    },
                    size: "small",
                    tooltip: {
                        global: true,
                        aladin,
                        content: "Explore the cube in 3D!",
                    },
                    action: (_) => {
                        fetch(fits.url)
                            .then((resp) => resp.arrayBuffer())
                            .then((fileBuffer) => {
                                const sessionId = Utils.uuidv4();
                                const fits3 = window.open('https://aladin.cds.unistra.fr/fits3?id=' + sessionId + '&origin=' + encodeURIComponent(window.location.origin));
                                window.addEventListener('message', (event) => {
                                    if (event.origin !== 'https://aladin.cds.unistra.fr') return;
                                    if (event.data.type === 'ready' && event.data.id === sessionId) {
                                        fits3.postMessage(
                                            { type: 'fits-file', id: sessionId, buffer: fileBuffer },
                                            'https://aladin.cds.unistra.fr/fits3'
                                        );
                                    }
                                });
                            })
                    }
                }));
            }
            ctxMenu.push({
                label,
                action: (_) => {
                    /*new Box({
                            header: {
                                draggable: true,
                                title: name
                            },
                            content: fitsHeaderHTML(header),
                        },
                        aladin.aladinDiv
                    );*/

                    let headerTable = {
                        name,
                        color: 'white',
                        rows: fitsHeaderHTML(header),
                        fields: [{name: 'Keyword'}, {name: 'Type'}, {name: 'Value'}, {name: 'Comment'}],
                        icon: uploadIconUrl
                    }

                    aladin.measurementTable.addTab(headerTable);
                },
            })
            i++;
        }

        return ctxMenu;
    }

    return FITSExplorerCtxMenu;

})();



function fitsHeaderHTML(header) {
  function typeInfo(obj) {
    const key = Object.keys(obj)[0];
    const inner = obj[key];
    const val = inner.value;
    const comment = (inner.comment || '').trim();
    let badge, display;
    if      (key === 'Integer') { badge = 'badge-int'; display = String(val); }
    else if (key === 'Float')   { badge = 'badge-flt'; display = String(val); }
    else if (key === 'String')  { badge = 'badge-str'; display = `'${val}'`; }
    else if (key === 'Logical') { badge = 'badge-log'; display = val ? 'T' : 'F'; }
    else                        { badge = 'badge-unk'; display = String(val); }
    return { type: key, badge, display, comment };
  }

  const rows = [...header].map(([kw, obj]) => {
    const { type, badge, display, comment } = typeInfo(obj);
    return { data: { Keyword: kw, Type: `<span class="badge ${badge}">${type}</span>`, Value: display, Comment: comment } };
  });

  return rows;
}
