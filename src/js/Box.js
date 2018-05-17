// Copyright 2013-2017 - UDS/CNRS
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
 * File Box
 *
 * A Box instance is a GUI element providing a div nested
 * in Aladin Lite parent div
 * 
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/
Box = (function() {

    // constructor
    var Box = function(properties) {

        this.$parentDiv = $('<div>');
        this.$parentDiv.addClass('aladin-box');

        properties = properties || {};

        this.css = properties.css || {padding: '4px'};

        this.position = properties.position || 'bottom'; // position can be bottom, left, top or right
        if (this.position=='right') {
            this.css['left'] = 'unset';
        }
        this.css[this.position] = '4px';

        this.contentCss = properties.contentCss || {};

        this.title = properties.title || undefined;

        this.content = properties.content || undefined;

        this.showHandler = properties.showHandler !== undefined ? properties.showHandler : true;

        this.openCallback = properties.openCallback || undefined; // callback called when the user opens the panel
        this.closeCallback = properties.closeCallback || undefined; // callback called when the user closes the panel

        this.changingDim = 'width';
        if (this.position=='top' || this.position=='bottom') {
            this.changingDim = 'height';
        }


        this.open = false;
        this._render();
        this.$parentDiv.show();
        this.open = true;
        this.hide();
    };

    Box.prototype = {

        show: function() {
            if (this.open) {
                return;
            }

            this.open = true;
            this.$parentDiv.show();
            this._updateChevron();

            if (this.changingDim=='width') {
                this.$parentDiv.find('.aladin-box-title-label').show();
            }
            var self = this;
            var options = {};
            options[this.changingDim] = 'show';
            var delay = this.changingDim=='width' ? 0 : 400;
            this.$parentDiv.find('.aladin-box-content').animate(options, delay, function() {
                self.css[self.position] = '4px';
                self.updateStyle(self.css);

                typeof self.openCallback === 'function' && self.openCallback();
            });

        },

        hide: function() {
            if (! this.open) {
                return;
            }

            this.open = false;
            this._updateChevron();

            if (this.changingDim=='width') {
                this.$parentDiv.find('.aladin-box-title-label').hide();
            }
            var self = this;
            var options = {};
            options[this.changingDim] = 'hide';
            var delay = this.changingDim=='width' ? 0 : 400;
            this.$parentDiv.find('.aladin-box-content').animate(options, delay, function() {
                self.css[self.position] = '0px';
                self.updateStyle(self.css);

                typeof self.closeCallback === 'function' && self.closeCallback();
            });
        },

        // complety hide parent div
        realHide: function() {
            this.open = false;
            this.$parentDiv.hide();
        },

        updateStyle: function(css) {
            this.css = css;
            this.$parentDiv.css(css);
        },

        setContent: function(content) {
            this.content = content;
            this._render();
        },

        setTitle: function(title) {
            this.title = title;
            this._render();
        },

        enable: function() {
            this.$parentDiv.enable();
        },

        disable: function() {
            this.$parentDiv.disable();
        },

        // fill $parentDiv with HTML corresponding to current state
        _render: function() {
            var self = this;

            this.$parentDiv.empty();
            this.$parentDiv.off();

            var titleDiv = $('<div class="aladin-box-title">');
            if (this.showHandler) {
                var chevron = $('<span class="aladin-chevron">');
                titleDiv.append(chevron);
            }
            if (this.title) {
                titleDiv.append(' <span class="aladin-box-title-label">' + this.title + '</span>');
            }
            this.$parentDiv.append(titleDiv);
            var $content = $('<div class="aladin-box-content">' + (this.content?this.content:'') + '</div>');
            $content.css(this.contentCss);
            this.$parentDiv.append($content);

            this._updateChevron();
            this.updateStyle(this.css);

            titleDiv.on('click', function() {
                if (self.open) {
                    self.hide();
                }
                else {
                    self.show();
                }
            });
        },

        _updateChevron: function() {
            this.$parentDiv.find('.aladin-chevron').removeClass().addClass('aladin-chevron ' + getChevronClass(this.position, this.open))
                                                        .attr('title', 'Click to ' + (this.open?'hide ':'show ') + (this.title?this.title:'') + ' panel');
        }
    };

    // return the jquery object corresponding to the given position and open/close state
    var getChevronClass = function(position, isOpen) {
        if (position=='top' && isOpen || position=='bottom' && !isOpen) {
            return 'aladin-chevron-up';
        }
        if (position=='bottom' && isOpen || position=='top' && !isOpen) {
            return 'aladin-chevron-down';
        }
        if (position=='right' && isOpen || position=='left' && !isOpen) {
            return 'aladin-chevron-right';
        }
        if (position=='left' && isOpen || position=='right' && !isOpen) {
            return 'aladin-chevron-left';
        }
        return '';
    };

    


    return Box;

})();

