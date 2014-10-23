/******************************************************************************
 * Aladin Lite project
 * 
 * File Popup.js
 * 
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/

Popup = (function() {
    
    
    // constructor
    Popup = function(parentDiv) {
        this.domEl = $('<div class="aladin-popup-container"><div class="aladin-popup"><a class="aladin-closeBtn">&times;</a><div class="aladin-popupTitle"></div><div class="aladin-popupText"></div></div><div class="aladin-popup-arrow"></div></div>');
        this.domEl.appendTo(parentDiv);

        var self = this;
        // close popup
        this.domEl.find('.aladin-closeBtn').click(function() {self.hide()});
        
    };
    
    Popup.prototype.hide = function() {
        this.domEl.hide();
    };

    Popup.prototype.show = function() {
        this.domEl.show();
    };

    Popup.prototype.setTitle = function(title) {
        this.domEl.find('.aladin-popupTitle').html(title);
    };

    Popup.prototype.setText = function(text) {
        this.domEl.find('.aladin-popupText').html(text);
        this.w = this.domEl.outerWidth();
        this.h = this.domEl.outerHeight();
    };

    Popup.prototype.setSource = function(source) {
        // remove reference to popup for previous source
        if (this.source) {
            this.source.popup = null;
        }
        source.popup = this;
        this.source = source;
        this.setPosition(source.x, source.y);
    };

    Popup.prototype.setPosition = function(x, y) {
        var newX = x - this.w/2;
        var newY = y - this.h + this.source.catalog.sourceSize/2;
        this.domEl[0].style.left = newX + 'px';
        this.domEl[0].style.top  = newY + 'px';
        //this.domEl.css({'left': newX+'px', 'top': newY+'px'});
    };
    
    return Popup;
})();
    
