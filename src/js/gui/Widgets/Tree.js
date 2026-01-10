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

import { DOMElement } from "./Widget";
import { Icon } from "./Icon";
import folderIconUrl from "../../../../assets/icons/folder.svg";

import { Layout } from "../Layout";
import { ActionButton } from "./ActionButton";
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
        this.aladin = options && options.aladin;

        let rootNode = options && options.root || {};
        this.params = null;
        this.filter = options && options.filter;
        this.label = options && options.label;

        this._setRoot(rootNode);

        this.attachTo(target, position);
        this._show();
        this.addClass('aladin-dark-theme')
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
        directoryListEl.classList.add('aladin-directory-path');
        directoryListEl.style.display = "inline-block"

        for (var link of directoryLinks) {
            directoryListEl.appendChild(link);
            
            // root node
            let spanSplitEl = document.createElement('span')
            spanSplitEl.innerText = ' \/ '
            directoryListEl.appendChild(spanSplitEl)
        }

        this.el.appendChild(directoryListEl)

        let listElt = document.createElement('ul');

        let labels = Object.keys(node).sort((la, lb) => {
            let na = node[la];
            let nb = node[lb];

            let aIsLeaf = typeof na === "object" && 'ID' in na;
            let bIsLeaf = typeof nb === "object" && 'ID' in nb;

            if (aIsLeaf !== bIsLeaf) {
                return aIsLeaf - bIsLeaf;
            } else if (la < lb) {
                return -1
            } else {
                return 1;
            }
        });
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
                        elt.style.display = "block";
                    }

                    let label = this.label(child);

                    let layout = [label];

                    if (child.dataproduct_subtype === "color") {
                        layout.push(new Icon({
                            size: "small",
                            url: Icon.dataURLFromSVG({ svg: Icon.SVG_ICONS.COLOR }),
                        }))
                    }

                    layout.push(ActionButton.BUTTONS(this.aladin)
                        .infoHiPS({
                            url: child.hips_service_url,
                            tooltip: {
                                aladin: this.aladin,
                                global: true,
                                content: "More info on the survey ?",
                            },
                        }).element()
                    )

                    layout = layout.concat([
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

                    let config = {
                        layout,
                        tooltip: {
                            content: '<figure class="aladin-fig"><img ' + 
                                `src="${child.hips_service_url + "/preview.jpg"}"` +
                                `alt="${label}" />` +
                                `<figcaption>${label}</figcaption>` +
                                '</figure>',
                            delayShowUpTime: "100ms",
                            mouse: true,
                            aladin: this.aladin,
                        }
                    };

                    elt.appendChild(Layout.horizontal(config).element());
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
                        elt.style.display = "block";
                    }
                }

                elt.style.color = this.hasChildLocatedInFov(child) ? 'yellowgreen' : 'orange';

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

                listElt.appendChild(elt)
            }
        }

        this.el.appendChild(listElt);
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

        let elts = this.el.querySelectorAll("li");
        let i = 0;

        let labels = Object.keys(this.curNode).sort((la, lb) => {
            let na = this.curNode[la];
            let nb = this.curNode[lb];

            let aIsLeaf = typeof na === "object" && 'ID' in na;
            let bIsLeaf = typeof nb === "object" && 'ID' in nb;

            if (aIsLeaf !== bIsLeaf) {
                return aIsLeaf - bIsLeaf;
            } else if (la < lb) {
                return -1
            } else {
                return 1;
            }
        });

        for (const label of labels) {
            if (label !== 'parent' && label !== "label") {
                let elt = elts[i];
                i += 1;
                // points towards the parent node
                let child = this.curNode[label];
                let isLeaf = typeof child === "object" && 'ID' in child;
                if (isLeaf) {
                    // Check if its ID is found in the view
                    if (this.highlight) {
                        elt.style.color = this.highlight.includes(child.ID) ? 'yellowgreen' : 'orange';
                    }
                } else {
                    // we see a parent, we must determine:
                    // * its color: he has at least 1 child inside the FoV => green
                    // * the number of children matching the filter params
                    elt.style.color = this.hasChildLocatedInFov(child) ? 'yellowgreen' : 'orange';

                    // we see a parent, we must determine:
                    // * its color: he has at least 1 child inside the FoV => green
                    // * the number of children matching the filter params
                    let numFilteringMatching = this.numChildMatchingFilter(child, true);
                    let numTotal = this.numChildMatchingFilter(child, false);

                    let name = elt.innerText.split('(');

                    elt.innerHTML = Layout.horizontal([
                        new Icon({
                            size: "small",
                            monochrome: true,
                            url: folderIconUrl,
                        }),
                        name[0] + ` (${numFilteringMatching}/${numTotal})`
                    ]).element().outerHTML

                    if (numFilteringMatching == 0) {
                        elt.style.display = "none";
                    } else {
                        elt.style.display = "block";
                    }
                }
            }
        }
    }

    // Set params to null, undefined or {} to disable the filtering
    triggerFilter(params) {
        if (params && params.title) {
            params.title = params.title.toLowerCase()
        }

        this.params = params;

        let elts = this.el.querySelectorAll("li");
        let i = 0;

        let labels = Object.keys(this.curNode).sort((la, lb) => {
            let na = this.curNode[la];
            let nb = this.curNode[lb];

            let aIsLeaf = typeof na === "object" && 'ID' in na;
            let bIsLeaf = typeof nb === "object" && 'ID' in nb;

            if (aIsLeaf !== bIsLeaf) {
                return aIsLeaf - bIsLeaf;
            } else if (la < lb) {
                return -1
            } else {
                return 1;
            }
        });
        for (const label of labels) {
            if (label !== 'parent' && label !== "label") {
                let elt = elts[i];
                i += 1;
                // points towards the parent node
                let child = this.curNode[label];
                let isLeaf = typeof child === "object" && 'ID' in child;
                if (isLeaf) {
                    if(this.params && this.filter && !this.filter(child, this.params)) {
                        elt.style.display = "none"
                    } else {
                        elt.style.display = "block"
                    }
                } else {
                    // we see a parent, we must determine:
                    // * its color: he has at least 1 child inside the FoV => green
                    // * the number of children matching the filter params
                    let numFilteringMatching = this.numChildMatchingFilter(child, true);
                    let numTotal = this.numChildMatchingFilter(child, false);

                    let name = elt.innerText.split('(');
                    elt.innerHTML = Layout.horizontal([
                        new Icon({
                            size: "small",
                            monochrome: true,
                            url: folderIconUrl,
                        }),
                        name[0] + ` (${numFilteringMatching}/${numTotal})`
                    ]).element().outerHTML;

                    if (numFilteringMatching == 0) {
                        elt.style.display = "none";
                    } else {
                        elt.style.display = "block";
                    }
                }
            }
        }
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
            if (this.highlight.includes(node.ID)) {
                return true;
            }
        } else {
            let labels = Object.keys(node).sort((la, lb) => {
                let na = node[la];
                let nb = node[lb];

                let aIsLeaf = typeof na === "object" && 'ID' in na;
                let bIsLeaf = typeof nb === "object" && 'ID' in nb;

                if (aIsLeaf !== bIsLeaf) {
                    return aIsLeaf - bIsLeaf;
                } else if (la < lb) {
                    return -1
                } else {
                    return 1;
                }
            });
            for (const label of labels) {
                if (label === "parent")
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
            let labels = Object.keys(node).sort((la, lb) => {
                let na = node[la];
                let nb = node[lb];

                let aIsLeaf = typeof na === "object" && 'ID' in na;
                let bIsLeaf = typeof nb === "object" && 'ID' in nb;

                if (aIsLeaf !== bIsLeaf) {
                    return aIsLeaf - bIsLeaf;
                } else if (la < lb) {
                    return -1
                } else {
                    return 1;
                }
            });
            for (const label of labels) {
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
