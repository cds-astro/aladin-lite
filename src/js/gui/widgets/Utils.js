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
