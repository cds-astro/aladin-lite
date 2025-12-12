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

        for (const label of Object.keys(node).sort()) {
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

                    elt.innerHTML = this.label(child);
                } else {
                    // we see a parent, we must determine:
                    // * its color: he has at least 1 child inside the FoV => green
                    // * the number of children matching the filter params
                    let numFilteringMatching = this.numChildMatchingFilter(child, true);
                    let numTotal = this.numChildMatchingFilter(child, false);

                    let name = label;
                    elt.innerHTML = name + ` (${numFilteringMatching}/${numTotal})`

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

        for (const label of Object.keys(this.curNode).sort()) {
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
                    elt.innerHTML = name[0] + ` (${numFilteringMatching}/${numTotal})`

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

        for (const label of Object.keys(this.curNode).sort()) {
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
                    elt.innerHTML = name[0] + ` (${numFilteringMatching}/${numTotal})`
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
            for (const label of Object.keys(node).sort()) {
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
            for (const label of Object.keys(node).sort()) {
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
