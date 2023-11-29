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

export let Utils = {}
/**
 * Append el to target
 *
 * target must be an DOM Element/Node
 *
 * @API
 *
 * @param el: el can be a Widget or Element object. Otherwise it is considered as text
 * @param target: target must be an DOM Element/Node
 *
 */
 Utils.binarySearch = function(array, value) {
    var low = 0,
        high = array.length;

    while (low < high) {
        var mid = (low + high) >>> 1;
        if (array[mid] < value) low = mid + 1;
        else high = mid;
    }
    return low;
}