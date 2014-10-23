/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Utils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Utils = Utils || {};

// adding relMouseCoords to HTMLCanvasElement prototype (see http://stackoverflow.com/questions/55677/how-do-i-get-the-coordinates-of-a-mouse-click-on-a-canvas-element ) 
function relMouseCoords(event){
    var totalOffsetX = 0;
    var totalOffsetY = 0;
    var canvasX = 0;
    var canvasY = 0;
    var currentElement = this;

    do {
        totalOffsetX += currentElement.offsetLeft - currentElement.scrollLeft;
        totalOffsetY += currentElement.offsetTop - currentElement.scrollTop;
    }
    while(currentElement = currentElement.offsetParent)

    if (event.pageX) {
        canvasX = event.pageX - totalOffsetX - document.body.scrollLeft;
        canvasY = event.pageY - totalOffsetY - document.body.scrollTop;
    }
    // if touch events
    else {
        canvasX = event.originalEvent.targetTouches[0].screenX - totalOffsetX - document.body.scrollLeft;
        canvasY = event.originalEvent.targetTouches[0].screenY - totalOffsetY - document.body.scrollTop;    	
    }

    return {x:canvasX, y:canvasY};
}
HTMLCanvasElement.prototype.relMouseCoords = relMouseCoords;


/* source : http://stackoverflow.com/a/8764051 */
$.urlParam = function(name){
	return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(location.search)||[,""])[1].replace(/\+/g, '%20'))||null;
};



