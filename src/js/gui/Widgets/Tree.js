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

import { DOMElement } from "./Widget";
import { Icon } from "./Icon";
import folderIconUrl from "../../../../assets/icons/folder.svg";

import { Layout } from "../Layout";
import { ActionButton } from "./ActionButton";
import { Input } from "./Input";
/******************************************************************************
 * Aladin Lite project
 *
 * File gui/Tree.js
 *
 * A tree
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************
*/
export class Tree extends DOMElement {
    constructor(options, target, position = "beforeend") {
        let el = document.createElement("div");
        el.classList.add('aladin-tree');

        super(el, options);

        this.click = options && options.click;
        this.dblclick = options && options.dblclick;
        this.aladin = options && options.aladin;

        this.onlyInView = false;

        let rootNode = options && options.root || {};
        this.params = null;
        this.filter = options && options.filter;
        this.label = options && options.label;

        this._setRoot(rootNode);


        this.attachTo(target, position);
        this._show();
    }

    _setRoot(root) {
        if (root) {
            root.label = new Icon({
                size: "small",
                url: folderIconUrl,
                monochrome: true,
                cssStyle: {'display': 'inline-block'}
            }).element().outerHTML;
        }

        this.root = root;
        this._createDOM(root);
    }

    _createDOM(node) {
        if (!node) {
            return;
        }

        this.el.innerHTML = "";
        this.curNode = node;
        
        // create the parent directory
        let curNode = this.curNode;

        let directoryLinks = []

        var levelsOfParenty = 0;

        while (curNode) {
            let parentLinkEl = document.createElement('a');
            parentLinkEl.innerHTML = curNode.label;
            let curLevel = levelsOfParenty;
            parentLinkEl.addEventListener('click', () => {
                this.navigate(curLevel)
            })
            levelsOfParenty += 1;

            parentLinkEl.classList.add("aladin-link");
            directoryLinks.push(parentLinkEl);

            curNode = curNode.parent;
        }

        directoryLinks.reverse()

        let directoryListEl = document.createElement('div');

        for (var link of directoryLinks) {
            directoryListEl.appendChild(link);
            
            // root node
            let spanSplitEl = document.createElement('span')
            spanSplitEl.innerText = ' \/ '
            directoryListEl.appendChild(spanSplitEl)
        }

        let self = this;
        this.el.appendChild(
            new Layout({
                start: [directoryListEl],
                end: [
                    Input.checkbox({
                        name: "filter-out-not-in-view",
                        tooltip: { content: "Mask data not in the view", position: {direction: "bottom"} },
                        checked: this.onlyInView,
                        click(e) {
                            self.onlyInView = e.target.checked;

                            self._createDOM(self.curNode)
                        },
                    })
                ]
            }, {classList: 'aladin-directory-path'}).element()
        )

        let listElt = document.createElement('ul');

        let seekMinEmInNode = (n) => {
            const isLeaf = typeof n === 'object' && 'ID' in n;

            if (isLeaf) {
                return (+n.em_min)
            } else if (typeof n === 'object') {
                let min = Number.POSITIVE_INFINITY;

                for (var [key, child] of Object.entries(n)) {
                    if (key === "parent" || key === "label") {
                        continue;
                    }

                    let val = seekMinEmInNode(child);
                    if (Number.isFinite(val)) {
                        min = Math.min(val, min);
                    }
                }

                return min;
            } else {
                return Number.POSITIVE_INFINITY;
            }
        };

        let labels = Object.keys(node).sort((la, lb) => {
            let na = node[la];
            let nb = node[lb];

            let aIsLeaf = typeof na === "object" && 'ID' in na;
            let bIsLeaf = typeof nb === "object" && 'ID' in nb;

            if (aIsLeaf !== bIsLeaf) {
                return aIsLeaf - bIsLeaf;
            } else if (typeof na === "object" && typeof nb === "object") {
                let emNa = seekMinEmInNode(na)
                let emNb = seekMinEmInNode(nb)

                if (emNa > emNb) {
                    return -1;
                } else {
                    return 1;
                }
            } else if (la < lb) {
                return -1
            } else {
                return 1
            }
        });

        let noEltsListed = true;
        for (const label of labels) {
            if (label !== 'parent' && label !== "label") {
                let elt = document.createElement('li');
                // points towards the parent node
                let child = node[label];
                let isLeaf = typeof child === "object" && 'ID' in child;
                if (isLeaf) {
                    if(this.params && this.filter && !this.filter(child, this.params)) {
                        elt.style.display = "none";
                    } else {
                        elt.style.display = "";
                        noEltsListed = false;
                    }

                    let label = this.label(child);
                    let layout = {start: [label], end: []};

                    if (child.dataproduct_subtype === "color") {
                        layout.end.push(new Icon({
                            size: "small",
                            url: Icon.dataURLFromSVG({ svg: Icon.SVG_ICONS.COLOR }),
                        }))
                    }

                    layout.end.push(ActionButton.BUTTONS(this.aladin)
                        .infoHiPS({
                            url: child.hips_service_url,
                            tooltip: {
                                aladin: this.aladin,
                                global: true,
                                content: "More info on the survey ?",
                            },
                        }).element()
                    )

                    layout.end = layout.end.concat([
                        ActionButton.BUTTONS(this.aladin)
                            .targetHiPSLocation({
                                ra: child.hips_initial_ra,
                                dec: child.hips_initial_dec,
                                fov: child.hips_initial_fov,
                                tooltip: {
                                    aladin: this.aladin,
                                    global: true,
                                    content: "Move to an interesting location",
                                },
                            })
                            .element(),
                        ActionButton.BUTTONS(this.aladin)
                            .addMOC({
                                name: label,
                                url: child.hips_service_url + '/Moc.fits',
                                tooltip: {
                                    aladin: this.aladin,
                                    global: true,
                                    content: "Click to add its coverage",
                                },
                            })
                            .element(),
                    ])

                    let childElt = new Layout(
                        layout,
                        {
                            vertical: false,
                            tooltip: {
                                content: '<figure class="aladin-fig"><img ' + 
                                    `src="${child.hips_service_url + "/preview.jpg"}"` +
                                    `alt="${label}" />` +
                                    `<figcaption>${label}</figcaption>` +
                                    '</figure>',
                                mouse: true,
                                aladin: this.aladin,
                            }
                        }).element();

                    if (this.highlight) {
                        if(this.highlight.includes(child.ID)) {
                            childElt.classList.add("aladin-valid");
                            childElt.classList.remove("aladin-not-found");
                        } else {
                            childElt.classList.remove("aladin-valid");
                            childElt.classList.add("aladin-not-found");

                            if (this.onlyInView) {
                                childElt.style.display = "none";
                            } else {
                                childElt.style.display = "";
                            }
                        }
                    }
                    elt.appendChild(childElt);
                } else {
                    // we see a parent, we must determine:
                    // * its color: he has at least 1 child inside the FoV => green
                    // * the number of children matching the filter params
                    let numFilteringMatching = this.numChildMatchingFilter(child, true);
                    let numTotal = this.numChildMatchingFilter(child, false);

                    let name = label;
                    elt.appendChild(Layout.horizontal([
                        new Icon({
                            size: "small",
                            monochrome: true,
                            url: folderIconUrl,
                        }),
                        name + ` (${numFilteringMatching}/${numTotal})`
                    ]).element())

                    if (numFilteringMatching == 0) {
                        elt.style.display = "none";
                    } else {
                        elt.style.display = "";
                        noEltsListed = false;
                    }
                }

                if(this.hasChildLocatedInFov(child)) {
                    elt.classList.add("aladin-valid");
                    elt.classList.remove("aladin-not-found");
                } else {
                    elt.classList.remove("aladin-valid");
                    elt.classList.add("aladin-not-found");

                    if (this.onlyInView) {
                        elt.style.display = "none";
                    }
                }

                child.label = label;
                child.parent = node;
                
                elt.for = label;
                elt.classList.add("aladin-link");

                elt.addEventListener('click', (e) => {
                    if (isLeaf) {
                        this.click(child)
                    } else {
                        // not leaf
                        this._createDOM(child);
                    }
                })
                elt.addEventListener('dblclick', (e) => {
                    if (isLeaf) {
                        this.dblclick(child)
                    }
                })

                listElt.appendChild(elt)
            }
        }

        if (noEltsListed && this.curNode !== this.root) {
            this.navigate(1)
        } else {
            this.el.appendChild(listElt);
        }
    }

    setHierarchy(root) {
        this._setRoot(root)
    }

    navigate(numOfLevels) {
        let curNode = this.curNode;
        while (curNode && curNode.parent && numOfLevels >= 1) {
            numOfLevels -= 1;
            curNode = curNode.parent;
        }

        this._createDOM(curNode)
    }

    highlightNodes(highlight) {
        this.highlight = highlight

        this._createDOM(this.curNode);
    }

    // Set params to null, undefined or {} to disable the filtering
    triggerFilter(params) {
        if (params && params.title) {
            params.title = params.title.toLowerCase()
        }

        this.params = params;

        this._createDOM(this.curNode);
    }

    hasChildLocatedInFov(node) {
        if (!this.highlight) {
            return false;
        }

        if (typeof node !== "object") {
            return false;
        }

        let isLeaf = typeof node === "object" && 'ID' in node;
        if (isLeaf) {
            if (this.highlight.includes(node.ID) && this.params && this.filter && this.filter(node, this.params)) {
                return true;
            }
        } else {
            for (const label of Object.keys(node)) {
                if (label === "parent" || label === "label")
                    continue;

                let child = node[label];
                if (child && this.hasChildLocatedInFov(child)) {
                    return true;
                }
            }
        }

        return false;
    }

    numChildMatchingFilter(node, filtering) {
        if (typeof node !== "object") {
            return 0;
        }

        let isLeaf = typeof node === "object" && 'ID' in node;
        if (isLeaf) {
            if (!filtering || (this.params && this.filter && this.filter(node, this.params))) {
                return 1;
            } else {
                return 0;
            }
        } else {
            let num = 0;
            for (const label of Object.keys(node)) {
                if (label === "parent")
                    continue;

                let child = node[label];
                
                if (child) {
                    num += this.numChildMatchingFilter(child, filtering);
                }
            }

            return num;
        }
    }
}
