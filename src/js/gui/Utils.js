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

Element.prototype.insertChildAtIndex = function(child, index) {
    if (index >= this.children.length) {
        this.appendChild(child)
    } else {
        this.insertBefore(child, this.children[index])
    }
};

Element.prototype.swap = function (node) {
    const parent = this.parentNode;
    const sibling = this.nextSibling === node ? this : this.nextSibling;

    // Move `this` to before the `node`
    node.parentNode.insertBefore(this, node);

    // Move `node` to before the sibling of `this`
    parent.insertBefore(node, sibling);
};

export function isJSObject(obj) {
    return obj !== null &&
        typeof obj === "object" &&
        obj.constructor === Object
}