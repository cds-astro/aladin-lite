// Copyright 2013 - UDS/CNRS
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
 * File Color
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
export let Color = (function() {
    let Color = function(label) {
      if (label.r && label.g && label.b) {
        return label;
      }
      
      // hex string form
      if (label.match(/^#?[0-9A-Fa-f]{6}$/) || label.match(/^#?[0-9A-Fa-f]{3}$/)) {
        // if it is hexadecimal already, return it in its actual form
        return Color.hexToRgb(label);
      }

      // rgb string form
      var rgb = label.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/);
     
      if (rgb) {
        var r = parseInt(rgb[1]);
        var g = parseInt(rgb[2]);
        var b = parseInt(rgb[3]);
        return {r: r, g: g, b: b}
      }
      
      // fill style label
      if (!Color.standardizedColors[label]) {
        var ctx = document.createElement('canvas').getContext('2d');
        ctx.fillStyle = label;
        const colorHexa = ctx.fillStyle;

        Color.standardizedColors[label] = colorHexa;
      }

      return Color.standardizedColors[label];
    };

    Color.curIdx = 0;
    Color.colors = ['#ff0000', '#0000ff', '#99cc00', '#ffff00','#000066', '#00ffff', '#9900cc', '#0099cc', '#cc9900', '#cc0099', '#00cc99', '#663333', '#ffcc9a', '#ff9acc', '#ccff33', '#660000', '#ffcc33', '#ff00ff', '#00ff00', '#ffffff'];
    Color.standardizedColors = {};

    Color.getNextColor = function() {
        var c = Color.colors[Color.curIdx % (Color.colors.length)];
        Color.curIdx++;
        return c;
    };

    /** return most suited (ie readable) color for a label, given a background color
     * bkgdColor: color, given as a 'rgb(<r value>, <g value>, <v value>)' . This is returned by $(<element>).css('background-color')
     * 
     * example call: Color.getLabelColorForBackground('rgb(3, 123, 42)')
     * adapted from http://stackoverflow.com/questions/1855884/determine-font-color-based-on-background-color
     */
    Color.getLabelColorForBackground = function(rgbBkgdColor) {
        var lightLabel = '#eee' 
        var darkLabel = '#111' 
        var rgb = rgbBkgdColor.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/);
        if (rgb==null) {
            // we return the dark label color if we can't parse the color
            return darkLabel
        }
        var r = parseInt(rgb[1]);
        var g = parseInt(rgb[2]);
        var b = parseInt(rgb[3]);
        
        // Counting the perceptive luminance - human eye favors green color... 
        var a = 1 - ( 0.299 * r + 0.587 * g + 0.114 * b) / 255;

        if (a < 0.5) {
            return darkLabel; // bright color --> dark font
        }
        else {
            return lightLabel; // dark color --> light font
        }
    };

    // convert hexadecimal string to r, g, b components
    Color.hexToRgb = function(hex) {
        // Expand shorthand form (e.g. "03F") to full form (e.g. "0033FF")
        var shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
        hex = hex.replace(shorthandRegex, function(m, r, g, b) {
          return r + r + g + g + b + b;
        });
      
        var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? {
          r: parseInt(result[1], 16),
          g: parseInt(result[2], 16),
          b: parseInt(result[3], 16)
        } : null;
    };

    Color.hexToRgba = function(hex) {
        // Expand shorthand form (e.g. "03AF") to full form (e.g. "0033AAFF")
        var shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])([a-f\d])$/i;
        hex = hex.replace(shorthandRegex, function(m, r, g, b, a) {
          return r + r + g + g + b + b + a + a;
        });
      
        var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? {
          r: parseInt(result[1], 16),
          g: parseInt(result[2], 16),
          b: parseInt(result[3], 16),
          a: parseInt(result[4], 16)
        } : null;
    };

    Color.rgbToHex = function (r, g, b) {
        return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
    }

    Color.rgbaToHex = function (r, g, b, a) {
        r = r.toString(16);
        g = g.toString(16);
        b = b.toString(16);
        a = a.toString(16);
      
        if (r.length == 1)
          r = "0" + r;
        if (g.length == 1)
          g = "0" + g;
        if (b.length == 1)
          b = "0" + b;
        if (a.length == 1)
          a = "0" + a;
      
        return "#" + r + g + b + a;
    }

    Color.standardizeColor = function(label) {
        if (label.match(/^#?[0-9A-Fa-f]{6}$/) || label.match(/^#?[0-9A-Fa-f]{3}$/)) {
            // if it is hexadecimal already, return it in its actual form
            return label;
        } else {
            if (!Color.standardizedColors[label]) {
                var ctx = document.createElement('canvas').getContext('2d');
                ctx.fillStyle = label;
                const colorHexa = ctx.fillStyle;
        
                Color.standardizedColors[label] = colorHexa;
            }
            return Color.standardizedColors[label];
        }
    }

    return Color;
})();

