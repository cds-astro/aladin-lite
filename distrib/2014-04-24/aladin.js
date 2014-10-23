// cds namespace

var cds = cds || {};

var A = A || {};
/*
    json2.js
    2012-10-08

    Public Domain.

    NO WARRANTY EXPRESSED OR IMPLIED. USE AT YOUR OWN RISK.

    See http://www.JSON.org/js.html


    This code should be minified before deployment.
    See http://javascript.crockford.com/jsmin.html

    USE YOUR OWN COPY. IT IS EXTREMELY UNWISE TO LOAD CODE FROM SERVERS YOU DO
    NOT CONTROL.


    This file creates a global JSON object containing two methods: stringify
    and parse.

        JSON.stringify(value, replacer, space)
            value       any JavaScript value, usually an object or array.

            replacer    an optional parameter that determines how object
                        values are stringified for objects. It can be a
                        function or an array of strings.

            space       an optional parameter that specifies the indentation
                        of nested structures. If it is omitted, the text will
                        be packed without extra whitespace. If it is a number,
                        it will specify the number of spaces to indent at each
                        level. If it is a string (such as '\t' or '&nbsp;'),
                        it contains the characters used to indent at each level.

            This method produces a JSON text from a JavaScript value.

            When an object value is found, if the object contains a toJSON
            method, its toJSON method will be called and the result will be
            stringified. A toJSON method does not serialize: it returns the
            value represented by the name/value pair that should be serialized,
            or undefined if nothing should be serialized. The toJSON method
            will be passed the key associated with the value, and this will be
            bound to the value

            For example, this would serialize Dates as ISO strings.

                Date.prototype.toJSON = function (key) {
                    function f(n) {
                        // Format integers to have at least two digits.
                        return n < 10 ? '0' + n : n;
                    }

                    return this.getUTCFullYear()   + '-' +
                         f(this.getUTCMonth() + 1) + '-' +
                         f(this.getUTCDate())      + 'T' +
                         f(this.getUTCHours())     + ':' +
                         f(this.getUTCMinutes())   + ':' +
                         f(this.getUTCSeconds())   + 'Z';
                };

            You can provide an optional replacer method. It will be passed the
            key and value of each member, with this bound to the containing
            object. The value that is returned from your method will be
            serialized. If your method returns undefined, then the member will
            be excluded from the serialization.

            If the replacer parameter is an array of strings, then it will be
            used to select the members to be serialized. It filters the results
            such that only members with keys listed in the replacer array are
            stringified.

            Values that do not have JSON representations, such as undefined or
            functions, will not be serialized. Such values in objects will be
            dropped; in arrays they will be replaced with null. You can use
            a replacer function to replace those with JSON values.
            JSON.stringify(undefined) returns undefined.

            The optional space parameter produces a stringification of the
            value that is filled with line breaks and indentation to make it
            easier to read.

            If the space parameter is a non-empty string, then that string will
            be used for indentation. If the space parameter is a number, then
            the indentation will be that many spaces.

            Example:

            text = JSON.stringify(['e', {pluribus: 'unum'}]);
            // text is '["e",{"pluribus":"unum"}]'


            text = JSON.stringify(['e', {pluribus: 'unum'}], null, '\t');
            // text is '[\n\t"e",\n\t{\n\t\t"pluribus": "unum"\n\t}\n]'

            text = JSON.stringify([new Date()], function (key, value) {
                return this[key] instanceof Date ?
                    'Date(' + this[key] + ')' : value;
            });
            // text is '["Date(---current time---)"]'


        JSON.parse(text, reviver)
            This method parses a JSON text to produce an object or array.
            It can throw a SyntaxError exception.

            The optional reviver parameter is a function that can filter and
            transform the results. It receives each of the keys and values,
            and its return value is used instead of the original value.
            If it returns what it received, then the structure is not modified.
            If it returns undefined then the member is deleted.

            Example:

            // Parse the text. Values that look like ISO date strings will
            // be converted to Date objects.

            myData = JSON.parse(text, function (key, value) {
                var a;
                if (typeof value === 'string') {
                    a =
/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2}(?:\.\d*)?)Z$/.exec(value);
                    if (a) {
                        return new Date(Date.UTC(+a[1], +a[2] - 1, +a[3], +a[4],
                            +a[5], +a[6]));
                    }
                }
                return value;
            });

            myData = JSON.parse('["Date(09/09/2001)"]', function (key, value) {
                var d;
                if (typeof value === 'string' &&
                        value.slice(0, 5) === 'Date(' &&
                        value.slice(-1) === ')') {
                    d = new Date(value.slice(5, -1));
                    if (d) {
                        return d;
                    }
                }
                return value;
            });


    This is a reference implementation. You are free to copy, modify, or
    redistribute.
*/

/*jslint evil: true, regexp: true */

/*members "", "\b", "\t", "\n", "\f", "\r", "\"", JSON, "\\", apply,
    call, charCodeAt, getUTCDate, getUTCFullYear, getUTCHours,
    getUTCMinutes, getUTCMonth, getUTCSeconds, hasOwnProperty, join,
    lastIndex, length, parse, prototype, push, replace, slice, stringify,
    test, toJSON, toString, valueOf
*/


// Create a JSON object only if one does not already exist. We create the
// methods in a closure to avoid creating global variables.

if (typeof JSON !== 'object') {
    JSON = {};
}

(function () {
    'use strict';

    function f(n) {
        // Format integers to have at least two digits.
        return n < 10 ? '0' + n : n;
    }

    if (typeof Date.prototype.toJSON !== 'function') {

        Date.prototype.toJSON = function (key) {

            return isFinite(this.valueOf())
                ? this.getUTCFullYear()     + '-' +
                    f(this.getUTCMonth() + 1) + '-' +
                    f(this.getUTCDate())      + 'T' +
                    f(this.getUTCHours())     + ':' +
                    f(this.getUTCMinutes())   + ':' +
                    f(this.getUTCSeconds())   + 'Z'
                : null;
        };

        String.prototype.toJSON      =
            Number.prototype.toJSON  =
            Boolean.prototype.toJSON = function (key) {
                return this.valueOf();
            };
    }

    var cx = /[\u0000\u00ad\u0600-\u0604\u070f\u17b4\u17b5\u200c-\u200f\u2028-\u202f\u2060-\u206f\ufeff\ufff0-\uffff]/g,
        escapable = /[\\\"\x00-\x1f\x7f-\x9f\u00ad\u0600-\u0604\u070f\u17b4\u17b5\u200c-\u200f\u2028-\u202f\u2060-\u206f\ufeff\ufff0-\uffff]/g,
        gap,
        indent,
        meta = {    // table of character substitutions
            '\b': '\\b',
            '\t': '\\t',
            '\n': '\\n',
            '\f': '\\f',
            '\r': '\\r',
            '"' : '\\"',
            '\\': '\\\\'
        },
        rep;


    function quote(string) {

// If the string contains no control characters, no quote characters, and no
// backslash characters, then we can safely slap some quotes around it.
// Otherwise we must also replace the offending characters with safe escape
// sequences.

        escapable.lastIndex = 0;
        return escapable.test(string) ? '"' + string.replace(escapable, function (a) {
            var c = meta[a];
            return typeof c === 'string'
                ? c
                : '\\u' + ('0000' + a.charCodeAt(0).toString(16)).slice(-4);
        }) + '"' : '"' + string + '"';
    }


    function str(key, holder) {

// Produce a string from holder[key].

        var i,          // The loop counter.
            k,          // The member key.
            v,          // The member value.
            length,
            mind = gap,
            partial,
            value = holder[key];

// If the value has a toJSON method, call it to obtain a replacement value.

        if (value && typeof value === 'object' &&
                typeof value.toJSON === 'function') {
            value = value.toJSON(key);
        }

// If we were called with a replacer function, then call the replacer to
// obtain a replacement value.

        if (typeof rep === 'function') {
            value = rep.call(holder, key, value);
        }

// What happens next depends on the value's type.

        switch (typeof value) {
        case 'string':
            return quote(value);

        case 'number':

// JSON numbers must be finite. Encode non-finite numbers as null.

            return isFinite(value) ? String(value) : 'null';

        case 'boolean':
        case 'null':

// If the value is a boolean or null, convert it to a string. Note:
// typeof null does not produce 'null'. The case is included here in
// the remote chance that this gets fixed someday.

            return String(value);

// If the type is 'object', we might be dealing with an object or an array or
// null.

        case 'object':

// Due to a specification blunder in ECMAScript, typeof null is 'object',
// so watch out for that case.

            if (!value) {
                return 'null';
            }

// Make an array to hold the partial results of stringifying this object value.

            gap += indent;
            partial = [];

// Is the value an array?

            if (Object.prototype.toString.apply(value) === '[object Array]') {

// The value is an array. Stringify every element. Use null as a placeholder
// for non-JSON values.

                length = value.length;
                for (i = 0; i < length; i += 1) {
                    partial[i] = str(i, value) || 'null';
                }

// Join all of the elements together, separated with commas, and wrap them in
// brackets.

                v = partial.length === 0
                    ? '[]'
                    : gap
                    ? '[\n' + gap + partial.join(',\n' + gap) + '\n' + mind + ']'
                    : '[' + partial.join(',') + ']';
                gap = mind;
                return v;
            }

// If the replacer is an array, use it to select the members to be stringified.

            if (rep && typeof rep === 'object') {
                length = rep.length;
                for (i = 0; i < length; i += 1) {
                    if (typeof rep[i] === 'string') {
                        k = rep[i];
                        v = str(k, value);
                        if (v) {
                            partial.push(quote(k) + (gap ? ': ' : ':') + v);
                        }
                    }
                }
            } else {

// Otherwise, iterate through all of the keys in the object.

                for (k in value) {
                    if (Object.prototype.hasOwnProperty.call(value, k)) {
                        v = str(k, value);
                        if (v) {
                            partial.push(quote(k) + (gap ? ': ' : ':') + v);
                        }
                    }
                }
            }

// Join all of the member texts together, separated with commas,
// and wrap them in braces.

            v = partial.length === 0
                ? '{}'
                : gap
                ? '{\n' + gap + partial.join(',\n' + gap) + '\n' + mind + '}'
                : '{' + partial.join(',') + '}';
            gap = mind;
            return v;
        }
    }

// If the JSON object does not yet have a stringify method, give it one.

    if (typeof JSON.stringify !== 'function') {
        JSON.stringify = function (value, replacer, space) {

// The stringify method takes a value and an optional replacer, and an optional
// space parameter, and returns a JSON text. The replacer can be a function
// that can replace values, or an array of strings that will select the keys.
// A default replacer method can be provided. Use of the space parameter can
// produce text that is more easily readable.

            var i;
            gap = '';
            indent = '';

// If the space parameter is a number, make an indent string containing that
// many spaces.

            if (typeof space === 'number') {
                for (i = 0; i < space; i += 1) {
                    indent += ' ';
                }

// If the space parameter is a string, it will be used as the indent string.

            } else if (typeof space === 'string') {
                indent = space;
            }

// If there is a replacer, it must be a function or an array.
// Otherwise, throw an error.

            rep = replacer;
            if (replacer && typeof replacer !== 'function' &&
                    (typeof replacer !== 'object' ||
                    typeof replacer.length !== 'number')) {
                throw new Error('JSON.stringify');
            }

// Make a fake root object containing our value under the key of ''.
// Return the result of stringifying the value.

            return str('', {'': value});
        };
    }


// If the JSON object does not yet have a parse method, give it one.

    if (typeof JSON.parse !== 'function') {
        JSON.parse = function (text, reviver) {

// The parse method takes a text and an optional reviver function, and returns
// a JavaScript value if the text is a valid JSON text.

            var j;

            function walk(holder, key) {

// The walk method is used to recursively walk the resulting structure so
// that modifications can be made.

                var k, v, value = holder[key];
                if (value && typeof value === 'object') {
                    for (k in value) {
                        if (Object.prototype.hasOwnProperty.call(value, k)) {
                            v = walk(value, k);
                            if (v !== undefined) {
                                value[k] = v;
                            } else {
                                delete value[k];
                            }
                        }
                    }
                }
                return reviver.call(holder, key, value);
            }


// Parsing happens in four stages. In the first stage, we replace certain
// Unicode characters with escape sequences. JavaScript handles many characters
// incorrectly, either silently deleting them, or treating them as line endings.

            text = String(text);
            cx.lastIndex = 0;
            if (cx.test(text)) {
                text = text.replace(cx, function (a) {
                    return '\\u' +
                        ('0000' + a.charCodeAt(0).toString(16)).slice(-4);
                });
            }

// In the second stage, we run the text against regular expressions that look
// for non-JSON patterns. We are especially concerned with '()' and 'new'
// because they can cause invocation, and '=' because it can cause mutation.
// But just to be safe, we want to reject all unexpected forms.

// We split the second stage into 4 regexp operations in order to work around
// crippling inefficiencies in IE's and Safari's regexp engines. First we
// replace the JSON backslash pairs with '@' (a non-JSON character). Second, we
// replace all simple value tokens with ']' characters. Third, we delete all
// open brackets that follow a colon or comma or that begin the text. Finally,
// we look to see that the remaining characters are only whitespace or ']' or
// ',' or ':' or '{' or '}'. If that is so, then the text is safe for eval.

            if (/^[\],:{}\s]*$/
                    .test(text.replace(/\\(?:["\\\/bfnrt]|u[0-9a-fA-F]{4})/g, '@')
                        .replace(/"[^"\\\n\r]*"|true|false|null|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/g, ']')
                        .replace(/(?:^|:|,)(?:\s*\[)+/g, ''))) {

// In the third stage we use the eval function to compile the text into a
// JavaScript structure. The '{' operator is subject to a syntactic ambiguity
// in JavaScript: it can begin a block or an object literal. We wrap the text
// in parens to eliminate the ambiguity.

                j = eval('(' + text + ')');

// In the optional fourth stage, we recursively walk the new structure, passing
// each name/value pair to a reviver function for possible transformation.

                return typeof reviver === 'function'
                    ? walk({'': j}, '')
                    : j;
            }

// If the text is not JSON parseable, then a SyntaxError is thrown.

            throw new SyntaxError('JSON.parse');
        };
    }
}());// log 
Logger = {};

Logger.log = function(action, params) {
    try {
        var logUrl = "http://alasky.u-strasbg.fr/cgi/AladinLiteLogger/log.py";
        var paramStr = "";
        if (params) {
            paramStr = JSON.stringify(params);
        }
        
        $.ajax({
            url: logUrl,
            data: {"action": action, "params": paramStr, "pageUrl": window.location.href, "referer": document.referrer ? document.referrer : ""},
            method: 'GET',
            dataType: 'json' // as alasky supports CORS, we do not need JSONP any longer
        });
        
    }
    catch(e) {
        window.console && console.log('Exception: ' + e);
    }

};
/*! Copyright (c) 2013 Brandon Aaron (http://brandon.aaron.sh)
 * Licensed under the MIT License (LICENSE.txt).
 *
 * Version: 3.1.4
 *
 * Requires: 1.2.2+
 */

(function (factory) {
    if ( typeof define === 'function' && define.amd ) {
        // AMD. Register as an anonymous module.
        define(['jquery'], factory);
    } else if (typeof exports === 'object') {
        // Node/CommonJS style for Browserify
        module.exports = factory;
    } else {
        // Browser globals
        factory(jQuery);
    }
}(function ($) {

    var toFix = ['wheel', 'mousewheel', 'DOMMouseScroll', 'MozMousePixelScroll'];
    var toBind = 'onwheel' in document || document.documentMode >= 9 ? ['wheel'] : ['mousewheel', 'DomMouseScroll', 'MozMousePixelScroll'];
    var lowestDelta, lowestDeltaXY;

    if ( $.event.fixHooks ) {
        for ( var i = toFix.length; i; ) {
            $.event.fixHooks[ toFix[--i] ] = $.event.mouseHooks;
        }
    }

    $.event.special.mousewheel = {
        setup: function() {
            if ( this.addEventListener ) {
                for ( var i = toBind.length; i; ) {
                    this.addEventListener( toBind[--i], handler, false );
                }
            } else {
                this.onmousewheel = handler;
            }
        },

        teardown: function() {
            if ( this.removeEventListener ) {
                for ( var i = toBind.length; i; ) {
                    this.removeEventListener( toBind[--i], handler, false );
                }
            } else {
                this.onmousewheel = null;
            }
        }
    };

    $.fn.extend({
        mousewheel: function(fn) {
            return fn ? this.bind('mousewheel', fn) : this.trigger('mousewheel');
        },

        unmousewheel: function(fn) {
            return this.unbind('mousewheel', fn);
        }
    });


    function handler(event) {
        var orgEvent   = event || window.event,
            args       = [].slice.call(arguments, 1),
            delta      = 0,
            deltaX     = 0,
            deltaY     = 0,
            absDelta   = 0,
            absDeltaXY = 0,
            fn;
        event = $.event.fix(orgEvent);
        event.type = 'mousewheel';

        // Old school scrollwheel delta
        if ( orgEvent.wheelDelta ) { delta = orgEvent.wheelDelta; }
        if ( orgEvent.detail )     { delta = orgEvent.detail * -1; }

        // At a minimum, setup the deltaY to be delta
        deltaY = delta;

        // Firefox < 17 related to DOMMouseScroll event
        if ( orgEvent.axis !== undefined && orgEvent.axis === orgEvent.HORIZONTAL_AXIS ) {
            deltaY = 0;
            deltaX = delta * -1;
        }

        // New school wheel delta (wheel event)
        if ( orgEvent.deltaY ) {
            deltaY = orgEvent.deltaY * -1;
            delta  = deltaY;
        }
        if ( orgEvent.deltaX ) {
            deltaX = orgEvent.deltaX;
            delta  = deltaX * -1;
        }

        // Webkit
        if ( orgEvent.wheelDeltaY !== undefined ) { deltaY = orgEvent.wheelDeltaY; }
        if ( orgEvent.wheelDeltaX !== undefined ) { deltaX = orgEvent.wheelDeltaX * -1; }

        // Look for lowest delta to normalize the delta values
        absDelta = Math.abs(delta);
        if ( !lowestDelta || absDelta < lowestDelta ) { lowestDelta = absDelta; }
        absDeltaXY = Math.max(Math.abs(deltaY), Math.abs(deltaX));
        if ( !lowestDeltaXY || absDeltaXY < lowestDeltaXY ) { lowestDeltaXY = absDeltaXY; }

        // Get a whole value for the deltas
        fn     = delta > 0 ? 'floor' : 'ceil';
        delta  = Math[fn](delta  / lowestDelta);
        deltaX = Math[fn](deltaX / lowestDeltaXY);
        deltaY = Math[fn](deltaY / lowestDeltaXY);

        // Add event and delta to the front of the arguments
        args.unshift(event, delta, deltaX, deltaY);

        return ($.event.dispatch || $.event.handle).apply(this, args);
    }

}));// requestAnimationFrame() shim by Paul Irish
// http://paulirish.com/2011/requestanimationframe-for-smart-animating/
window.requestAnimFrame = (function() {
	return  window.requestAnimationFrame       || 
			window.webkitRequestAnimationFrame || 
			window.mozRequestAnimationFrame    || 
			window.oRequestAnimationFrame      || 
			window.msRequestAnimationFrame     || 
			function(/* function */ callback, /* DOMElement */ element){
				window.setTimeout(callback, 1000 / 60);
			};
})();// stats.js r6 - http://github.com/mrdoob/stats.js
var Stats=function(){function s(a,g,d){var f,c,e;for(c=0;c<30;c++)for(f=0;f<73;f++)e=(f+c*74)*4,a[e]=a[e+4],a[e+1]=a[e+5],a[e+2]=a[e+6];for(c=0;c<30;c++)e=(73+c*74)*4,c<g?(a[e]=b[d].bg.r,a[e+1]=b[d].bg.g,a[e+2]=b[d].bg.b):(a[e]=b[d].fg.r,a[e+1]=b[d].fg.g,a[e+2]=b[d].fg.b)}var r=0,t=2,g,u=0,j=(new Date).getTime(),F=j,v=j,l=0,w=1E3,x=0,k,d,a,m,y,n=0,z=1E3,A=0,f,c,o,B,p=0,C=1E3,D=0,h,i,q,E,b={fps:{bg:{r:16,g:16,b:48},fg:{r:0,g:255,b:255}},ms:{bg:{r:16,g:48,b:16},fg:{r:0,g:255,b:0}},mb:{bg:{r:48,g:16,
b:26},fg:{r:255,g:0,b:128}}};g=document.createElement("div");g.style.cursor="pointer";g.style.width="80px";g.style.opacity="0.9";g.style.zIndex="10001";g.addEventListener("click",function(){r++;r==t&&(r=0);k.style.display="none";f.style.display="none";h.style.display="none";switch(r){case 0:k.style.display="block";break;case 1:f.style.display="block";break;case 2:h.style.display="block"}},!1);k=document.createElement("div");k.style.backgroundColor="rgb("+Math.floor(b.fps.bg.r/2)+","+Math.floor(b.fps.bg.g/
2)+","+Math.floor(b.fps.bg.b/2)+")";k.style.padding="2px 0px 3px 0px";g.appendChild(k);d=document.createElement("div");d.style.fontFamily="Helvetica, Arial, sans-serif";d.style.textAlign="left";d.style.fontSize="9px";d.style.color="rgb("+b.fps.fg.r+","+b.fps.fg.g+","+b.fps.fg.b+")";d.style.margin="0px 0px 1px 3px";d.innerHTML='<span style="font-weight:bold">FPS</span>';k.appendChild(d);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";k.appendChild(a);
m=a.getContext("2d");m.fillStyle="rgb("+b.fps.bg.r+","+b.fps.bg.g+","+b.fps.bg.b+")";m.fillRect(0,0,a.width,a.height);y=m.getImageData(0,0,a.width,a.height);f=document.createElement("div");f.style.backgroundColor="rgb("+Math.floor(b.ms.bg.r/2)+","+Math.floor(b.ms.bg.g/2)+","+Math.floor(b.ms.bg.b/2)+")";f.style.padding="2px 0px 3px 0px";f.style.display="none";g.appendChild(f);c=document.createElement("div");c.style.fontFamily="Helvetica, Arial, sans-serif";c.style.textAlign="left";c.style.fontSize=
"9px";c.style.color="rgb("+b.ms.fg.r+","+b.ms.fg.g+","+b.ms.fg.b+")";c.style.margin="0px 0px 1px 3px";c.innerHTML='<span style="font-weight:bold">MS</span>';f.appendChild(c);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";f.appendChild(a);o=a.getContext("2d");o.fillStyle="rgb("+b.ms.bg.r+","+b.ms.bg.g+","+b.ms.bg.b+")";o.fillRect(0,0,a.width,a.height);B=o.getImageData(0,0,a.width,a.height);try{performance&&performance.memory&&performance.memory.totalJSHeapSize&&
(t=3)}catch(G){}h=document.createElement("div");h.style.backgroundColor="rgb("+Math.floor(b.mb.bg.r/2)+","+Math.floor(b.mb.bg.g/2)+","+Math.floor(b.mb.bg.b/2)+")";h.style.padding="2px 0px 3px 0px";h.style.display="none";g.appendChild(h);i=document.createElement("div");i.style.fontFamily="Helvetica, Arial, sans-serif";i.style.textAlign="left";i.style.fontSize="9px";i.style.color="rgb("+b.mb.fg.r+","+b.mb.fg.g+","+b.mb.fg.b+")";i.style.margin="0px 0px 1px 3px";i.innerHTML='<span style="font-weight:bold">MB</span>';
h.appendChild(i);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";h.appendChild(a);q=a.getContext("2d");q.fillStyle="#301010";q.fillRect(0,0,a.width,a.height);E=q.getImageData(0,0,a.width,a.height);return{domElement:g,update:function(){u++;j=(new Date).getTime();n=j-F;z=Math.min(z,n);A=Math.max(A,n);s(B.data,Math.min(30,30-n/200*30),"ms");c.innerHTML='<span style="font-weight:bold">'+n+" MS</span> ("+z+"-"+A+")";o.putImageData(B,0,0);F=j;if(j>
v+1E3){l=Math.round(u*1E3/(j-v));w=Math.min(w,l);x=Math.max(x,l);s(y.data,Math.min(30,30-l/100*30),"fps");d.innerHTML='<span style="font-weight:bold">'+l+" FPS</span> ("+w+"-"+x+")";m.putImageData(y,0,0);if(t==3)p=performance.memory.usedJSHeapSize*9.54E-7,C=Math.min(C,p),D=Math.max(D,p),s(E.data,Math.min(30,30-p/2),"mb"),i.innerHTML='<span style="font-weight:bold">'+Math.round(p)+" MB</span> ("+Math.round(C)+"-"+Math.round(D)+")",q.putImageData(E,0,0);v=j;u=0}}}};

Constants={},Constants.PI=Math.PI,Constants.C_PR=Math.PI/180,Constants.VLEV=2,Constants.EPS=1e-7,Constants.c=.105,Constants.LN10=Math.log(10),Constants.PIOVER2=Math.PI/2,Constants.TWOPI=2*Math.PI,Constants.TWOTHIRD=2/3,Constants.ARCSECOND_RADIAN=484813681109536e-20,SpatialVector=function(){function t(t,s,i){"use strict";this.x=t,this.y=s,this.z=i,this.ra_=0,this.dec_=0,this.okRaDec_=!1}return t.prototype.setXYZ=function(t,s,i){this.x=t,this.y=s,this.z=i,this.okRaDec_=!1},t.prototype.length=function(){"use strict";return Math.sqrt(this.lengthSquared())},t.prototype.lengthSquared=function(){"use strict";return this.x*this.x+this.y*this.y+this.z*this.z},t.prototype.normalized=function(){"use strict";var t=this.length();this.x/=t,this.y/=t,this.z/=t},t.prototype.set=function(t,s){"use strict";this.ra_=t,this.dec_=s,this.okRaDec_=!0,this.updateXYZ()},t.prototype.angle=function(t){"use strict";var s=this.y*t.z-this.z*t.y,i=this.z*t.x-this.x*t.z,n=this.x*t.y-this.y*t.x,a=Math.sqrt(s*s+i*i+n*n);return Math.abs(Math.atan2(a,dot(t)))},t.prototype.get=function(){"use strict";return[x,y,z]},t.prototype.toString=function(){"use strict";return"SpatialVector["+this.x+", "+this.y+", "+this.z+"]"},t.prototype.cross=function(s){"use strict";return new t(this.y*s.z-s.y*this.z,this.z*s.x-s.z*this.x,this.x*s.y-s.x()*this.y)},t.prototype.equal=function(t){"use strict";return this.x==t.x&&this.y==t.y&&this.z==t.z()?!0:!1},t.prototype.mult=function(s){"use strict";return new t(s*this.x,s*this.y,s*this.z)},t.prototype.dot=function(t){"use strict";return this.x*t.x+this.y*t.y+this.z*t.z},t.prototype.add=function(s){"use strict";return new t(this.x+s.x,this.y+s.y,this.z+s.z)},t.prototype.sub=function(s){"use strict";return new t(this.x-s.x,this.y-s.y,this.z-s.z)},t.prototype.dec=function(){"use strict";return this.okRaDec_||(this.normalized(),this.updateRaDec()),this.dec_},t.prototype.ra=function(){"use strict";return this.okRaDec_||(this.normalized(),this.updateRaDec()),this.ra_},t.prototype.updateXYZ=function(){"use strict";var t=Math.cos(this.dec_*Constants.C_PR);this.x=Math.cos(this.ra_*Constants.C_PR)*t,this.y=Math.sin(this.ra_*Constants.C_PR)*t,this.z=Math.sin(this.dec_*Constants.C_PR)},t.prototype.updateRaDec=function(){"use strict";this.dec_=Math.asin(this.z)/Constants.C_PR;var t=Math.cos(this.dec_*Constants.C_PR);this.ra_=t>Constants.EPS||-Constants.EPS>t?this.y>Constants.EPS||this.y<-Constants.EPS?0>this.y?360-Math.acos(this.x/t)/Constants.C_PR:Math.acos(this.x/t)/Constants.C_PR:0>this.x?180:0:0,this.okRaDec_=!0},t.prototype.toRaRadians=function(){"use strict";var t=0;return(0!=this.x||0!=this.y)&&(t=Math.atan2(this.y,this.x)),0>t&&(t+=2*Math.PI),t},t.prototype.toDeRadians=function(){var t=z/this.length(),s=Math.acos(t);return Math.PI/2-s},t}(),AngularPosition=function(){return AngularPosition=function(t,s){"use strict";this.theta=t,this.phi=s},AngularPosition.prototype.toString=function(){"use strict";return"theta: "+this.theta+", phi: "+this.phi},AngularPosition}(),LongRangeSetBuilder=function(){function t(){this.items=[]}return t.prototype.appendRange=function(t,s){for(var i=t;s>=i;i++)i in this.items||this.items.push(i)},t}(),HealpixIndex=function(){function t(t){"use strict";this.nside=t}return t.NS_MAX=8192,t.ORDER_MAX=13,t.NSIDELIST=[1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192],t.JRLL=[2,2,2,2,3,3,3,3,4,4,4,4],t.JPLL=[1,3,5,7,0,2,4,6,1,3,5,7],t.XOFFSET=[-1,-1,0,1,1,1,0,-1],t.YOFFSET=[0,1,1,1,0,-1,-1,-1],t.FACEARRAY=[[8,9,10,11,-1,-1,-1,-1,10,11,8,9],[5,6,7,4,8,9,10,11,9,10,11,8],[-1,-1,-1,-1,5,6,7,4,-1,-1,-1,-1],[4,5,6,7,11,8,9,10,11,8,9,10],[0,1,2,3,4,5,6,7,8,9,10,11],[1,2,3,0,0,1,2,3,5,6,7,4],[-1,-1,-1,-1,7,4,5,6,-1,-1,-1,-1],[3,0,1,2,3,0,1,2,4,5,6,7],[2,3,0,1,-1,-1,-1,-1,0,1,2,3]],t.SWAPARRAY=[[0,0,0,0,0,0,0,0,3,3,3,3],[0,0,0,0,0,0,0,0,6,6,6,6],[0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,5,5,5,5],[0,0,0,0,0,0,0,0,0,0,0,0],[5,5,5,5,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0],[6,6,6,6,0,0,0,0,0,0,0,0],[3,3,3,3,0,0,0,0,0,0,0,0]],t.Z0=Constants.TWOTHIRD,t.prototype.init=function(){"use strict";var s=256;this.ctab=Array(s),this.utab=Array(s);for(var i=0;256>i;++i)this.ctab[i]=1&i|(2&i)<<7|(4&i)>>1|(8&i)<<6|(16&i)>>2|(32&i)<<5|(64&i)>>3|(128&i)<<4,this.utab[i]=1&i|(2&i)<<1|(4&i)<<2|(8&i)<<3|(16&i)<<4|(32&i)<<5|(64&i)<<6|(128&i)<<7;this.nl2=2*this.nside,this.nl3=3*this.nside,this.nl4=4*this.nside,this.npface=this.nside*this.nside,this.ncap=2*this.nside*(this.nside-1),this.npix=12*this.npface,this.fact2=4/this.npix,this.fact1=(this.nside<<1)*this.fact2,this.order=t.nside2order(this.nside)},t.calculateNSide=function(s){for(var i=0,n=s*s,a=180/Constants.PI,e=3600*3600*4*Constants.PI*a*a,h=Utils.castToInt(e/n),r=h/12,o=Math.sqrt(r),c=t.NS_MAX,u=0,p=0;t.NSIDELIST.length>p;p++)if(c>=Math.abs(o-t.NSIDELIST[p])&&(c=Math.abs(o-t.NSIDELIST[p]),i=t.NSIDELIST[p],u=p),o>i&&t.NS_MAX>o&&(i=t.NSIDELIST[u+1]),o>t.NS_MAX)return console.log("nside cannot be bigger than "+t.NS_MAX),t.NS_MAX;return i},t.nside2order=function(s){"use strict";return(s&s-1)>0?-1:Utils.castToInt(t.log2(s))},t.log2=function(t){"use strict";return Math.log(t)/Math.log(2)},t.prototype.ang2pix_nest=function(s,i){"use strict";var n,a,e,h,r,o,c,u,p,l,d,f,I;if(i>=Constants.TWOPI&&(i-=Constants.TWOPI),0>i&&(i+=Constants.TWOPI),s>Constants.PI||0>s)throw{name:"Illegal argument",message:"theta must be between 0 and "+Constants.PI};if(i>Constants.TWOPI||0>i)throw{name:"Illegal argument",message:"phi must be between 0 and "+Constants.TWOPI};if(a=Math.cos(s),e=Math.abs(a),h=i/Constants.PIOVER2,t.Z0>=e){var M=this.nside*(.5+h),y=this.nside*.75*a,u=M-y,p=M+y;o=u>>this.order,c=p>>this.order,d=o==c?4==o?4:o+4:c>o?o:c+8,f=Utils.castToInt(p&this.nside-1),I=Utils.castToInt(this.nside-(u&this.nside-1)-1)}else{l=Utils.castToInt(h),l>=4&&(l=3),r=h-l;var g=this.nside*Math.sqrt(3*(1-e));u=Utils.castToInt(r*g),p=Utils.castToInt((1-r)*g),u=Math.min(t.NS_MAX-1,u),p=Math.min(t.NS_MAX-1,p),a>=0?(d=l,f=Utils.castToInt(this.nside-p-1),I=Utils.castToInt(this.nside-u-1)):(d=l+8,f=u,I=p)}return n=this.xyf2nest(f,I,d)},t.prototype.xyf2nest=function(t,s,i){"use strict";return(i<<2*this.order)+(this.utab[255&t]|this.utab[255&t>>8]<<16|this.utab[255&t>>16]<<32|this.utab[255&t>>24]<<48|this.utab[255&s]<<1|this.utab[255&s>>8]<<17|this.utab[255&s>>16]<<33|this.utab[255&s>>24]<<49)},t.prototype.nest2xyf=function(t){"use strict";var s={};s.face_num=t>>2*this.order;var i=t&this.npface-1,n=(93823560581120&i)>>16|(614882086624428e4&i)>>31|21845&i|(1431633920&i)>>15;return s.ix=this.ctab[255&n]|this.ctab[255&n>>8]<<4|this.ctab[255&n>>16]<<16|this.ctab[255&n>>24]<<20,i>>=1,n=(93823560581120&i)>>16|(614882086624428e4&i)>>31|21845&i|(1431633920&i)>>15,s.iy=this.ctab[255&n]|this.ctab[255&n>>8]<<4|this.ctab[255&n>>16]<<16|this.ctab[255&n>>24]<<20,s},t.prototype.pix2ang_nest=function(s){"use strict";if(0>s||s>this.npix-1)throw{name:"Illegal argument",message:"ipix out of range"};var i,n,a,e=this.nest2xyf(s),h=e.ix,r=e.iy,o=e.face_num,c=(t.JRLL[o]<<this.order)-h-r-1;this.nside>c?(i=c,n=1-i*i*this.fact2,a=0):c>this.nl3?(i=this.nl4-c,n=i*i*this.fact2-1,a=0):(i=this.nside,n=(this.nl2-c)*this.fact1,a=1&c-this.nside);var u=Math.acos(n),p=(t.JPLL[o]*i+h-r+1+a)/2;p>this.nl4&&(p-=this.nl4),1>p&&(p+=this.nl4);var l=(p-.5*(a+1))*(Constants.PIOVER2/i);return{theta:u,phi:l}},t.nside2Npix=function(s){"use strict";if(0>this.NSIDELIST.indexOf(s))throw{name:"Illegal argument",message:"nside should be >0, power of 2, <"+t.NS_MAX};var i=12*s*s;return i},t.prototype.xyf2ring=function(s,i,n){"use strict";var a,e,h,r=t.JRLL[n]*this.nside-s-i-1;this.nside>r?(a=r,h=2*a*(a-1),e=0):r>3*this.nside?(a=this.nl4-r,h=this.npix-2*(a+1)*a,e=0):(a=this.nside,h=this.ncap+(r-this.nside)*this.nl4,e=1&r-this.nside);var o=(t.JPLL[n]*a+s-i+1+e)/2;return o>this.nl4?o-=this.nl4:1>o&&(o+=this.nl4),h+o-1},t.prototype.nest2ring=function(t){"use strict";var s=this.nest2xyf(t),i=this.xyf2ring(s.ix,s.iy,s.face_num);return i},t.prototype.corners_nest=function(t,s){"use strict";var i=this.nest2ring(t);return this.corners_ring(i,s)},t.prototype.pix2ang_ring=function(t){"use strict";var s,i,n,a,e,h,r,o,c;if(0>t||t>this.npix-1)throw{name:"Illegal argument",message:"ipix out of range"};return h=t+1,this.ncap>=h?(o=h/2,c=Utils.castToInt(o),n=Utils.castToInt(Math.sqrt(o-Math.sqrt(c)))+1,a=h-2*n*(n-1),s=Math.acos(1-n*n*this.fact2),i=(a-.5)*Constants.PI/(2*n)):this.npix-this.ncap>t?(e=t-this.ncap,n=e/this.nl4+this.nside,a=e%this.nl4+1,r=(1&n+this.nside)>0?1:.5,s=Math.acos((this.nl2-n)*this.fact1),i=(a-r)*Constants.PI/this.nl2):(e=this.npix-t,n=Utils.castToInt(.5*(1+Math.sqrt(2*e-1))),a=4*n+1-(e-2*n*(n-1)),s=Math.acos(-1+Math.pow(n,2)*this.fact2),i=(a-.5)*Constants.PI/(2*n)),[s,i]},t.prototype.ring=function(t){"use strict";var s,i,n=0,a=t+1,e=0;return this.ncap>=a?(i=a/2,e=Utils.castToInt(i),n=Utils.castToInt(Math.sqrt(i-Math.sqrt(e)))+1):this.nl2*(5*this.nside+1)>=a?(s=Utils.castToInt(a-this.ncap-1),n=Utils.castToInt(s/this.nl4+this.nside)):(s=this.npix-a+1,i=s/2,e=Utils.castToInt(i),n=Utils.castToInt(Math.sqrt(i-Math.sqrt(e)))+1,n=this.nl4-n),n},t.prototype.integration_limits_in_costh=function(t){"use strict";var s,i,n,a;return a=1*this.nside,this.nside>=t?(i=1-Math.pow(t,2)/3/this.npface,n=1-Math.pow(t-1,2)/3/this.npface,s=t==this.nside?2*(this.nside-1)/3/a:1-Math.pow(t+1,2)/3/this.npface):this.nl3>t?(i=2*(2*this.nside-t)/3/a,n=2*(2*this.nside-t+1)/3/a,s=2*(2*this.nside-t-1)/3/a):(n=t==this.nl3?2*(-this.nside+1)/3/a:-1+Math.pow(4*this.nside-t+1,2)/3/this.npface,s=-1+Math.pow(this.nl4-t-1,2)/3/this.npface,i=-1+Math.pow(this.nl4-t,2)/3/this.npface),[n,i,s]},t.prototype.pixel_boundaries=function(t,s,i,n){var a,e,h,r,o,c,u,p,l=1*this.nside;if(Math.abs(n)>=1-1/3/this.npface)return u=i*Constants.PIOVER2,p=(i+1)*Constants.PIOVER2,[u,p];if(1.5*n>=1)a=Math.sqrt(3*(1-n)),e=1/l/a,h=s,r=h-1,o=t-s,c=o+1,u=Constants.PIOVER2*(Math.max(r*e,1-c*e)+i),p=Constants.PIOVER2*(Math.min(1-o*e,h*e)+i);else if(1.5*n>-1){var d=.5*(1-1.5*n),f=d+1,I=this.nside+t%2;h=s-(I-t)/2,r=h-1,o=(I+t)/2-s,c=o+1,u=Constants.PIOVER2*(Math.max(f-c/l,-d+r/l)+i),p=Constants.PIOVER2*(Math.min(f-o/l,-d+h/l)+i)}else{a=Math.sqrt(3*(1+n)),e=1/l/a;var M=2*this.nside;h=t-M+s,r=h-1,o=M-s,c=o+1,u=Constants.PIOVER2*(Math.max(1-(M-r)*e,(M-c)*e)+i),p=Constants.PIOVER2*(Math.min(1-(M-h)*e,(M-o)*e)+i)}return[u,p]},t.vector=function(t,s){"use strict";var i=1*Math.sin(t)*Math.cos(s),n=1*Math.sin(t)*Math.sin(s),a=1*Math.cos(t);return new SpatialVector(i,n,a)},t.prototype.corners_ring=function(s,i){"use strict";var n=2*i+2,a=Array(n),e=this.pix2ang_ring(s),h=Math.cos(e[0]),r=e[0],o=e[1],c=Utils.castToInt(o/Constants.PIOVER2),u=this.ring(s),p=Math.min(u,Math.min(this.nside,this.nl4-u)),l=0,d=Constants.PIOVER2/p;l=u>=this.nside&&this.nl3>=u?Utils.castToInt(o/d+u%2/2)+1:Utils.castToInt(o/d)+1,l-=c*p;var f=n/2,I=this.integration_limits_in_costh(u),M=Math.acos(I[0]),y=Math.acos(I[2]),g=this.pixel_boundaries(u,l,c,I[0]);if(a[0]=l>p/2?t.vector(M,g[1]):t.vector(M,g[0]),g=this.pixel_boundaries(u,l,c,I[2]),a[f]=l>p/2?t.vector(y,g[1]):t.vector(y,g[0]),1==i){var x=Math.acos(I[1]);g=this.pixel_boundaries(u,l,c,I[1]),a[1]=t.vector(x,g[0]),a[3]=t.vector(x,g[1])}else for(var P=I[2]-I[0],C=P/(i+1),v=1;i>=v;v++)h=I[0]+C*v,r=Math.acos(h),g=this.pixel_boundaries(u,l,c,h),a[v]=t.vector(r,g[0]),a[n-v]=t.vector(r,g[1]);return a},t.vec2Ang=function(t){"use strict";var s=t.z/t.length(),i=Math.acos(s),n=0;return(0!=t.x||0!=t.y)&&(n=Math.atan2(t.y,t.x)),0>n&&(n+=2*Math.PI),[i,n]},t.prototype.queryDisc=function(s,i,n,a){"use strict";if(0>i||i>Constants.PI)throw{name:"Illegal argument",message:"angular radius is in RADIAN and should be in [0,pi]"};var e,h,r,o,c,u,p,l,d,f,I,M,y,g,x,P,C,v,_,T=new LongRangeSetBuilder,R=null,c=i;if(a&&(c+=Constants.PI/this.nl4),R=t.vec2Ang(s),u=R[0],p=R[1],I=this.fact2,M=this.fact1,o=Math.cos(u),_=1/Math.sqrt((1-o)*(1+o)),g=u-c,x=u+c,l=Math.cos(c),C=Math.cos(g),e=this.ringAbove(C)+1,P=Math.cos(x),h=this.ringAbove(P),e>h&&0==h&&(h=e),0>=g)for(var m=1;e>m;++m)this.inRing(m,0,Math.PI,T);for(r=e;h>=r;++r)v=this.nside>r?1-r*r*I:this.nl3>=r?(this.nl2-r)*M:-1+(this.nl4-r)*(this.nl4-r)*I,d=(l-v*o)*_,f=1-v*v-d*d,y=Math.atan2(Math.sqrt(f),d),isNaN(y)&&(y=c),this.inRing(r,p,y,T);if(x>=Math.PI)for(var m=h+1;this.nl4>m;++m)this.inRing(m,0,Math.PI,T,!1);var b;if(n){for(var S=T.items,U=[],O=0;S.length>O;O++){var A=this.ring2nest(S[O]);U.indexOf(A)>=0||U.push(A)}b=U}else b=T.items;return b},t.prototype.inRing=function(t,s,i,n,a){"use strict";var e,h,r,o,c=!1,u=!1,p=1e-12,l=0,d=0,f=0,I=0,M=(s-i)%Constants.TWOPI-p,y=s+i+p,g=(s+i)%Constants.TWOPI+p;if(p>Math.abs(i-Constants.PI)&&(c=!0),t>=this.nside&&this.nl3>=t?(d=t-this.nside+1,r=this.ncap+this.nl4*(d-1),o=r+this.nl4-1,e=d%2,h=this.nl4):(this.nside>t?(d=t,r=2*d*(d-1),o=r+4*d-1):(d=4*this.nside-t,r=this.npix-2*d*(d+1),o=r+4*d-1),h=4*d,e=1),c)return n.appendRange(r,o),void 0;if(l=e/2,a)f=Math.round(h*M/Constants.TWOPI-l),I=Math.round(h*y/Constants.TWOPI-l),f%=h,I>h&&(I%=h);else{if(f=Math.ceil(h*M/Constants.TWOPI-l),I=Utils.castToInt(h*g/Constants.TWOPI-l),f>I&&1==t&&(I=Utils.castToInt(h*y/Constants.TWOPI-l)),f==I+1&&(f=I),1==f-I&&Constants.PI>i*h)return console.log("the interval is too small and avay from center"),void 0;f=Math.min(f,h-1),I=Math.max(I,0)}if(f>I&&(u=!0),u)f+=r,I+=r,n.appendRange(r,I),n.appendRange(f,o);else{if(0>f)return f=Math.abs(f),n.appendRange(r,r+I),n.appendRange(o-f+1,o),void 0;f+=r,I+=r,n.appendRange(f,I)}},t.prototype.ringAbove=function(t){"use strict";var s=Math.abs(t);if(s>Constants.TWOTHIRD){var i=Utils.castToInt(this.nside*Math.sqrt(3*(1-s)));return t>0?i:4*this.nside-i-1}return Utils.castToInt(this.nside*(2-1.5*t))},t.prototype.ring2nest=function(t){"use strict";var s=this.ring2xyf(t);return this.xyf2nest(s.ix,s.iy,s.face_num)},t.prototype.ring2xyf=function(s){"use strict";var i,n,a,e,h={};if(this.ncap>s){i=Utils.castToInt(.5*(1+Math.sqrt(1+2*s))),n=s+1-2*i*(i-1),a=0,e=i,h.face_num=0;var r=n-1;r>=2*i&&(h.face_num=2,r-=2*i),r>=i&&++h.face_num}else if(this.npix-this.ncap>s){var o=s-this.ncap;this.order>=0?(i=(o>>this.order+2)+this.nside,n=(o&this.nl4-1)+1):(i=o/this.nl4+this.nside,n=o%this.nl4+1),a=1&i+this.nside,e=this.nside;var c,u,p=i-this.nside+1,l=this.nl2+2-p;this.order>=0?(c=n-Utils.castToInt(p/2)+this.nside-1>>this.order,u=n-Utils.castToInt(l/2)+this.nside-1>>this.order):(c=(n-Utils.castToInt(p/2)+this.nside-1)/this.nside,u=(n-Utils.castToInt(l/2)+this.nside-1)/this.nside),h.face_num=u==c?4==u?4:Utils.castToInt(u)+4:c>u?Utils.castToInt(u):Utils.castToInt(c)+8}else{var o=this.npix-s;i=Utils.castToInt(.5*(1+Math.sqrt(2*o-1))),n=4*i+1-(o-2*i*(i-1)),a=0,e=i,i=2*this.nl2-i,h.face_num=8;var r=n-1;r>=2*e&&(h.face_num=10,r-=2*e),r>=e&&++h.face_num}var d=i-t.JRLL[h.face_num]*this.nside+1,f=2*n-t.JPLL[h.face_num]*e-a-1;return f>=this.nl2&&(f-=8*this.nside),h.ix=f-d>>1,h.iy=-(f+d)>>1,h},t}(),Utils=function(){},Utils.radecToPolar=function(t,s){return{theta:Math.PI/2-s/180*Math.PI,phi:t/180*Math.PI}},Utils.polarToRadec=function(t,s){return{ra:180*s/Math.PI,dec:180*(Math.PI/2-t)/Math.PI}},Utils.castToInt=function(t){return t>0?Math.floor(t):Math.ceil(t)};//=================================
//            AstroMath
//=================================

// Class AstroMath having 'static' methods
function AstroMath() {}

// Constant for conversion Degrees => Radians (rad = deg*AstroMath.D2R)
AstroMath.D2R = Math.PI/180.0;
// Constant for conversion Radians => Degrees (deg = rad*AstroMath.R2D)
AstroMath.R2D = 180.0/Math.PI;
/**
 * Function sign
 * @param x value for checking the sign
 * @return -1, 0, +1 respectively if x < 0, = 0, > 0
 */
AstroMath.sign = function(x) { return x > 0 ? 1 : (x < 0 ? -1 : 0 ); };

/**
 * Function cosd(degrees)
 * @param x angle in degrees
 * @returns the cosine of the angle
 */
AstroMath.cosd = function(x) {
	if (x % 90 == 0) {
		var i = Math.abs(Math.floor(x / 90 + 0.5)) % 4;
		switch (i) {
			case 0:	return 1;
			case 1:	return 0;
			case 2:	return -1;
			case 3:	return 0;
		}
	}
	return Math.cos(x*AstroMath.D2R);
};

/**
 * Function sind(degrees)
 * @param x angle in degrees
 * @returns the sine of the angle
 */
AstroMath.sind = function(x) {
	if (x % 90 === 0) {
		var i = Math.abs(Math.floor(x / 90 - 0.5)) % 4;
		switch (i) {
			case 0:	return 1;
			case 1:	return 0;
			case 2:	return -1;
			case 3:	return 0;
		}
	}

	return Math.sin(x*AstroMath.D2R);
};

/**
 * Function tand(degrees)
 * @param x angle in degrees
 * @returns the tangent of the angle
 */
AstroMath.tand = function(x) {
	var resid;

	resid = x % 360;
	if (resid == 0 || Math.abs(resid) == 180) {
		return 0;
	} else if (resid == 45 || resid == 225) {
		return 1;
	} else if (resid == -135 || resid == -315) {
		return -1
	}

	return Math.tan(x * AstroMath.D2R);
};

/**
 * Function asin(degrees)
 * @param sine value [0,1]
 * @return the angle in degrees
 */
AstroMath.asind = function(x) { return Math.asin(x)*AstroMath.R2D; };

/**
 * Function acos(degrees)
 * @param cosine value [0,1]
 * @return the angle in degrees
 */
AstroMath.acosd = function(x) { return Math.acos(x)*AstroMath.R2D; };

/**
 * Function atan(degrees)
 * @param tangent value
 * @return the angle in degrees
 */
AstroMath.atand = function(x) { return Math.atan(x)*AstroMath.R2D; };

/**
 * Function atan2(y,x)
 * @param y y component of the vector
 * @param x x component of the vector
 * @return the angle in radians
 */
AstroMath.atan2 = function(y,x) {
	if (y != 0.0) {
		var sgny = AstroMath.sign(y);
		if (x != 0.0) {
			var phi = Math.atan(Math.abs(y/x));
			if (x > 0.0) return phi*sgny;
			else if (x < 0) return (Math.PI-phi)*sgny;
		} else return (Math.PI/2)*sgny;
	} else {
		return x > 0.0 ? 0.0 : (x < 0 ? Math.PI : 0.0/0.0);
	}
}  

/**
 * Function atan2d(y,x)
 * @param y y component of the vector
 * @param x x component of the vector
 * @return the angle in degrees
 */
AstroMath.atan2d = function(y,x) {
	return AstroMath.atan2(y,x)*AstroMath.R2D;
}

/*=========================================================================*/
/**
 * Computation of hyperbolic cosine
 * @param x argument
 */
AstroMath.cosh = function(x) {
	return (Math.exp(x)+Math.exp(-x))/2;
}

/**
 * Computation of hyperbolic sine
 * @param x argument
 */
AstroMath.sinh = function(x) {
	return (Math.exp(x)-Math.exp(-x))/2;
}

/**
 * Computation of hyperbolic tangent
 * @param x argument
 */
AstroMath.tanh = function(x) {
	return (Math.exp(x)-Math.exp(-x))/(Math.exp(x)+Math.exp(-x));
}

/**
 * Computation of Arg cosh
 * @param x argument in degrees. Must be in the range [ 1, +infinity ]
 */
AstroMath.acosh = function(x) {
	return(Math.log(x+Math.sqrt(x*x-1.0)));
}

/**
 * Computation of Arg sinh
 * @param x argument in degrees
 */
AstroMath.asinh = function(x) {
	return(Math.log(x+Math.sqrt(x*x+1.0)));
}

/**
 * Computation of Arg tanh
 * @param x argument in degrees. Must be in the range ] -1, +1 [
 */
AstroMath.atanh = function(x) {
	return(0.5*Math.log((1.0+x)/(1.0-x)));
}

//=============================================================================
//      Special Functions using trigonometry
//=============================================================================
/**
 * Computation of sin(x)/x
 *	@param x in degrees.
 * For small arguments x <= 0.001, use approximation 
 */
AstroMath.sinc = function(x) {
	var ax = Math.abs(x);
	var y;

	if (ax <= 0.001) {
		ax *= ax;
		y = 1 - ax*(1.0-ax/20.0)/6.0;
	} else {
		y = Math.sin(ax)/ax;
	}

	return y;
}

/**
 * Computes asin(x)/x
 * @param x in degrees.
 * For small arguments x <= 0.001, use an approximation
 */
AstroMath.asinc = function(x) {
	var ax = Math.abs(x);
	var y;

	if (ax <= 0.001) {
		ax *= ax; 
		y = 1 + ax*(6.0 + ax*(9.0/20.0))/6.0;
	} else {
		y = Math.asin(ax)/ax;	// ???? radians ???
	}

	return (y);
}


//=============================================================================
/**
 * Computes the hypotenuse of x and y
 * @param x value
 * @param y value
 * @return sqrt(x*x+y*y)
 */
AstroMath.hypot = function(x,y) {
	return Math.sqrt(x*x+y*y);
}

/** Generate the rotation matrix from the Euler angles
 * @param z	Euler angle
 * @param theta	Euler angle
 * @param zeta	Euler angles
 * @return R [3][3]		the rotation matrix
 * The rotation matrix is defined by:<pre>
 *    R =      R_z(-z)      *        R_y(theta)     *     R_z(-zeta)
 *   |cos.z -sin.z  0|   |cos.the  0 -sin.the|   |cos.zet -sin.zet 0|
 * = |sin.z  cos.z  0| x |   0     1     0   | x |sin.zet  cos.zet 0|
 *   |   0      0   1|   |sin.the  0  cos.the|   |   0        0    1|
 * </pre>
 */
AstroMath.eulerMatrix = function(z, theta, zeta) {
	var R = new Array(3);
	R[0] = new Array(3);
	R[1] = new Array(3);
	R[2] = new Array(3);
	var cosdZ = AstroMath.cosd(z);
	var sindZ = AstroMath.sind(z);
	var cosdTheta = AstroMath.cosd(theta);
	var w = AstroMath.sind(theta) ;
	var cosdZeta = AstroMath.cosd(zeta);
	var sindZeta = AstroMath.sind(zeta);

	R[0][0] = cosdZeta*cosdTheta*cosdZ - sindZeta*sindZ;
	R[0][1] = -sindZeta*cosdTheta*cosdZ - cosdZeta*sindZ;
	R[0][2] = -w*cosdZ;

	R[1][0] = cosdZeta*cosdTheta*sindZ + sindZeta*cosdZ;
	R[1][1] = -sindZeta*cosdTheta*sindZ + cosdZeta*cosdZ;
	R[1][2] = -w*sindZ;

	R[2][0] = -w*cosdZeta;
	R[2][1] = -w*cosdZ;
	R[2][2] = cosdTheta;
	return R ;
};


AstroMath.displayMatrix = function(m) {
	// Number of rows
	var nbrows = m.length;
	// Max column count
	var nbcols = 0
	for (var i=0; i<nbrows; i++) {
		if (m[i].length > nbcols) nbcols = m[i].length;
	}
	var str = '<table>\n';
	for (var i=0; i<nbrows; i++) {
		str += '<tr>';
		for (var j=0; j<nbrows; j++) {
			str += '<td>';
			if (i < m[i].length)
				str += (m[i][j]).toString();
			str += '</td>';
		}
		str += '</td>\n';
	}
	str += '</table>\n';

	return str;
}
function Projection(lon0, lat0) {
	this.PROJECTION = Projection.PROJ_TAN;
	this.ROT = this.tr_oR(lon0, lat0);
}

//var ROT;
//var PROJECTION = Projection.PROJ_TAN;	// Default projection


Projection.PROJ_TAN = 1;	/* Gnomonic projection*/
Projection.PROJ_TAN2 = 2;	/* Stereographic projection*/
Projection.PROJ_STG = 2;	
Projection.PROJ_SIN = 3;	/* Orthographic		*/
Projection.PROJ_SIN2 = 4;	/* Equal-area 		*/
Projection.PROJ_ZEA = 4;	/* Zenithal Equal-area 	*/
Projection.PROJ_ARC = 5;	/* For Schmidt plates	*/
Projection.PROJ_SCHMIDT = 5;	/* For Schmidt plates	*/
Projection.PROJ_AITOFF = 6;	/* Aitoff Projection	*/
Projection.PROJ_AIT = 6;	/* Aitoff Projection	*/
Projection.PROJ_GLS = 7;	/* Global Sin (Sanson)	*/
Projection.PROJ_MERCATOR = 8;
Projection.PROJ_MER = 8;	
Projection.PROJ_LAM = 9;	/* Lambert Projection	*/
Projection.PROJ_LAMBERT = 9;	
Projection.PROJ_TSC = 10;	/* Tangent Sph. Cube	*/
Projection.PROJ_QSC = 11;	/* QuadCube Sph. Cube	*/

Projection.PROJ_LIST = [
	"Mercator",Projection.PROJ_MERCATOR,
	"Gnomonic",Projection.PROJ_TAN,
	"Stereographic",Projection.PROJ_TAN2,
	"Orthographic",Projection.PROJ_SIN,
	"Zenithal",Projection.PROJ_ZEA,
	"Schmidt",Projection.PROJ_SCHMIDT,
	"Aitoff",Projection.PROJ_AITOFF,
	"Lambert",Projection.PROJ_LAMBERT,
//	"Tangential",Projection.PROJ_TSC,
//	"Quadrilaterized",Projection.PROJ_QSC,
];
Projection.PROJ_NAME = [
	'-', 'Gnomonic', 'Stereographic', 'Orthographic', 'Equal-area', 'Schmidt plates',
	'Aitoff', 'Global sin', 'Mercator', 'Lambert'
];

Projection.prototype = { 
	
	/** Set the center of the projection
	 * 
	 * (ajout T. Boch, 19/02/2013)
	 * 
	 * */
	setCenter: function(lon0, lat0) {
		this.ROT = this.tr_oR(lon0, lat0);
	},
	
	/**
	 * Set the projection to use
	 * p = projection code
	 */
	setProjection: function(p) {
		this.PROJECTION = p;
	},


	/**
	 * Computes the projection of 1 point : ra,dec => X,Y
	 * alpha, delta = longitude, lattitude
	 */
	project: function(alpha, delta) {
		var u1 = this.tr_ou(alpha, delta);	// u1[3]
		var u2 = this.tr_uu(u1, this.ROT);	// u2[3]
		var P = this.tr_up(this.PROJECTION, u2);	// P[2] = [X,Y]
		if (P == null) {
			return null;
		}

		return { X: -P[0], Y: -P[1] };
	},

	/**
	 * Computes the coordinates from a projection point : X,Y => ra,dec
	 * return o = [ ra, dec ]
	 */
	unproject: function(X,Y) {
		X = -X; Y = -Y;
		var u1 = this.tr_pu(this.PROJECTION, X, Y);	// u1[3]
		var u2 = this.tr_uu1(u1, this.ROT);	// u2[3]
		var o = this.tr_uo(u2);	// o[2]

		return { ra: o[0], dec: o[1] };
	},

	/**
	 * Compute projections from unit vector
	 * The center of the projection correspond to u = [1, 0, 0)
	 * proj = projection system (integer code like _PROJ_MERCATOR_
	 * u[3] = unit vector
	 * return: an array [x,y] or null
	 */
	tr_up: function(proj, u) {
		var x = u[0]; var y = u[1]; var z = u[2];
		var r, den;
		var pp;
		var X,Y;

		r = AstroMath.hypot(x,y);			// r = cos b
		if (r == 0.0 && z == 0.0) return null;

		switch(proj) {
			default:
				pp = null;
				break;

			case Projection.PROJ_AITOFF:
				den = Math.sqrt(r*(r+x)/2.0); 		// cos b . cos l/2
				X = Math.sqrt(2.0*r*(r-x));
				den = Math.sqrt((1.0 + den)/2.0); 
				X = X / den;
				Y = z / den;
				if (y < 0.0) X = -X;
				pp = [ X, Y];
				break;

			case Projection.PROJ_GLS:
				Y = Math.asin(z);				// sin b
				X = (r != 0) ? Math.atan2(y,x)*r : 0.0;
				pp = [ X, Y];
				break;

			case Projection.PROJ_MERCATOR:
				if (r != 0) {
					X = Math.atan2(y,x);
					Y = AstroMath.atanh(z);
					pp = [ X, Y];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_TAN:
				if (x > 0.0) {
					X = y/x;
					Y = z/x;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_TAN2:
				den = (1.0 + x)/2.0;
				if (den > 0.0)	{
					X = y/den;
					Y = z/den;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
			 	break;

			case Projection.PROJ_ARC:
				if (x <= -1.0) {
					// Distance of 180 degrees
					X = Math.PI
					Y = 0.0;
				} else {
					// Arccos(x) = Arcsin(r)
					r = AstroMath.hypot(y,z);
					if (x > 0.0) den = AstroMath.asinc(r);
					else den = Math.acos(x)/r;
					X = y * den;
					Y = z * den;
				}
				pp = [ X, Y ];
				break;

			case Projection.PROJ_SIN:
				if (x >= 0.0) {
					X = y;
					Y = z;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_SIN2:	// Always possible
				den = Math.sqrt((1.0 + x)/2.0);
				if (den != 0)	{
					X = y / den;
					Y = z / den;
				} else {
					// For x = -1
					X = 2.0;
					Y = 0.0;
				}
				pp = [ X, Y ];
				break;

			case Projection.PROJ_LAMBERT:	// Always possible
				Y = z;
				X = 0;
				if (r != 0)	X = Math.atan2(y,x);
				pp = [ X, Y ];
				break;
	  }
	  return pp;
	},

	/**
	 * Computes Unit vector from a position in projection centered at position (0,0).
	 * proj = projection code
	 * X,Y : coordinates of the point in the projection
	 * returns : the unit vector u[3] or a face number for cube projection. 
	 *           null if the point is outside the limits, or if the projection is unknown.
	 */
	tr_pu: function( proj, X, Y ) {
		var r,s,x,y,z;

		switch(proj) {
			default:
			return null;

			case Projection.PROJ_AITOFF:
				// Limit is ellipse with axises 
				// a = 2 * sqrt(2) ,  b = sqrt(2)
				// Compute dir l/2, b
				r = X*X/8.e0 + Y*Y/2.e0; 	// 1 - cos b . cos l/2
				if (r > 1.0) {
	  				// Test outside domain */
					return null;
				}
				x = 1.0 - r ;	// cos b . cos l/2
				s = Math.sqrt(1.0 - r/2.0) ;	// sqrt(( 1 + cos b . cos l/2)/2)
				y = X * s / 2.0;
				z = Y * s ;
				// From (l/2,b) to (l,b)
				r = AstroMath.hypot( x, y ) ;	// cos b
				if (r != 0.0) {
					s = x;
					x = (s*s - y*y) /r;
					y = 2.0 * s * y/r;
				}
				break;

			case Projection.PROJ_GLS:
				// Limit is |Y| <= pi/2
				z = Math.sin(Y);
				r = 1 - z*z;		// cos(b) ** 2
				if (r < 0.0) {
					return null;
				}
				r = Math.sqrt(r);		// cos b
				if (r != 0.0) {
					s = X/r;	// Longitude
				} else {
					s = 0.0;	// For poles
				}
				x = r * Math.cos(s);
				y = r * Math.sin(s);
				break;

			case Projection.PROJ_MERCATOR:
				z = AstroMath.tanh(Y);
				r = 1.0/AstroMath.cosh(Y);
				x = r * Math.cos(X);
				y = r * Math.sin(X);
				break;

			case Projection.PROJ_LAMBERT:
				// Always possible
				z = Y;
				r = 1 - z*z;		// cos(b) ** 2
				if (r < 0.0) {
					return null;
				}
				r = Math.sqrt(r);		// cos b
				x = r * Math.cos(X);
				y = r * Math.sin(X);
				break;
	
			case Projection.PROJ_TAN:
				// No limit
				x = 1.0 / Math.sqrt(1.0 + X*X + Y*Y);
				y = X * x;
				z = Y * x;
				break;

			case Projection.PROJ_TAN2:
				// No limit
				r = (X*X + Y*Y)/4.0;
				s = 1.0 + r;
				x = (1.0 - r)/s;
				y = X / s;
				z = Y / s;
				break;

			case Projection.PROJ_ARC:
				// Limit is circle, radius PI
				r = AstroMath.hypot(X, Y);
				if (r > Math.PI) {
					return null;
				}
				s = AstroMath.sinc(r);
				x = Math.cos(r);
				y = s * X;
				z = s * Y;
				break;

			case Projection.PROJ_SIN:
				// Limit is circle, radius 1
				s = 1.0 - X*X - Y*Y;
				if (s < 0.0) {
					return null;
				}
				x = Math.sqrt(s);
				y = X;
				z = Y;
				break;

			case Projection.PROJ_SIN2:
				// Limit is circle, radius 2	*/
				r = (X*X + Y*Y)/4.e0;
				if (r > 1.0) {
					return null;
				}
				s = Math.sqrt(1.0 - r);
				x = 1.0 - 2.0 * r;
				y = s * X;
				z = s * Y;
				break;
	  }
	  return [ x,y,z ];
	},

	/**
	 * Creates the rotation matrix R[3][3] defined as
	 * R[0] (first row) = unit vector towards Zenith
	 * R[1] (second row) = unit vector towards East
	 * R[2] (third row) = unit vector towards North
	 * o[2] original angles
	 * @return rotation matrix
	 */
	tr_oR: function(lon, lat) {
		var R = new Array(3);
		R[0] = new Array(3);
		R[1] = new Array(3);
		R[2] = new Array(3);
		R[2][2] =  AstroMath.cosd(lat);
		R[0][2] =  AstroMath.sind(lat);
		R[1][1] =  AstroMath.cosd(lon);
		R[1][0] =  -AstroMath.sind(lon);
		R[1][2] =  0.0;
		R[0][0] =  R[2][2] * R[1][1];  
		R[0][1] = -R[2][2] * R[1][0];
		R[2][0] = -R[0][2] * R[1][1];
		R[2][1] =  R[0][2] * R[1][0];
		return R;
	},

	/**
	 * Transformation from polar coordinates to Unit vector
	 * @return U[3]
	 */
	tr_ou: function(ra, dec) {
		var u = new Array(3);
		var cosdec = AstroMath.cosd(dec);

		u[0] = cosdec * AstroMath.cosd(ra);
		u[1] = cosdec * AstroMath.sind(ra);
		u[2] = AstroMath.sind(dec);

		return u;
	},

	/**
	 * Rotates the unit vector u1 using the rotation matrix
	 * u1[3] unit vector
	 * R[3][3] rotation matrix
	 * return resulting unit vector u2[3]
	 */
	tr_uu: function( u1, R ) {
		var u2 = new Array(3);
		var x = u1[0];
		var y = u1[1];
		var z = u1[2];

		u2[0] = R[0][0]*x + R[0][1]*y + R[0][2]*z ;
		u2[1] = R[1][0]*x + R[1][1]*y + R[1][2]*z ;
		u2[2] = R[2][0]*x + R[2][1]*y + R[2][2]*z ;

		return u2;
	},

	/**
	 * reverse rotation the unit vector u1 using the rotation matrix
	 * u1[3] unit vector
	 * R[3][3] rotation matrix
	 * return resulting unit vector u2[3]
	 */
	tr_uu1: function( u1 , R) {
		var u2 = new Array(3);
		var x = u1[0];
		var y = u1[1];
		var z = u1[2];

		u2[0] = R[0][0]*x + R[1][0]*y + R[2][0]*z;
		u2[1] = R[0][1]*x + R[1][1]*y + R[2][1]*z;
		u2[2] = R[0][2]*x + R[1][2]*y + R[2][2]*z;

		return u2;
	},

	/**
	 * Computes angles from direction cosines
	 * u[3] = direction cosines vector
	 * return o = [ ra, dec ]
	 */
	tr_uo: function(u) {
		var x = u[0]; var y = u[1]; var z = u[2];  
		var r2 = x*x + y*y;
		var ra, dec;
		if (r2  == 0.0) {
	 		// in case of poles
			if (z == 0.0) {
				return null;
			}
			ra = 0.0;
			dec = z > 0.0 ? 90.0 : -90.0;
		} else {
			dec = AstroMath.atand( z / Math.sqrt(r2));
			ra  = AstroMath.atan2d (y , x );
			if (ra < 0.0) ra += 360.0;
		}

		return [ ra, dec ];
	}
}//=================================
// Class Coo
//=================================

/**
 * Constructor
 * @param longitude longitude (decimal degrees)
 * @param latitude latitude (decimal degrees)
 * @param prec precision
 * (8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 */
function Coo(longitude, latitude, prec) {
	this.lon = longitude;
	this.lat = latitude;
	this.prec = prec;
	this.frame = null;

	this.computeDirCos();
}

Coo.factor = [ 3600.0, 60.0, 1.0 ];
Coo.prototype = {
	setFrame: function(astroframe) {
		this.frame = astroframe;
	},
	computeDirCos: function() {
		var coslat = AstroMath.cosd(this.lat);

		this.x = coslat*AstroMath.cosd(this.lon);
		this.y = coslat*AstroMath.sind(this.lon);
		this.z = AstroMath.sind(this.lat);	
	}, 
	computeLonLat: function() {
		var r2 = this.x*this.x+this.y*this.y;
		this.lon = 0.0;
		if (r2 == 0.0) {
			// In case of poles
			if (this.z == 0.0) {
				this.lon = 0.0/0.0;
				this.lat = 0.0/0.0;
			} else {
				this.lat = (this.z > 0.0) ? 90.0 : -90.0;
			}
		} else {
			this.lon = AstroMath.atan2d(this.y, this.x);
			this.lat = AstroMath.atan2d(this.z, Math.sqrt(r2));
			if (this.lon < 0) this.lon += 360.0;
		}
	},

  /**
    * Squared distance between 2 points (= 4.sin<sup>2</sup>(r/2))
    * @param  pos      another position on the sphere
    * @return ||pos-this||<sup>2</sup> = 4.sin<sup>2</sup>(r/2)
   **/
   dist2: function(pos) {
//    	if ((this.x==0)&&(this.y==0)&&(this.z==0)) return(0./0.);
//    	if ((pos.x==0)&&(pos.y==0)&&(pos.z==0)) return(0./0.);
	var w = pos.x - this.x;
	var r2 = w * w;
	w = pos.y - this.y; r2 += w * w;
	w = pos.z - this.z; r2 += w * w;
	return r2;
   },

   /**
    * Distance between 2 points on the sphere.
    * @param  pos another position on the sphere
    * @return distance in degrees in range [0, 180]
   **/
    distance: function(pos) {
      // Take care of NaN:
    	if ((pos.x==0)&&(pos.y==0)&&(pos.z==0)) return(0./0.);
    	if ((this.x==0)&&(this.y==0)&&(this.z==0)) return(0./0.);
      return (2. * AstroMath.asind(0.5 * Math.sqrt(this.dist2(pos))));
    },

   /**
    * Transform the position into another frame.
    * @param new_frame	The frame of the resulting position.
   **/
   convertTo: function(new_frame) {
		// Verify first if frames identical -- then nothing to do !
		if (this.frame.equals(new_frame)) {
	    		return;
		}

		// Move via ICRS
		this.frame.toICRS(this.coo);	// Position now in ICRS
		new_frame.fromICRS(this.coo);	// Position now in new_frame
		this.frame = new_frame;
		this.lon = this.lat = 0./0.;	// Actual angles not recomputed
   },

    /**
     * Rotate a coordinate (apply a rotation to the position).
     * @param R [3][3] Rotation Matrix
     */
    rotate: function(R) {
      var X, Y, Z;
		if (R == Umatrix3) return;
		X = R[0][0]*this.x + R[0][1]*this.y + R[0][2]*this.z;
		Y = R[1][0]*this.x + R[1][1]*this.y + R[1][2]*this.z;
		Z = R[2][0]*this.x + R[2][1]*this.y + R[2][2]*this.z;
    	// this.set(X, Y, Z); Not necessary to compute positions each time.
		this.x = X; this.y = Y; this.z = Z;
		this.lon = this.lat = 0./0.;
    },

    /**
     * Rotate a coordinate (apply a rotation to the position) in reverse direction.
     * The method is the inverse of rotate.
     * @param R [3][3] Rotation Matrix
     */
    rotate_1: function(R) {
      var X, Y, Z;
      if (R == Umatrix3) return;
		X = R[0][0]*this.x + R[1][0]*this.y + R[2][0]*this.z;
		Y = R[0][1]*this.x + R[1][1]*this.y + R[2][1]*this.z;
		Z = R[0][2]*this.x + R[1][2]*this.y + R[2][2]*this.z;
    	// this.set(X, Y, Z); Not necessary to compute positions each time.
		this.x = X; this.y = Y; this.z = Z;
		this.lon = this.lat = 0./0.;
    },


    /**
     * Test equality of Coo.
     * @param coo Second coordinate to compare with
     * @return  True if the two coordinates are equal
     */
    equals: function(coo) {
		return this.x == coo.x && this.y == coo.y && this.z == coo.z;
    },

	/**
	 * parse a coordinate string. The coordinates can be in decimal or sexagesimal
	 * @param str string to parse
	 * @return true if the parsing succeded, false otherwise
	 */
	parse: function(str) {
		var p = str.indexOf('+');
		if (p < 0) p = str.indexOf('-');
		if (p < 0) p = str.indexOf(' ');
		if (p < 0) {
			this.lon = 0.0/0.0;
			this.lat = 0.0/0.0;
			this.prec = 0;
			return false;
		}
		var strlon = str.substring(0,p);
		var strlat = str.substring(p);
	
		this.lon = this.parseLon(strlon);	// sets the precision parameter
		this.lat = this.parseLat(strlat);	// sets the precision parameter
		return true;
	},

	parseLon: function(str) {
		var str = str.trim();
		if (str.indexOf(' ') < 0) {
			// The longitude is a integer or decimal number
			var p = str.indexOf('.');
			this.prec = p < 0 ? 0 : str.length - p - 1;
			return parseFloat(str);
		} else {
			var stok = new Tokenizer(str,' ');
			var i = 0;
			var l = 0;
			var pr = 0;
			while (stok.hasMore()) {
				var tok = stok.nextToken();
				var dec = tok.indexOf('.');
				l += parseFloat(tok)*Coo.factor[i];
//				pr = dec < 0 ? 1 : 2;
				switch (i) {
					case 0: pr = dec < 0 ? 1 : 2; break;
					case 1: pr = dec < 0 ? 3 : 4; break;
					case 2: pr = dec < 0 ? 5 : 4+tok.length-dec;
					default: break;
				}
				i++;
			}
			this.prec = pr;
			return l*15/3600.0;	
		}
	},
			
	parseLat: function(str) {
		var str = str.trim();
		var sign;
		if (str.charAt(0) == '-') {
			sign = -1;
			str = str.substring(1);
		} else if (str.charAt(0) == '-') {
			sign = 1;
			str = str.substring(1);
		} else {
			// No sign specified
			sign = 1;
		}
		if (str.indexOf(' ') < 0) {
			// The longitude is a integer or decimal number
			var p = str.indexOf('.');
			this.prec = p < 0 ? 0 : str.length - p - 1;
			return parseFloat(str)*sign;
		} else {
			var stok = new Tokenizer(str,' ');
			var i = 0;
			var l = 0;
			var pr = 0;
			while (stok.hasMore()) {
				var tok = stok.nextToken();
				var dec = tok.indexOf('.');
				l += parseFloat(tok)*Coo.factor[i];
				switch (i) {
					case 0: pr = dec < 0 ? 1 : 2; break;
					case 1: pr = dec < 0 ? 3 : 4; break;
					case 2: pr = dec < 0 ? 5 : 4+tok.length-dec;
					default: break;
				}
				i++;
			}
			this.prec = pr;
			return l*sign/3600.0;	
		}
	},

	/**
	 * Format coordinates according to the options
	 * @param options 'd': decimal, 's': sexagésimal, '/': space separated, '2': return [ra,dec] in an array
	 * @return the formatted coordinates
	 */
	format: function(options) {
		if (isNaN(this.lon)) this.computeLonLat();
		var strlon = "", strlat = "";
		if (options.indexOf('d') >= 0) {
			// decimal display
			strlon = Numbers.format(this.lon, this.prec);
			strlat = Numbers.format(this.lat, this.prec);
		} else {
			// sexagesimal display
			var hlon = this.lon/15.0;
			var strlon = Numbers.toSexagesimal(hlon, this.prec+1, false);
			var strlat = Numbers.toSexagesimal(this.lat, this.prec, true);
		}
		if (this.lat > 0) strlat = '+'+strlat;

		if (options.indexOf('/') >= 0) {
			return strlon+' '+strlat;
		} else if (options.indexOf('2') >= 0) {
			return [strlon, strlat];
		}
		return strlon+strlat;
	}
		
}

/**
 * Distance between 2 points on the sphere.
 * @param coo1 firs	var coslat = AstroMath.cosd(this.lat);

	this.x = coslat*AstroMath.cosd(this.lon);
	this.y = coslat*AstroMath.sind(this.lon);
	this.z = AstroMath.sind(this.lat);
t coordinates point
 * @param coo2 second coordinates point
 * @return distance in degrees in range [0, 180]
**/
/*
Coo.distance = function(Coo coo1, Coo coo2) {
	return Coo.distance(coo1.lon, coo1.lat, coo2.lon, coo2.lat);
}
*/
/**
 * Distance between 2 points on the sphere.
 * @param lon1 longitude of first point in degrees
 * @param lat1 latitude of first point in degrees
 * @param lon2 longitude of second point in degrees
 * @param lat2 latitude of second point in degrees
 * @return distance in degrees in range [0, 180]
**/
/*
Coo.distance = function(lon1, lat1, lon2, lat2) {
	var c1 = AstroMath.cosd(lat1);
	var c2 = AstroMath.cosd(lat2);

	var w, r2;
	w = c1 * AstroMath.cosd(lon1) - c2 * AstroMath.cosd(lon2);
	r2 = w*w;
	w = c1 * AstroMath.sind(lon1) - c2 * AstroMath.sind(lon2);
	r2 += w*w;
	w = AstroMath.sind(lat1) - AstroMath.sind(lat2);
	r2 += w*w;

	return 2. * AstroMath.asind(0.5 * Math.sqrt(r2));
}


//===================================
// Class Tokenizer (similar to Java)
//===================================

/**
 * Constructor
 * @param str String to tokenize
 * @param sep token separator char
 */
function Tokenizer(str, sep) {
	this.string = Strings.trim(str, sep);
	this.sep = sep;
	this.pos = 0;
}

Tokenizer.prototype = {
	/**
	 * Check if the string has more tokens
	 * @return true if a token remains (read with nextToken())
	 */
	hasMore: function() {
		return this.pos < this.string.length;
	},

	/**
	 * Returns the next token (as long as hasMore() is true)
	 * @return the token string
	 */
	nextToken: function() {
		// skip all the separator chars
		var p0 = this.pos;
		while (p0 < this.string.length && this.string.charAt(p0) == this.sep) p0++;
		var p1 = p0;
		// get the token
		while (p1 < this.string.length && this.string.charAt(p1) != this.sep) p1++;
		this.pos = p1;
		return this.string.substring(p0, p1);
	},
}

//================================
// Class Strings (static methods)
//================================
function Strings() {}

/**
 * Removes a given char at the beginning and the end of a string
 * @param str string to trim
 * @param c char to remove
 * @return the trimmed string
 */

Strings.trim = function(str, c) {
	var p0=0, p1=str.length-1;
	while (p0 < str.length && str.charAt(p0) == c) p0++;
	if (p0 == str.length) return "";
	while (p1 > p0 && str.charAt(p1) == c) p1--;
	return str.substring(p0, p1+1);
}

//================================
// Class Numbers (static methods)
//================================
function Numbers() {}
//                0  1   2    3     4      5       6        7         8          9
Numbers.pow10 = [ 1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
//      10           11            12             13              14
	10000000000, 100000000000, 1000000000000, 10000000000000, 100000000000000 ];
//                 0    1     2      3       4        5         6          7
Numbers.rndval = [ 0.5, 0.05, 0.005, 0.0005, 0.00005, 0.000005, 0.0000005, 0.00000005,
//      8            9             10             11              12
	0.000000005, 0.0000000005, 0.00000000005, 0.000000000005, 0.0000000000005,
//      13                14
	0.00000000000005, 0.00000000000005 ];
/**
 * Format a integer or decimal number, adjusting the value with 'prec' decimal digits
 * @param num number (integer or decimal)
 * @param prec precision (= number of decimal digit to keep or append)
 * @return a string with the formatted number
 */
Numbers.format = function(num, prec) {
		if (prec <= 0) {
			// Return an integer number
			return (Math.round(num)).toString();
		}
		var str = num.toString();
		var p = str.indexOf('.');
		var nbdec = p >= 0 ? str.length-p-1 : 0;
		if (prec >= nbdec) {
			if (p < 0) str += '.';
			for (var i=0; i<prec-nbdec; i++)
				str += '0';
			return str;
		}
		// HERE: prec > 0 and prec < nbdec
		str = (num+Numbers.rndval[prec]).toString();
		return str.substr(0, p+prec+1);
}


/**
 * Convert a decimal coordinate into sexagesimal string, according to the given precision<br>
 * 8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 * @param num number (integer or decimal)
 * @param prec precision (= number of decimal digit to keep or append)
 * @param plus if true, the '+' sign is displayed
 * @return a string with the formatted sexagesimal number
 */
Numbers.toSexagesimal = function(num, prec, plus) {
	var resu = "";
	var sign = num < 0 ? '-' : (plus ? '+' : '');
	var n = Math.abs(num);

	switch (prec) {
		case 1:	// deg
			var n1 = Math.round(n);
			return sign+n1.toString();
		case 2:	// deg.d
			return sign+Numbers.format(n, 1);
		case 3:	// deg min
			var n1 = Math.floor(n);
			var n2 = Math.round((n-n1)*60);
			return sign+n1+' '+n2;
		case 4:	// deg min.d
			var n1 = Math.floor(n);
			var n2 = (n-n1)*60;
			return sign+n1+' '+Numbers.format(n2, 1);
		case 5:	// deg min sec
			var n1 = Math.floor(n);	// d
			var n2 = (n-n1)*60;		// M.d
			var n3 = Math.floor(n2);// M
			var n4 = Math.round((n2-n3)*60);	// S
			return sign+n1+' '+n3+' '+n4;
		case 6:	// deg min sec.d
		case 7:	// deg min sec.dd
		case 8:	// deg min sec.ddd
			var n1 = Math.floor(n);	// d
			if (n1<10) n1 = '0' + n1;
			var n2 = (n-n1)*60;		// M.d
			var n3 = Math.floor(n2);// M
			if (n3<10) n3 = '0' + n3;
			var n4 = (n2-n3)*60;		// S.ddd
			return sign+n1+' '+n3+' '+Numbers.format(n4, prec-5);
		default:
			return sign+Numbers.format(n, 1);
	}
}
CooConversion = (function() {

    var CooConversion = {};
    
    CooConversion.GALACTIC_TO_J2000 = [
       -0.0548755604024359,  0.4941094279435681, -0.8676661489811610,
       -0.8734370902479237, -0.4448296299195045, -0.1980763734646737,
       -0.4838350155267381,  0.7469822444763707,  0.4559837762325372 ];
    
    CooConversion.J2000_TO_GALACTIC = [
        -0.0548755604024359, -0.873437090247923, -0.4838350155267381,
         0.4941094279435681, -0.4448296299195045, 0.7469822444763707,
        -0.8676661489811610, -0.1980763734646737, 0.4559837762325372 ];
    
    // adapted from www.robertmartinayers.org/tools/coordinates.html
    // radec : array of ra, dec in degrees
    // return coo in degrees
    CooConversion.Transform = function( radec, matrix ) {// returns a radec array of two elements
        radec[0] = radec[0]*Math.PI/180;
        radec[1] = radec[1]*Math.PI/180;
      var r0 = new Array ( 
       Math.cos(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[1]) );
        
     var s0 = new Array (
       r0[0]*matrix[0] + r0[1]*matrix[1] + r0[2]*matrix[2], 
       r0[0]*matrix[3] + r0[1]*matrix[4] + r0[2]*matrix[5], 
       r0[0]*matrix[6] + r0[1]*matrix[7] + r0[2]*matrix[8] ); 
     
      var r = Math.sqrt ( s0[0]*s0[0] + s0[1]*s0[1] + s0[2]*s0[2] ); 
    
      var result = new Array ( 0.0, 0.0 );
      result[1] = Math.asin ( s0[2]/r ); // New dec in range -90.0 -- +90.0 
      // or use sin^2 + cos^2 = 1.0  
      var cosaa = ( (s0[0]/r) / Math.cos(result[1] ) );
      var sinaa = ( (s0[1]/r) / Math.cos(result[1] ) );
      result[0] = Math.atan2 (sinaa,cosaa);
      if ( result[0] < 0.0 ) result[0] = result[0] + 2*Math.PI;
    
        result[0] = result[0]*180/Math.PI;
        result[1] = result[1]*180/Math.PI;
      return result;
    };
    
    // coo : array of lon, lat in degrees
    CooConversion.GalacticToJ2000 = function(coo) {
        return CooConversion.Transform(coo, CooConversion.GALACTIC_TO_J2000);
    };
    // coo : array of lon, lat in degrees
    CooConversion.J2000ToGalactic = function(coo) {
        return CooConversion.Transform(coo, CooConversion.J2000_TO_GALACTIC);
    };
    return CooConversion;
})();/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Sesame = (function() {
    Sesame = {};
    
    Sesame.cache = {};
    
    Sesame.resolve = function(objectName, callbackFunctionSuccess, callbackFunctionError) {
        //var sesameUrl = "http://cdsportal.u-strasbg.fr/services/sesame?format=json";
        var sesameUrl = "http://cds.u-strasbg.fr/cgi-bin/nph-sesame.jsonp?";
        $.ajax({
            url: sesameUrl ,
            data: {"object": objectName},
            method: 'GET',
            dataType: 'jsonp',
            success: function(data) {
                if (data.Target && data.Target.Resolver && data.Target.Resolver) {
                    callbackFunctionSuccess(data);
                }
                else {
                    callbackFunctionError(data);
                }
            },
            error: callbackFunctionError
            });
    };
    
    return Sesame;
})();

/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HealpixCache
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// class holding some HEALPix computations for better performances
//
// it is made of :
// - a static cache for HEALPix corners at nside=8 
// - a dynamic cache for 
HealpixCache = (function() {

    var HealpixCache = {};
    
    HealpixCache.staticCache = {corners: {nside8: []}};
    // TODO : utilisation du dynamicCache
    HealpixCache.dynamicCache = {};
    
    HealpixCache.lastNside = 8;
    
    HealpixCache.hpxIdxCache = null;
    
    // TODO : conserver en cache le dernier résultat ?
    
    HealpixCache.init = function() {
    	// pre-compute corners position for nside=8
    	var hpxIdx = new HealpixIndex(8);
    	hpxIdx.init();
    	var npix = HealpixIndex.nside2Npix(8);
    	for (var ipix=0; ipix<npix; ipix++) {
    		HealpixCache.staticCache.corners.nside8[ipix] = hpxIdx.corners_nest(ipix, 1);
    	}
    	
    	HealpixCache.hpxIdxCache = hpxIdx;
    };
    
    HealpixCache.corners_nest = function(ipix, nside) {
    	if (nside==8) {
    		return HealpixCache.staticCache.corners.nside8[ipix];
    	}
    	
    	if (nside != HealpixCache.lastNside) {
    		HealpixCache.hpxIdxCache = new HealpixIndex(nside);
    		HealpixCache.hpxIdxCache.init();
    		HealpixCache.lastNside = nside;
    	}
    	
    	return HealpixCache.hpxIdxCache.corners_nest(ipix, 1);
    	
    };
    
    return HealpixCache;
})();
	
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Utils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Utils = Utils || {};

Utils.cssScale = undefined;
// adding relMouseCoords to HTMLCanvasElement prototype (see http://stackoverflow.com/questions/55677/how-do-i-get-the-coordinates-of-a-mouse-click-on-a-canvas-element ) 
function relMouseCoords(event){
    var totalOffsetX = 0;
    var totalOffsetY = 0;
    var canvasX = 0;
    var canvasY = 0;
    var currentElement = this;
   

    if (event.offsetX) {
        return {x: event.offsetX, y:event.offsetY};
    } 
    else {
        if (!Utils.cssScale) {
            var st = window.getComputedStyle(document.body, null);
            var tr = st.getPropertyValue("-webkit-transform") ||
                    st.getPropertyValue("-moz-transform") ||
                    st.getPropertyValue("-ms-transform") ||
                    st.getPropertyValue("-o-transform") ||
                    st.getPropertyValue("transform");
            var matrixRegex = /matrix\((-?\d*\.?\d+),\s*0,\s*0,\s*(-?\d*\.?\d+),\s*0,\s*0\)/;
            var matches = tr.match(matrixRegex);
            if (matches) {
                Utils.cssScale = parseFloat(matches[1]);
            }
            else {
                Utils.cssScale = 1;
            }
        }
        var e = event;
        var canvas = e.target;
        // http://www.jacklmoore.com/notes/mouse-position/
        var target = e.target || e.srcElement,
        style = target.currentStyle || window.getComputedStyle(target, null),
        borderLeftWidth = parseInt(style['borderLeftWidth'], 10),
        borderTopWidth = parseInt(style['borderTopWidth'], 10),
        rect = target.getBoundingClientRect(),
        offsetX = e.clientX - borderLeftWidth - rect.left,
        offsetY = e.clientY - borderTopWidth - rect.top;
        return {x: parseInt(offsetX/Utils.cssScale), y: parseInt(offsetY/Utils.cssScale)};
    }

    // TODO : should we cache the value of scrollLeft/scrollTop to prevent a reflow ? (cf. http://www.phpied.com/rendering-repaint-reflowrelayout-restyle/ )
    do {
        totalOffsetX += currentElement.offsetLeft - currentElement.scrollLeft;
        totalOffsetY += currentElement.offsetTop - currentElement.scrollTop;
    }
    while(currentElement = currentElement.offsetParent)
        

    // NB: Chrome seems to always use document.body.scrollTop whereas Firefox sometimes use document.documentElement.scrollTop
    if (event.pageX) {
        canvasX = event.pageX - totalOffsetX - (document.body.scrollLeft || document.documentElement.scrollLeft);
        canvasY = event.pageY - totalOffsetY - (document.body.scrollTop || document.documentElement.scrollTop);
    }
    // if touch events
    else {
        canvasX = event.originalEvent.targetTouches[0].screenX - totalOffsetX - (document.body.scrollLeft || document.documentElement.scrollLeft);
        canvasY = event.originalEvent.targetTouches[0].screenY - totalOffsetY - (document.body.scrollTop || document.documentElement.scrollTop);
    }


    

    return {x: canvasX, y: canvasY};
    //return {x: parseInt(canvasX/Utils.cssScale), y: parseInt(canvasY/Utils.cssScale)};
}
HTMLCanvasElement.prototype.relMouseCoords = relMouseCoords;



//Function.prototype.bind polyfill from 
//https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Function/bind
if (!Function.prototype.bind) {
    Function.prototype.bind = function (obj) {
        // closest thing possible to the ECMAScript 5 internal IsCallable function
        if (typeof this !== 'function') {
            throw new TypeError('Function.prototype.bind - what is trying to be bound is not callable');
        }

        var slice = [].slice,
        args = slice.call(arguments, 1),
        self = this,
        nop = function () { },
        bound = function () {
            return self.apply(this instanceof nop ? this : (obj || {}),
                    args.concat(slice.call(arguments)));
        };

        bound.prototype = this.prototype;

        return bound;
    };
}








$ = $ || jQuery;

/* source : http://stackoverflow.com/a/8764051 */
$.urlParam = function(name, queryString){
    if (queryString===undefined) {
        queryString = location.search;
    }
	return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(queryString)||[,""])[1].replace(/\+/g, '%20'))||null;
};

Utils.isNumber = function(n) {
  return !isNaN(parseFloat(n)) && isFinite(n);
};

/* a debounce function, used to prevent multiple calls to the same function if less than delay seconds have passed */
Utils.debounce = function(fn, delay) {
    var timer = null;
    return function () {
      var context = this, args = arguments;
      clearTimeout(timer);
      timer = setTimeout(function () {
        fn.apply(context, args);
      }, delay);
    };
};


/* A LRU cache, inspired by https://gist.github.com/devinus/409353#file-gistfile1-js */
// TODO : utiliser le LRU cache pour les tuiles images
Utils.LRUCache = function (maxsize) {
    this._keys = [];
    this._items = {};
    this._expires = {};
    this._size = 0;
    this._maxsize = maxsize || 1024;
};
   
Utils.LRUCache.prototype = {
        set: function (key, value) {
            var keys = this._keys,
                items = this._items,
                expires = this._expires,
                size = this._size,
                maxsize = this._maxsize;

            if (size >= maxsize) { // remove oldest element when no more room
                keys.sort(function (a, b) {
                    if (expires[a] > expires[b]) return -1;
                    if (expires[a] < expires[b]) return 1;
                    return 0;
                });

                size--;
                delete expires[keys[size]];
                delete items[keys[size]];
            }

            keys[size] = key;
            items[key] = value;
            expires[key] = Date.now();
            size++;

            this._keys = keys;
            this._items = items;
            this._expires = expires;
            this._size = size;
        },

        get: function (key) {
            var item = this._items[key];
            if (item) this._expires[key] = Date.now();
            return item;
        },
        
        keys: function() {
            return this._keys;
        }
};
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Color
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


    Color = {};
    
    Color.curIdx = 0;
    Color.colors = ['#ff0000', '#0000ff', '#99cc00', '#ffff00','#000066', '#00ffff', '#9900cc', '#0099cc', '#cc9900', '#cc0099', '#00cc99', '#663333', '#ffcc9a', '#ff9acc', '#ccff33', '#660000', '#ffcc33', '#ff00ff', '#00ff00', '#ffffff'];

    
    Color.getNextColor = function() {
        var c = Color.colors[Color.curIdx % (Color.colors.length)];
        Color.curIdx++;
        return c;
    };
    

/******************************************************************************
 * Aladin HTML5 project
 * 
 * File AladinUtils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
AladinUtils = (function() {
    return {
    	/**
    	 * passage de xy projection à xy dans la vue écran 
    	 * @param x
    	 * @param y
    	 * @param width
    	 * @param height
    	 * @param largestDim largest dimension of the view
    	 * @returns position in the view
    	 */
    	xyToView: function(x, y, width, height, largestDim, zoomFactor) {
    		// we round the result for performance gains
    		return {vx: AladinUtils.myRound(largestDim/2*(1+zoomFactor*x)-(largestDim-width)/2), vy: AladinUtils.myRound(largestDim/2*(1+zoomFactor*y)-(largestDim-height)/2)};
    	},
    	
    	/**
    	 * passage de xy dans la vue écran à xy projection
    	 * @param vx
    	 * @param vy
    	 * @param width
    	 * @param height
    	 * @param largestDim
    	 * @param zoomFactor
    	 * @returns position in xy projection
    	 */
    	viewToXy: function(vx, vy, width, height, largestDim, zoomFactor) {
    		return {x: ((2*vx+(largestDim-width))/largestDim-1)/zoomFactor, y: ((2*vy+(largestDim-height))/largestDim-1)/zoomFactor};
    	},
    	
    	myRound: function(a) {
    		if (a<0) {
    			return -1*( (-a) | 0);
    		}
    		else {
    			return a | 0;
    		}
    	},
    	
    	/**
    	 * tests whether a healpix pixel is visible or not
    	 * @param pixCorners array of position (xy view) of the corners of the pixel
    	 * @param viewW
    	 */
    	isHpxPixVisible: function(pixCorners, viewWidth, viewHeight) {
    		for (var i = 0; i<pixCorners.length; i++) {
    			if ( pixCorners[i].vx>=-20 && pixCorners[i].vx<(viewWidth+20) &&
    				 pixCorners[i].vy>=-20 && pixCorners[i].vy<(viewHeight+20) ) {
    				return true;
    			}
    		}
    		return false;
    	},
    	
    	ipixToIpix: function(npixIn, norderIn, norderOut) {
    		var npixOut = [];
    		if (norderIn>=norderOut) {
    		}
    	},
        
        getZoomFactorForAngle: function(angleInDegrees, projectionMethod) {
            var p1 = {ra: 0, dec: 0};
            var p2 = {ra: angleInDegrees, dec: 0};
            var projection = new Projection(angleInDegrees/2, 0);
            projection.setProjection(projectionMethod);
            var p1Projected = projection.project(p1.ra, p1.dec);
            var p2Projected = projection.project(p2.ra, p2.dec);
           
            var zoomFactor = 1/(p1Projected.X - p2Projected.Y);
            return zoomFactor;
        }
    	
    };
})();

/******************************************************************************
 * Aladin HTML5 project
 * 
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
 ProjectionEnum = {
    SIN: Projection.PROJ_SIN,
    AITOFF:  Projection.PROJ_AITOFF
 };/******************************************************************************
 * Aladin HTML5 project
 * 
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
CooFrameEnum = (function() {

    return {
        J2000: "J2000",
        GAL:  "Galactic"
    };
 
})();

// TODO : utiliser cette fonction partout où on reçoit une string frame en entrée
CooFrameEnum.fromString = function(str, defaultValue) {
    if (! str) {
        return defaultValue ? defaultValue : null;
    }
    
    str = str.toLowerCase().replace(/^\s+|\s+$/g, ''); // convert to lowercase and trim
    
    if (str.indexOf('j2000')==0 || str.indexOf('icrs')==0) {
        return CooFrameEnum.J2000;
    }
    else if (str.indexOf('gal')==0) {
        return CooFrameEnum.GAL;
    }
    else {
        return defaultValue ? defaultValue : null;
    }
};/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Downloader
 * Queue downloading for image elements
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Downloader = (function() {
	var NB_MAX_SIMULTANEOUS_DL = 4;
	// TODO : le fading ne marche pas bien actuellement
	var FADING_ENABLED = false;
	var FADING_DURATION = 700; // in milliseconds
	
	
	var Downloader = function(view) {
		this.view = view; // reference to the view to be able to request redraw
		this.nbDownloads = 0; // number of current downloads
		this.dlQueue = []; // queue of items being downloaded
        this.urlsInQueue = {};
	};
	
	Downloader.prototype.requestDownload = function(img, url, cors) {
        // first check if url already in queue
        if (url in this.urlsInQueue)  {
            return;
        }
		// put in queue
		this.dlQueue.push({img: img, url: url, cors: cors});
		this.urlsInQueue[url] = 1;
		
		this.tryDownload();
	};
	
	// try to download next items in queue if possible
	Downloader.prototype.tryDownload = function() {
	    //if (this.dlQueue.length>0 && this.nbDownloads<NB_MAX_SIMULTANEOUS_DL) {
		while (this.dlQueue.length>0 && this.nbDownloads<NB_MAX_SIMULTANEOUS_DL) {
			this.startDownloadNext();
		}
	};
	
	Downloader.prototype.startDownloadNext = function() {
		// get next in queue
		var next = this.dlQueue.shift();
		if ( ! next) {
			return;
		}

		this.nbDownloads++;
		var downloaderRef = this;
		next.img.onload = function() {
			downloaderRef.completeDownload(this, true); // in this context, 'this' is the Image
		};
			
		next.img.onerror = function(e) {
			downloaderRef.completeDownload(this, false); // in this context, 'this' is the Image
		};
		if (next.cors) {
		    next.img.crossOrigin = 'anonymous';
		}
		
		else {
		    if (next.img.crossOrigin !== undefined) {
		        delete next.img.crossOrigin;
		    }
		}
		
		
		next.img.src = next.url;
	};
	
	Downloader.prototype.completeDownload = function(img, success) {
        delete this.urlsInQueue[img.src];
		img.onerror = null;
		img.onload = null;
		this.nbDownloads--;
		if (success) {
			if (FADING_ENABLED) {
				var now = new Date().getTime();
				img.fadingStart = now;
				img.fadingEnd = now + FADING_DURATION;
			}
			this.view.requestRedraw();
		}
		else {
		    img.dlError = true;
		}
		
		this.tryDownload();
	};
	
	
	
	return Downloader;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Footprint
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Footprint = (function() {
    // constructor
    Footprint = function(polygons) {
        this.polygons = polygons;
    	this.overlay = null;
    	
    	this.isShowing = true;
    	this.isSelected = false;
    };
    
    Footprint.prototype.setOverlay = function(overlay) {
        this.overlay = overlay;
    };
    
    Footprint.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Footprint.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Footprint.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    Footprint.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.overlay) {
            this.overlay.reportChange();
        }
    };
    
    return Footprint;
})();
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
    
/******************************************************************************
 * Aladin Lite project
 * 
 * File Overlay
 *
 * Description: a plane holding overlays (footprints, polylines)
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Overlay = (function() {
   Overlay = function(options) {
        options = options || {};
    	this.name = options.name || "overlay";
    	this.color = options.color || Color.getNextColor();
        
    	
    	//this.indexationNorder = 5; // à quel niveau indexe-t-on les overlays
    	this.overlays = [];
    	//this.hpxIdx = new HealpixIndex(this.indexationNorder);
    	//this.hpxIdx.init();
    	
    	

    };
    

    
    
    // return an array of Footprint from a STC-S string
    Overlay.parseSTCS = function(stcs) {
        var polygons = [];
        var parts = stcs.match(/\S+/g);
        var k = 0, len = parts.length;
        var curPolygon;
        while(k<len) {
            var s = parts[k].toLowerCase();
            if(s=='polygon') {
                curPolygon = [];
                k++;
                frame = parts[k].toLowerCase();
                if (frame=='icrs' || frame=='j2000') {
                    while(k+2<len) {
                        var ra = parseFloat(parts[k+1]);
                        if (isNaN(ra)) {
                            break;
                        }
                        var dec = parseFloat(parts[k+2]);
                        curPolygon.push([ra, dec]);
                        k += 2;
                    }
                    curPolygon.push(curPolygon[0]);
                    polygons.push(curPolygon);
                }
            }
            k++;
        }

        return polygons;
    };
    
    // ajout d'un tableau d'overlays (= footprints)
    Overlay.prototype.addFootprints = function(overlaysToAdd) {
    	this.overlays = this.overlays.concat(overlaysToAdd);
    	for (var k=0, len=overlaysToAdd.length; k<len; k++) {
    	    overlaysToAdd[k].setOverlay(this);
    	}
        this.view.requestRedraw();
    };
    
    // return a footprint by index
   Overlay.prototype.getFootprint = function(idx) {
        if (idx<this.footprints.length) {
            return this.footprints[idx];
        }
        else {
            return null;
        }
    };
    
    Overlay.prototype.setView = function(view) {
        this.view = view;
    };
    
    Overlay.prototype.removeAll = function() {
        // TODO : RAZ de l'index
        this.overlays = [];
    };
    
    Overlay.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        // tracé simple
        ctx.strokeStyle= this.color;

        ctx.lineWidth = 2;
    	ctx.beginPath();
    	xyviews = [];
    	for (var k=0, len = this.overlays.length; k<len; k++) {
    		xyviews.push(this.drawFootprint(this.overlays[k], ctx, projection, frame, width, height, largestDim, zoomFactor));
    	}
        ctx.stroke();

    	// tracé sélection
        ctx.strokeStyle= Overlay.increase_brightness(this.color, 80);
        //ctx.strokeStyle= 'green';
        ctx.beginPath();
        for (var k=0, len = this.overlays.length; k<len; k++) {
            if (! this.overlays[k].isSelected) {
                continue;
            }
            this.drawFootprintSelected(ctx, xyviews[k]);
            
        }
    	ctx.stroke();
    };

    Overlay.increase_brightness = function(hex, percent){
        // strip the leading # if it's there
        hex = hex.replace(/^\s*#|\s*$/g, '');

        // convert 3 char codes --> 6, e.g. `E0F` --> `EE00FF`
        if(hex.length == 3){
            hex = hex.replace(/(.)/g, '$1$1');
        }

        var r = parseInt(hex.substr(0, 2), 16),
            g = parseInt(hex.substr(2, 2), 16),
            b = parseInt(hex.substr(4, 2), 16);

        return '#' +
                ((0|(1<<8) + r + (256 - r) * percent / 100).toString(16)).substr(1) +
                ((0|(1<<8) + g + (256 - g) * percent / 100).toString(16)).substr(1) +
                ((0|(1<<8) + b + (256 - b) * percent / 100).toString(16)).substr(1);
    }
    
    
    
    Overlay.prototype.drawFootprint = function(f, ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! f.isShowing) {
            return null;
        }
        var xyviewArray = [];
        var show = false;
        var radecArray = f.polygons;
        // for
            for (var k=0, len=radecArray.length; k<len; k++) {
                var xy;
                if (frame!=CooFrameEnum.J2000) {
                    var lonlat = CooConversion.J2000ToGalactic([radecArray[k][0], radecArray[k][1]]);
                    xy = projection.project(lonlat[0], lonlat[1]);
                }
                else {
                    xy = projection.project(radecArray[k][0], radecArray[k][1]);
                }
                if (!xy) {
                    return null;
                }
                var xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
                xyviewArray.push(xyview);
                if (!show && xyview.vx<width  && xyview.vx>=0 && xyview.vy<=height && xyview.vy>=0) {
                    show = true;
                }
            }

            if (show) {
                ctx.moveTo(xyviewArray[0].vx, xyviewArray[0].vy);
                for (var k=1, len=xyviewArray.length; k<len; k++) {
                    ctx.lineTo(xyviewArray[k].vx, xyviewArray[k].vy);
                }
            }
            else {
                //return null;
            }
        // end for

        return xyviewArray;



    };

    Overlay.prototype.drawFootprintSelected = function(ctx, xyview) {
        if (!xyview) {
            return;
        }

        var xyviewArray = xyview;
        ctx.moveTo(xyviewArray[0].vx, xyviewArray[0].vy);
        for (var k=1, len=xyviewArray.length; k<len; k++) {
            ctx.lineTo(xyviewArray[k].vx, xyviewArray[k].vy);
        }
    };


    
    // callback function to be called when the status of one of the footprints has changed
    Overlay.prototype.reportChange = function() {
        this.view.requestRedraw();
    };

    return Overlay;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Source
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

cds.Source = (function() {
    // constructor
    cds.Source = function(ra, dec, data, options) {
    	this.ra = ra;
    	this.dec = dec;
    	this.data = data;
    	this.catalog = null;
    	
        this.marker = (options && options.marker) || false;
        if (this.marker) {
            this.popupTitle = (options && options.popupTitle) ? options.popupTitle : '';
            this.popupDesc = (options && options.popupDesc) ? options.popupDesc : '';
        }
    	this.isShowing = true;
    	this.isSelected = false;
    };
    
    cds.Source.prototype.setCatalog = function(catalog) {
        this.catalog = catalog;
    };
    
    cds.Source.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    cds.Source.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    return cds.Source;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File ProgressiveCat.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// TODO: indexer sources par numéro HEALPix
// TODO : harmoniser parsing avec classe Catalog
ProgressiveCat = (function() {
    
    // TODO : test if CORS support. If no, need to pass through a proxy
    // currently, we suppose CORS is supported
    
    // constructor
    ProgressiveCat = function(rootUrl, frameStr, maxOrder, options) {
        options = options || {};

        this.type = 'progressivecat';
        
        this.rootUrl = rootUrl;
        this.frame = CooFrameEnum.fromString(frameStr) || CooFrameEnum.J2000;
        this.maxOrder = maxOrder;
        this.isShowing = true; // TODO : inherit from catalogue

        this.name = options.name || "progressive-cat";
        this.color = options.color || Color.getNextColor();
        this.sourceSize = options.sourceSize || 10;
        

        // we cache the list of sources in each healpix tile. Key of the cache is norder+'-'+npix
        this.sourcesCache = new Utils.LRUCache(100);

        this.cacheCanvas = document.createElement('canvas');
        this.cacheCanvas.width = this.sourceSize;
        this.cacheCanvas.height = this.sourceSize;
        var cacheCtx = this.cacheCanvas.getContext('2d');
        cacheCtx.beginPath();
        cacheCtx.strokeStyle = this.color;
        cacheCtx.lineWidth = 2.0;
        cacheCtx.moveTo(0, 0);
        cacheCtx.lineTo(0,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize, 0);
        cacheCtx.lineTo(0, 0);
        cacheCtx.stroke();
    };

    function getFields(instance, xml) {
        var attributes = ["name", "ID", "ucd", "utype", "unit", "datatype", "arraysize", "width", "precision"];

        var fields = [];
        var k = 0;
        instance.keyRa = instance.keyDec = null;
        $(xml).find("FIELD").each(function() {
            var f = {};
            for (var i=0; i<attributes.length; i++) {
                var attribute = attributes[i];
                if ($(this).attr(attribute)) {
                    f[attribute] = $(this).attr(attribute);
                }
                
            }
            if ( ! f.ID) {
                f.ID = "col_" + k;
            }
            
            if (!instance.keyRa && f.ucd && (f.ucd.indexOf('pos.eq.ra')==0 || f.ucd.indexOf('POS_EQ_RA')==0)) {
                if (f.name) {
                    instance.keyRa = f.name;
                }
                else {
                    instance.keyRa = f.ID;
                }
            }
            if (!instance.keyDec && f.ucd && (f.ucd.indexOf('pos.eq.dec')==0 || f.ucd.indexOf('POS_EQ_DEC')==0)) {
                if (f.name) {
                    instance.keyDec = f.name;
                }
                else {
                    instance.keyDec = f.ID;
                }
            }
            
            fields.push(f);
            k++;
        });

        return fields;
    }

    function getSources(instance, csv, fields) {
        // TODO : find ra and dec key names (see in Catalog)
        if (!instance.keyRa || ! instance.keyDec) {
            return [];
        }
        lines = csv.split('\n');
        var mesureKeys = [];
        for (var k=0; k<fields.length; k++) {
            if (fields[k].name) {
                mesureKeys.push(fields[k].name);
            }
            else {
                mesureKeys.push(fields[k].ID);
            }
        }
        

        var sources = [];
        var coo = new Coo();
        // start at i=1, as first line repeat the fields names
        for (var i=2; i<lines.length; i++) {
            var mesures = {};
            var data = lines[i].split('\t');
            if (data.length<mesureKeys.length) {
                continue;
            }
            for (var j=0; j<mesureKeys.length; j++) {
                mesures[mesureKeys[j]] = data[j];
            }
            var ra, dec;
            if (Utils.isNumber(mesures[instance.keyRa]) && Utils.isNumber(mesures[instance.keyDec])) {
                ra = parseFloat(mesures[instance.keyRa]);
                dec = parseFloat(mesures[instance.keyDec]);
            }
            else {
                coo.parse(mesures[instance.keyRa] + " " + mesures[instance.keyDec]);
                ra = coo.lon;
                dec = coo.lat;
            }
            sources.push(new cds.Source(ra, dec, mesures));
        }
        return sources;
    }

    ProgressiveCat.prototype = {

        init: function(view) {
            this.view = view;
            if (this.level3Sources) {
                return; // if already loaded, do nothing
            }
            this.loadLevel2Sources();
        },

        loadLevel2Sources: function() {
            var self = this;
            $.ajax({
                /*
                url: Aladin.JSONP_PROXY,
                data: {"url": self.rootUrl + '/' + 'Norder2/Allsky.xml'},
                datatype: 'jsonp',
                */
                url: self.rootUrl + '/' + 'Norder2/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.fields = getFields(self, xml);
                    self.level2Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    self.loadLevel3Sources();
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        loadLevel3Sources: function() {
            var self = this;
            $.ajax({
                /*
                url: Aladin.JSONP_PROXY,
                data: {"url": self.rootUrl + '/' + 'Norder3/Allsky.xml'},
                datatype: 'jsonp',
                */
                url: self.rootUrl + '/' + 'Norder3/Allsky.xml',
                method: 'GET',
                success: function(xml) {
                    self.level3Sources = getSources(self, $(xml).find('CSV').text(), self.fields);
                    //console.log(self.level3Sources);
                    self.view.requestRedraw();
                },
                error: function(err) {
                    console.log('Something went wrong: ' + err);
                }
            });
        },

        draw: function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
            if (! this.isShowing || ! this.level3Sources) {
                return;
            }
            //var sources = this.getSources();
            this.drawSources(this.level2Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            this.drawSources(this.level3Sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
            
            if (!this.tilesInView) {
                return;
            }
            var sources, key, t;
            for (var k=0; k<this.tilesInView.length; k++) {
                t = this.tilesInView[k];
                key = t[0] + '-' + t[1];
                sources = this.sourcesCache.get(key);
                if (sources) {
                    this.drawSources(sources, ctx, projection, frame, width, height, largestDim, zoomFactor);
                }
            }
            
            
            
        },
        drawSources: function(sources, ctx, projection, frame, width, height, largestDim, zoomFactor) {
            for (var k=0, len = sources.length; k<len; k++) {
                this.drawSource(sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor);
            }
        },
        getSources: function() {
            var ret = [];
            if (this.level2Sources) {
                ret = ret.concat(this.level2Sources);
            }
            if (this.level3Sources) {
                ret = ret.concat(this.level3Sources);
            }
            if (this.tilesInView) {
                var sources, key, t;
                for (var k=0; k<this.tilesInView.length; k++) {
                    t = this.tilesInView[k];
                    key = t[0] + '-' + t[1];
                    sources = this.sourcesCache.get(key);
                    if (sources) {
                        ret = ret.concat(sources);
                    }
                }
            }
            
            return ret;
        },

        // TODO : factoriser avec drawSource de Catalog
        drawSource: function(s, ctx, projection, frame, width, height, largestDim, zoomFactor) {
            if (! s.isShowing) {
                return;
            }
            var sourceSize = this.sourceSize;
            var xy;
            if (frame!=CooFrameEnum.J2000) {
                var lonlat = CooConversion.J2000ToGalactic([s.ra, s.dec]);
                xy = projection.project(lonlat[0], lonlat[1]);
            }
            else {
                xy = projection.project(s.ra, s.dec);
            }
            if (xy) {
                var xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
                if (xyview) {
                    // TODO : index sources
                    // check if source is visible in view ?
                    if (xyview.vx>(width+sourceSize)  || xyview.vx<(0-sourceSize) ||
                        xyview.vy>(height+sourceSize) || xyview.vy<(0-sourceSize)) {
                        s.x = s.y = undefined;
                        return;
                    }

                    s.x = xyview.vx;
                    s.y = xyview.vy;
                    ctx.drawImage(this.cacheCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
                }
            }
        },
        
        deselectAll: function() {
            for (var k=0; k<this.level2Sources.length; k++) {
                this.level2Sources[k].deselect();
            }
            for (var k=0; k<this.level3Sources.length; k++) {
                this.level3Sources[k].deselect();
            }
            var keys = this.sourcesCache.keys();
            for (key in keys) {
                if ( ! this.sourcesCache[key]) {
                    continue;
                }
                var sources = this.sourcesCache[key];
                for (var k=0; k<sources.length; k++) {
                    sources[k].deselect();
                }
            }
        },

        show: function() {
            if (this.isShowing) {
                return;
            }
            this.isShowing = true;
            this.reportChange();
        },
        hide: function() {
            if (! this.isShowing) {
                return;
            }
            this.isShowing = false;
            this.reportChange();
        },
        reportChange: function() {
            this.view.requestRedraw();
        },
        
        getTileURL: function(norder, npix) {
            var dirIdx = Math.floor(npix/10000)*10000;
            return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + ".tsv";
        },
    
        loadNeededTiles: function() {
            this.tilesInView = [];
            
            this.otherSources = [];
            var norder = this.view.realNorder;
            if (norder>this.maxOrder) {
                norder = this.maxOrder;
            }
            if (norder<=3) {
                return; // nothing to do, hurrayh !
            }
            var cells = this.view.getVisibleCells(norder, this.frame);
            var ipixList, ipix;
            for (var curOrder=4; curOrder<=norder; curOrder++) {
                ipixList = [];
                for (var k=0; k<cells.length; k++) {
                    ipix = Math.floor(cells[k].ipix / Math.pow(4, norder - curOrder));
                    if (ipixList.indexOf(ipix)<0) {
                        ipixList.push(ipix);
                    }
                }
                
                // load needed tiles
                for (var i=0; i<ipixList.length; i++) {
                    this.tilesInView.push([curOrder, ipixList[i]]);
                }
            }
            
            var t, key;
            var self = this;
            for (var k=0; k<this.tilesInView.length; k++) {
                t = this.tilesInView[k];
                key = t[0] + '-' + t[1]; // t[0] is norder, t[1] is ipix
                if (!this.sourcesCache.get(key)) {
                    (function(self, norder, ipix) { // wrapping function is needed to be able to retrieve norder and ipix in ajax success function
                        var key = norder + '-' + ipix;
                        $.ajax({
                            /*
                            url: Aladin.JSONP_PROXY,
                            data: {"url": self.getTileURL(norder, ipix)},
                            */
                            // ATTENTIOn : je passe en JSON direct, car je n'arrive pas à choper les 404 en JSONP
                            url: self.getTileURL(norder, ipix),
                            method: 'GET',
                            //dataType: 'jsonp',
                            success: function(tsv) {
                                self.sourcesCache.set(key, getSources(self, tsv, self.fields));
                                //self.otherSources = self.otherSources.concat(getSources(tsv, self.fields));
                                self.view.requestRedraw();
                            },
                            error: function() {
                                // on suppose qu'il s'agit d'une erreur 404
                                self.sourcesCache.set(key, []);
                            }
                        });
                    })(this, t[0], t[1]);
                }
            }
        }



    }; // END OF .prototype functions
    
    
    return ProgressiveCat;
})();
    
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// TODO : harmoniser parsing avec classe ProgressiveCat
cds.Catalog = (function() {
   cds.Catalog = function(options) {
        options = options || {};
        this.type = 'catalog';
    	this.name = options.name || "catalog";
    	this.color = options.color || Color.getNextColor();
    	this.sourceSize = options.sourceSize || 6;
        
        this.selectSize = this.sourceSize + 2;
        
        this.isShowing = true;

    	
    	this.indexationNorder = 5; // à quel niveau indexe-t-on les sources
    	this.sources = [];
    	this.hpxIdx = new HealpixIndex(this.indexationNorder);
    	this.hpxIdx.init();
    	this.selectionColor = '#00ff00';
    	
    	
    	// cacheCanvas permet de ne créer le path de la source qu'une fois, et de le réutiliser (cf. http://simonsarris.com/blog/427-increasing-performance-by-caching-paths-on-canvas)
        this.cacheCanvas = document.createElement('canvas');
        this.cacheCanvas.width = this.sourceSize;
        this.cacheCanvas.height = this.sourceSize;
        var cacheCtx = this.cacheCanvas.getContext('2d');
        cacheCtx.beginPath();
        cacheCtx.strokeStyle = this.color;
        cacheCtx.lineWidth = 2.0;
        cacheCtx.moveTo(0, 0);
        cacheCtx.lineTo(0,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize,  this.sourceSize);
        cacheCtx.lineTo( this.sourceSize, 0);
        cacheCtx.lineTo(0, 0);
        cacheCtx.stroke();

        this.cacheMarkerCanvas = document.createElement('canvas');
        this.cacheMarkerCanvas.width = this.sourceSize;
        this.cacheMarkerCanvas.height = this.sourceSize;
        var cacheMarkerCtx = this.cacheMarkerCanvas.getContext('2d');
        cacheMarkerCtx.fillStyle = this.color;
        cacheMarkerCtx.beginPath();
        var half = (this.sourceSize)/2.;
        cacheMarkerCtx.arc(half, half, half-2, 0, 2 * Math.PI, false);
        cacheMarkerCtx.fill();
        cacheMarkerCtx.lineWidth = 2;
        cacheMarkerCtx.strokeStyle = '#ccc';
        cacheMarkerCtx.stroke();
        //cacheMarkerCtx.fillRect(0, 0, this.sourceSize, this.sourceSize);
        
        this.cacheSelectCanvas = document.createElement('canvas');
        this.cacheSelectCanvas.width = this.selectSize;
        this.cacheSelectCanvas.height = this.selectSize;
        var cacheSelectCtx = this.cacheSelectCanvas.getContext('2d');
        cacheSelectCtx.beginPath();
        cacheSelectCtx.strokeStyle = this.selectionColor;
        cacheSelectCtx.lineWidth = 2.0;
        cacheSelectCtx.moveTo(0, 0);
        cacheSelectCtx.lineTo(0,  this.selectSize);
        cacheSelectCtx.lineTo( this.selectSize,  this.selectSize);
        cacheSelectCtx.lineTo( this.selectSize, 0);
        cacheSelectCtx.lineTo(0, 0);
        cacheSelectCtx.stroke();

    };
    

    
    
    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed
    cds.Catalog.parseVOTable = function(url, callback) {
        
        function doParseVOTable(xml, callback) {
            xml = xml.replace(/^\s+/g, ''); // we need to trim whitespaces at start of document
            var attributes = ["name", "ID", "ucd", "utype", "unit", "datatype", "arraysize", "width", "precision"];
            
            var fields = [];
            var k = 0;
            $(xml).find("FIELD").each(function() {
                var f = {};
                for (var i=0; i<attributes.length; i++) {
                    var attribute = attributes[i];
                    if ($(this).attr(attribute)) {
                        f[attribute] = $(this).attr(attribute);
                    }
                }
                if ( ! f.ID) {
                    f.ID = "col_" + k;
                }
                fields.push(f);
                k++;
            });
                
            // find RA/DEC fields
            var raFieldIdx,  decFieldIdx;
            raFieldIdx = decFieldIdx = null;
            for (var l=0, len=fields.length; l<len; l++) {
                var field = fields[l];
                if ( ! raFieldIdx) {
                    if (field.ucd) {
                        var ucd = field.ucd.toLowerCase();
                        if (ucd.indexOf('pos.eq.ra')>=0 || ucd.indexOf('pos_eq_ra')>=0) {
                            raFieldIdx = l;
                            continue;
                        }
                    }
                }
                    
                if ( ! decFieldIdx) {
                    if (field.ucd) {
                        var ucd = field.ucd.toLowerCase();
                        if (ucd.indexOf('pos.eq.dec')>=0 || ucd.indexOf('pos_eq_dec')>=0) {
                            decFieldIdx = l;
                            continue;
                        }
                    }
                }
            }
            var sources = [];
            
            var coo = new Coo();
            var ra, dec;
            $(xml).find("TR").each(function() {
               var mesures = {};
               var k = 0;
               $(this).find("TD").each(function() {
                   var key = fields[k].name ? fields[k].name : fields[k].id;
                   mesures[key] = $(this).text();
                   k++;
               });
               var keyRa = fields[raFieldIdx].name ? fields[raFieldIdx].name : fields[raFieldIdx].id;
               var keyDec = fields[decFieldIdx].name ? fields[decFieldIdx].name : fields[decFieldIdx].id;

               if (Utils.isNumber(mesures[keyRa]) && Utils.isNumber(mesures[keyDec])) {
                   ra = parseFloat(mesures[keyRa]);
                   dec = parseFloat(mesures[keyDec]);
               }
               else {
                   coo.parse(mesures[keyRa] + " " + mesures[keyDec]);
                   ra = coo.lon;
                   dec = coo.lat;
               }
               sources.push(new cds.Source(ra, dec, mesures));
            });
            if (callback) {
                callback(sources);
            }
        }
        
        $.ajax({
            url: Aladin.JSONP_PROXY,
            data: {"url": url},
            method: 'GET',
            dataType: 'jsonp',
            success: function(xml) {
                doParseVOTable(xml, callback);
            }/*,
            }
            error: callbackFunctionError*/
        });
    };
    
    cds.Catalog.prototype.addSources = function(sourcesToAdd) {
    	this.sources = this.sources.concat(sourcesToAdd);
    	for (var k=0, len=sourcesToAdd.length; k<len; k++) {
    	    sourcesToAdd[k].setCatalog(this);
    	}
        this.view.requestRedraw();
    };
    
    cds.Catalog.prototype.getSources = function() {
        return this.sources;
    };
    
    // TODO : fonction générique traversant la liste des sources
    cds.Catalog.prototype.selectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].select();
        }
    };
    
    cds.Catalog.prototype.deselectAll = function() {
        if (! this.sources) {
            return;
        }
        
        for (var k=0; k<this.sources.length; k++) {
            this.sources[k].deselect();
        }
    };
    
    // return a source by index
    cds.Catalog.prototype.getSource = function(idx) {
        if (idx<this.sources.length) {
            return this.sources[idx];
        }
        else {
            return null;
        }
    };
    
    cds.Catalog.prototype.setView = function(view) {
        this.view = view;
    };
    
    cds.Catalog.prototype.removeAll = function() {
        // TODO : RAZ de l'index
        this.sources = [];
    };
    
    cds.Catalog.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! this.isShowing) {
            return;
        }
        // tracé simple
        //ctx.strokeStyle= this.color;

        //ctx.lineWidth = 1;
    	//ctx.beginPath();
    	for (var k=0, len = this.sources.length; k<len; k++) {
    		this.drawSource(this.sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor);
    	}
        //ctx.stroke();

    	// tracé sélection
        ctx.strokeStyle= this.selectionColor;
        ctx.beginPath();
        for (var k=0, len = this.sources.length; k<len; k++) {
            if (! this.sources[k].isSelected) {
                continue;
            }
            this.drawSourceSelection(this.sources[k], ctx);
            
        }
    	ctx.stroke();
    };
    
    
    
    cds.Catalog.prototype.drawSource = function(s, ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! s.isShowing) {
            return;
        }
        var sourceSize = this.sourceSize;
        var xy;
        if (frame!=CooFrameEnum.J2000) {
            var lonlat = CooConversion.J2000ToGalactic([s.ra, s.dec]);
            xy = projection.project(lonlat[0], lonlat[1]);
        }
        else {
            xy = projection.project(s.ra, s.dec);
        }
        if (xy) {
            var xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
            var max = s.popup ? 100 : s.sourceSize;
            if (xyview) {
                // TODO : index sources
                // check if source is visible in view ?
                if (xyview.vx>(width+max)  || xyview.vx<(0-max) ||
                    xyview.vy>(height+max) || xyview.vy<(0-max)) {
                    s.x = s.y = undefined;
                    return;
                }
                
                s.x = xyview.vx;
                s.y = xyview.vy;

                if (s.marker) {
                    ctx.drawImage(this.cacheMarkerCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
                }
                else {
                    ctx.drawImage(this.cacheCanvas, s.x-sourceSize/2, s.y-sourceSize/2);
                }


                // has associated popup ?
                if (s.popup) {
                    s.popup.setPosition(s.x, s.y);
                }
                
//                ctx.moveTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
//                ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy-sourceSize/2);
//                ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
//                ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy+sourceSize/2);
//                ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
                
            }
        }
    };
    
    cds.Catalog.prototype.drawSourceSelection = function(s, ctx) {
        if (!s || !s.isShowing || !s.x || !s.y) {
            return;
        }
        var sourceSize = this.selectSize;
        
        ctx.drawImage(this.cacheSelectCanvas, s.x-sourceSize/2, s.y-sourceSize/2);

//        ctx.moveTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
//        ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy+sourceSize/2);
//        ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
//        ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy-sourceSize/2);
//        ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
    };

    
    // callback function to be called when the status of one of the sources has changed
    cds.Catalog.prototype.reportChange = function() {
        this.view.requestRedraw();
    };
    
    cds.Catalog.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };
    
    cds.Catalog.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.view && this.view.popup && this.view.popup.source && this.view.popup.source.catalog==this) {
            this.view.popup.hide();
        }
        this.reportChange();
    };

    return cds.Catalog;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Tile
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Tile = (function() {
    // constructor
	function Tile(img, url) {
		this.img = img;
		this.url = url;
	};
	
	// check whether the image corresponding to the tile is loaded and ready to be displayed
	//
	// source : http://www.sajithmr.me/javascript-check-an-image-is-loaded-or-not
	Tile.isImageOk = function(img) {
		if (img.allSkyTexture) {
			return true;
		}
		
        if (!img.src) {
            return false;
        }

	    // During the onload event, IE correctly identifies any images that
	    // weren’t downloaded as not complete. Others should too. Gecko-based
	    // browsers act like NS4 in that they report this incorrectly.
	    if (!img.complete) {
	        return false;
	    }

	    // However, they do have two very useful properties: naturalWidth and
	    // naturalHeight. These give the true size of the image. If it failed
	    // to load, either of these should be zero.

	    if (typeof img.naturalWidth != "undefined" && img.naturalWidth == 0) {
	        return false;
	    }

	    // No other way of checking: assume it’s ok.
	    return true;
	};
	

	return Tile;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File TileBuffer
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

TileBuffer = (function() {
	var NB_MAX_TILES = 800; // buffer size
	
	// constructor
	function TileBuffer() {
		this.pointer = 0;
		this.tilesMap = {};
		this.tilesArray = new Array(NB_MAX_TILES);

		for (var i=0; i<NB_MAX_TILES; i++) {
			this.tilesArray[i] = new Tile(new Image(), null);
		}
	};
	
	TileBuffer.prototype.addTile = function(url) {
	    // return null if already in buffer
        if (this.getTile(url)) {
            return null;
        }

        // delete existing tile
        var curTile = this.tilesArray[this.pointer];
        if (curTile.url != null) {
            curTile.img.src = null;
            delete this.tilesMap[curTile.url];
        }

        this.tilesArray[this.pointer].url = url;
        this.tilesMap[url] = this.tilesArray[this.pointer];

        this.pointer++;
        if (this.pointer>=NB_MAX_TILES) {
            this.pointer = 0;
        }

        return this.tilesMap[url];
	};
	
	TileBuffer.prototype.getTile = function(url) {
        return this.tilesMap[url];
	};
	
	return TileBuffer;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File ColorMap.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

ColorMap = (function() {
    
    
    // constructor
    ColorMap = function(view) {
        this.view = view;
        this.reversed = false;
        this.map = 'native';
        this.sig = this.signature();
    };
    
ColorMap.MAPS = {};
    
    ColorMap.MAPS['eosb'] = {
            name: 'Eos B',
            r: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,9,18,27,36,45,49,57,72,81,91,100,109,118,127,
                136,131,139,163,173,182,191,200,209,218,227,213,221,255,255,255,255,255,
                255,255,255,229,229,255,255,255,255,255,255,255,255,229,229,255,255,255,
                255,255,255,255,255,229,229,255,255,255,255,255,255,255,255,229,229,255,
                255,255,255,255,255,255,255,229,229,255,255,255,255,255,255,255,255,229,
                229,255,255,255,255,255,255,255,255,229,229,255,255,255,255,255,255,255,
                255,229,229,255,255,255,255,255,255,255,255,229,229,255,253,251,249,247,
                245,243,241,215,214,235,234,232,230,228,226,224,222,198,196,216,215,213,
                211,209,207,205,203,181,179,197,196,194,192,190,188,186,184,164,162,178,
                176,175,173,171,169,167,165,147,145,159,157,156,154,152,150,148,146,130,
                128,140,138,137,135,133,131,129,127,113,111,121,119,117,117],
            g: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,15,23,31,39,47,55,57,64,79,87,95,
                103,111,119,127,135,129,136,159,167,175,183,191,199,207,215,200,207,239,
                247,255,255,255,255,255,255,229,229,255,255,255,255,255,255,255,255,229,
                229,255,255,255,255,255,255,255,255,229,229,255,250,246,242,238,233,229,
                225,198,195,212,208,204,199,195,191,187,182,160,156,169,165,161,157,153,
                148,144,140,122,118,127,125,123,121,119,116,114,112,99,97,106,104,102,
                99,97,95,93,91,80,78,84,82,80,78,76,74,72,70,61,59,63,61,59,57,55,53,50,
                48,42,40,42,40,38,36,33,31,29,27,22,21,21,19,16,14,12,13,8,6,3,1,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            b: [116,121,127,131,136,140,144,148,153,
                157,145,149,170,174,178,182,187,191,195,199,183,187,212,216,221,225,229,
                233,238,242,221,225,255,247,239,231,223,215,207,199,172,164,175,167,159,
                151,143,135,127,119,100,93,95,87,79,71,63,55,47,39,28,21,15,7,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0]
    };
    ColorMap.MAPS['rainbow'] = {
            name: 'Rainbow',
            r: [0,4,9,13,18,22,27,31,36,40,45,50,54,
                58,61,64,68,69,72,74,77,79,80,82,83,85,84,86,87,88,86,87,87,87,85,84,84,
                84,83,79,78,77,76,71,70,68,66,60,58,55,53,46,43,40,36,33,25,21,16,12,4,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,8,12,21,25,29,33,42,
                46,51,55,63,67,72,76,80,89,93,97,101,110,114,119,123,131,135,140,144,153,
                157,161,165,169,178,182,187,191,199,203,208,212,221,225,229,233,242,246,
                250,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255],
            g: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,4,8,16,21,25,29,38,42,46,51,55,63,67,72,76,84,89,93,97,
                106,110,114,119,127,131,135,140,144,152,157,161,165,174,178,182,187,195,
                199,203,208,216,220,225,229,233,242,246,250,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,250,242,238,233,229,221,216,212,208,199,195,191,187,178,174,170,165,
                161,153,148,144,140,131,127,123,119,110,106,102,97,89,85,80,76,72,63,59,
                55,51,42,38,34,29,21,17,12,8,0],
            b: [0,3,7,10,14,19,23,28,32,38,43,48,53,
                59,63,68,72,77,81,86,91,95,100,104,109,113,118,122,127,132,136,141,145,
                150,154,159,163,168,173,177,182,186,191,195,200,204,209,214,218,223,227,
                232,236,241,245,250,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,
                255,255,255,255,255,255,246,242,238,233,225,220,216,212,203,199,195,191,
                187,178,174,170,165,157,152,148,144,135,131,127,123,114,110,106,102,97,
                89,84,80,76,67,63,59,55,46,42,38,34,25,21,16,12,8,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    };

    
    ColorMap.MAPS_CUSTOM = ['rainbow', 'eosb'];
    ColorMap.MAPS_NAMES = ['native', 'grayscale'].concat(ColorMap.MAPS_CUSTOM);
    
    ColorMap.prototype.reverse = function(val) {
        if (val) {
            this.reversed = val;
        }
        else {
            this.reversed = ! this.reversed;
        }
        this.sig = this.signature();
        this.view.requestRedraw();
    };
    
    
    ColorMap.prototype.signature = function() {
        var s = this.map;
        
        if (this.reversed) {
            s += ' reversed';
        }
        
        return s;
    };
    
    ColorMap.prototype.update = function(map) {
        this.map = map;
        this.sig = this.signature();
        this.view.requestRedraw();
    };
    
    ColorMap.prototype.apply = function(img) {
        if ( this.sig=='native' ) {
            return img;
        }
        
        if (img.cmSig==this.sig) {
            return img.cmImg; // return cached pixels
        }
        
        var canvas = document.createElement("canvas");
        canvas.width = img.width;
        canvas.height = img.height;
        var ctx = canvas.getContext("2d");
        ctx.drawImage(img, 0, 0);
        
        var imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        var pixelData = imageData.data;
        var length = pixelData.length;
        var a, b, c;
        var switchCase = 3;
        if (this.map=='grayscale') {
            switchCase = 1;
        }
        else if (ColorMap.MAPS_CUSTOM.indexOf(this.map)>=0) {
            switchCase = 2;
        }
        for (var i = 0; i < length; i+= 4) {
            switch(switchCase) {
                case 1:
                    a = b = c = AladinUtils.myRound((pixelData[i]+pixelData[i+1]+pixelData[i+2])/3);
                    break;
                case 2:
                    if (this.reversed) {
                        a = ColorMap.MAPS[this.map].r[255-pixelData[i]];
                        b = ColorMap.MAPS[this.map].g[255-pixelData[i+1]];
                        c = ColorMap.MAPS[this.map].b[255-pixelData[i+2]];
                    }
                    else {
                        a = ColorMap.MAPS[this.map].r[pixelData[i]];
                        b = ColorMap.MAPS[this.map].g[pixelData[i+1]];
                        c = ColorMap.MAPS[this.map].b[pixelData[i+2]];
                    }
                    break;
                default:
                    a = pixelData[i];
                    b = pixelData[i + 1];
                    c = pixelData[i + 2];
                    
            }
            if (switchCase!=2 && this.reversed) {
                a = 255-a;
                b = 255-b;
                c = 255-c;
              
            }
            pixelData[i]     = a;
            pixelData[i + 1] = b;
            pixelData[i + 2] = c;
            
        }
        imageData.data = pixelData;
        ctx.putImageData(imageData, 0, 0);
        
        // cache image with color map applied
        img.cmSig = this.sig;
        img.cmImg = canvas;

        return img.cmImg;
    };
    
    return ColorMap;
})();
    
/******************************************************************************
 * Aladin Lite project
 * 
 * File HpxImageSurvey
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HpxImageSurvey = (function() {


    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    var HpxImageSurvey = function(id, name, rootUrl, cooFrame, maxOrder, options) {
        this.id = id;
    	this.name = name;
    	if (rootUrl.slice(-1 )=== '/') {
    	    this.rootUrl = rootUrl.substr(0, rootUrl.length-1);
    	}
    	else {
    	    this.rootUrl = rootUrl;
    	}
    	
    	options = options || {};
    	// TODO : support PNG
    	this.imgFormat = options.imgFormat || 'jpg';

        // permet de forcer l'affichage d'un certain niveau
        this.minOrder = options.minOrder || null;

        // TODO : lire depuis fichier properties
        this.cooFrame = CooFrameEnum.fromString(cooFrame, CooFrameEnum.J2000);
        
        // force coo frame for Glimpse 360
        if (this.rootUrl.indexOf('/glimpse360/aladin/data')>=0) {
            this.cooFrame = CooFrameEnum.J2000;
        }
        
        // TODO : lire depuis fichier properties
        this.maxOrder = maxOrder;
    	
    	this.allskyTextures = [];
    	
    	this.alpha = 0.0; // opacity value between 0 and 1 (if this layer is an opacity layer)
    
    	this.allskyTextureSize = 0;
    
        this.lastUpdateDateNeededTiles = 0;

        var found = false;
        for (var k=0; k<HpxImageSurvey.SURVEYS.length; k++) {
            if (HpxImageSurvey.SURVEYS[k].id==this.id) {
                found = true;
            }
        }
        if (! found) {
            HpxImageSurvey.SURVEYS.push({
                 "id": this.id,
                 "url": this.rootUrl,
                 "name": this.name,
                 "maxOrder": this.maxOrder,
                 "frame": this.cooFrame
            });
        }
        HpxImageSurvey.SURVEYS_OBJECTS[this.id] = this;
    };
    
    HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY = 1000; // in milliseconds
    
    HpxImageSurvey.prototype.init = function(view, callback) {
    	this.view = view;
    	
        if (!this.cm) {
            this.cm = new ColorMap(this.view);
        }
    	
    	//this.tileBuffer = new TileBuffer();
    	// tileBuffer is now shared across different image surveys
    	this.tileBuffer = this.view.tileBuffer;
    	
    	this.useCors = false;
    	var self = this;
        if ($.support.cors) {
            // testing if server supports CORS ( http://www.html5rocks.com/en/tutorials/cors/ )
            $.ajax({
                type: 'GET',
                url: this.rootUrl + '/properties',
                contentType: 'text/plain',
                xhrFields: {
                },
                headers: {
                },
                success: function() {
                    // CORS is supported
                    self.useCors = true;
                    
                    self.retrieveAllskyTextures();
                    if (callback) {
                        callback();
                    }
                },
                error: function() {
                    // CORS is not supported
                    self.retrieveAllskyTextures();
                    if (callback) {
                        callback();
                    }
                }
              });
        }
        else {
            this.retrieveAllskyTextures();
            callback();
        }
    	
    };
    
    HpxImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";
    
    HpxImageSurvey.SURVEYS_OBJECTS = {};
    HpxImageSurvey.SURVEYS = [
     {
        "id": "P/2MASS/color",
        "url": "http://alasky.u-strasbg.fr/2MASS/Color",
        "name": "2MASS colored",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/DSS2/color",
        "url": "http://alasky.u-strasbg.fr/DssColor",
        "name": "DSS colored",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/DSS2/red",
        "url": "http://alasky.u-strasbg.fr/DSS/DSS2Merged",
        "name": "DSS2 Red (F+R)",
        "maxOrder": 9,
        "frame": "equatorial",
        "format": "jpeg fits"
     },
     {
        "id": "P/Fermi/color",
        "url": "http://alasky.u-strasbg.fr/Fermi/Color",
        "name": "Fermi color",
        "maxOrder": 3,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/Finkbeiner",
        "url": "http://alasky.u-strasbg.fr/FinkbeinerHalpha",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "jpeg fits",
        "name": "Halpha"
     },
     {
        "id": "P/GALEXGR6/AIS/color",
        "url": "http://alasky.u-strasbg.fr/GALEX/GR6-02-Color",
        "name": "GALEX Allsky Imaging Survey colored",
        "maxOrder": 8,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/IRIS/color",
        "url": "http://alasky.u-strasbg.fr/IRISColor",
        "name": "IRIS colored",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/Mellinger/color",
        "url": "http://alasky.u-strasbg.fr/MellingerRGB",
        "name": "Mellinger colored",
        "maxOrder": 4,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/SDSS9/color",
        "url": "http://alasky.u-strasbg.fr/SDSS/DR9/color",
        "name": "SDSS9 colored",
        "maxOrder": 10,
        "frame": "equatorial",
        "format": "jpeg"
     },
     {
        "id": "P/SPITZER/color",
        "url": "http://alasky.u-strasbg.fr/SpitzerI1I2I4color",
        "name": "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
        "maxOrder": 9,
        "frame": "galactic",
        "format": "jpeg"
     },
     {
        "id": "P/VTSS/Ha",
        "url": "http://alasky.u-strasbg.fr/VTSS/Ha",
        "maxOrder": 3,
        "frame": "galactic",
        "format": "png jpeg fits",
        "name": "VTSS-Ha"
     },
     {
        "id": "P/XMM/EPIC",
        "url": "http://saada.u-strasbg.fr/xmmallsky",
        "name": "XMM-Newton stacked EPIC images (no phot. normalization)",
        "maxOrder": 7,
        "frame": "equatorial",
        "format": "png jpeg fits"
     },
     {
         "id": "P/XMM/PN/color",
          "url": "http://saada.unistra.fr/xmmpnsky",
          "name": "XMM PN colored",
          "maxOrder": 7,
          "frame": "equatorial",
          "format": "png jpeg"
     }
  ];


    
    HpxImageSurvey.getAvailableSurveys = function() {
    	return HpxImageSurvey.SURVEYS;
    };
    
    HpxImageSurvey.getSurveyInfoFromId = function(id) {
        var surveys = HpxImageSurvey.getAvailableSurveys();
        for (var i=0; i<surveys.length; i++) {
            if (surveys[i].id==id) {
                return surveys[i];
            }
        }
        return null;
    };

    HpxImageSurvey.getSurveyFromId = function(id) {
        if (HpxImageSurvey.SURVEYS_OBJECTS[id]) {
            return HpxImageSurvey.SURVEYS_OBJECTS[id];
        }
        var surveyInfo = HpxImageSurvey.getSurveyInfoFromId(id);
        if (surveyInfo) {
            return new HpxImageSurvey(surveyInfo.id, surveyInfo.name, surveyInfo.url, surveyInfo.frame, surveyInfo.maxOrder);
        }

        return null;
    }
   
/* 
    HpxImageSurvey.getSurveyFromId = function(id) {
    	var info = HpxImageSurvey.getSurveyInfoFromId(id);
    	if (info) {
    		return new HpxImageSurvey(info.id, info.name, info.url, info.frame, info.maxOrder);
    	}
    	
    	return null;
    };
*/
    
    HpxImageSurvey.prototype.getTileURL = function(norder, npix) {
    	var dirIdx = Math.floor(npix/10000)*10000;
    	return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + "." + this.imgFormat;
    };
    
    HpxImageSurvey.prototype.retrieveAllskyTextures = function() {
    	// start loading of allsky
    	var img = new Image();
    	if (this.useCors) {
            img.crossOrigin = 'anonymous';
        }
    	var self = this;
    	img.onload = function() {
    		// sur ipad, le fichier qu'on récupère est 2 fois plus petit. Il faut donc déterminer la taille de la texture dynamiquement
    	    self.allskyTextureSize = img.width/27;
    
    		// récupération des 768 textures (NSIDE=4)
    		for (var j=0; j<29; j++) {
    			for (var i=0; i<27; i++) {
    				var c = document.createElement('canvas');
    				c.width = c.height = self.allskyTextureSize;
    				c.allSkyTexture = true;
    				var context = c.getContext('2d');
    				context.drawImage(img, i*self.allskyTextureSize, j*self.allskyTextureSize, self.allskyTextureSize, self.allskyTextureSize, 0, 0, c.width, c.height);
    				self.allskyTextures.push(c);
    			}
    		}
    		self.view.requestRedraw();
    	};
    	img.src = this.rootUrl + '/Norder3/Allsky.' + this.imgFormat;
    
    };
    
    HpxImageSurvey.prototype.redrawAllsky = function(ctx, cornersXYViewMap, fov, norder) {
    	// for norder deeper than 6, we think it brings nothing to draw the all-sky
    	if (this.view.curNorder>6) {
    		return;
    	}
    	
    	if ( ! this.allskyTextures ) {
    		return;
    	}
    	

    	var cornersXYView;
        var coeff = 0;
        var center;
        var ipix;
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;
    		
            if ( ! this.allskyTextures[ipix] || !Tile.isImageOk(this.allskyTextures[ipix]) ) {
                continue;
            }
    		
    
    		// TODO : plutot agrandir le clip ?
    	    // grow cornersXYView
    	    if (fov>40) {
    			coeff = 0.02;
    	        center = {x: (cornersXYView[0].vx+cornersXYView[2].vx)/2, y: (cornersXYView[0].vy+cornersXYView[2].vy)/2};
    	        for (var i=0; i<4; i++) {
    	            var diff = {x: cornersXYView[i].vx-center.x, y: cornersXYView[i].vy-center.y};
    	            cornersXYView[i].vx += coeff*diff.x;
    	            cornersXYView[i].vy += coeff*diff.y;
    	        }
    	    }
    			
    	    this.drawOneTile(ctx, this.allskyTextures[ipix], cornersXYView, this.allskyTextureSize);
    	}
    };
    
    HpxImageSurvey.prototype.getColorMap = function() {
        return this.cm;
    };
    
    var drawEven = true;
    // TODO: avoir un mode où on ne cherche pas à dessiner d'abord les tuiles parentes (pour génération vignettes côté serveur)
    HpxImageSurvey.prototype.redrawHighres = function(ctx, cornersXYViewMap, norder) {
        drawEven = ! drawEven;
        var now = new Date().getTime();
        var updateNeededTiles = (now-this.lastUpdateDateNeededTiles) > HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY;
        var tile, url, parentTile, parentUrl;
        var parentNorder = norder - 1;
        var cornersXYView, parentCornersXYView;
        var tilesToDraw = [];
        var parentTilesToDraw = [];
        var parentTilesToDrawIpix = {};
        var missingTiles = false;
        
        var tilesToDownload = [];
        var parentTilesToDownload = [];
        
        var parentIpix;
        var ipix;
        
        // tri des tuiles selon la distance
        if (updateNeededTiles) {
            var center = [(cornersXYViewMap[0][0].vx+cornersXYViewMap[0][1].vx)/2, (cornersXYViewMap[0][0].vy+cornersXYViewMap[0][1].vy)/2];
            var newCornersXYViewMap = cornersXYViewMap.sort(function(a, b) {
                var cA = [(a[0].vx+a[2].vx)/2, (a[0].vy+a[2].vy)/2];
                var cB = [(b[0].vx+b[2].vx)/2, (b[0].vy+b[2].vy)/2]; 

                var distA = (cA[0]-center[0])*(cA[0]-center[0]) + (cA[1]-center[1])*(cA[1]-center[1]);
                var distB = (cB[0]-center[0])*(cB[0]-center[0]) + (cB[1]-center[1])*(cB[1]-center[1]);
                
                return distA-distB;
                    
            });
            cornersXYViewMap = newCornersXYViewMap;
        }
        
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;
    		/*
    		if (ipix%2==0 && ! drawEven) {
    		    continue;
    		}
    		else if (ipix%2==1 && drawEven) {
    		    continue;
    		}
    		*/
            
            // on demande à charger le parent (cas d'un zoomOut)
            // TODO : mettre priorité plus basse
            parentIpix = ~~(ipix/4);
        	parentUrl = this.getTileURL(parentNorder, parentIpix);
            if (updateNeededTiles && parentNorder>=3) {
            	parentTile = this.tileBuffer.addTile(parentUrl);
                if (parentTile) {
                    parentTilesToDownload.push({img: parentTile.img, url: parentUrl});
                }        }
            
            url = this.getTileURL(norder, ipix);
            tile = this.tileBuffer.getTile(url);
            
            if ( ! tile ) {
                missingTiles = true;
                
                if (updateNeededTiles) {
                    var tile = this.tileBuffer.addTile(url);
                    if (tile) {
                        tilesToDownload.push({img: tile.img, url: url});
                    }
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = this.view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
    
                continue;
            }
            else if ( ! Tile.isImageOk(tile.img)) {
                missingTiles = true;
                if (updateNeededTiles && ! tile.img.dlError) {
                    tilesToDownload.push({img: tile.img, url: url});
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = this.view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
                
                continue;
            }
            tilesToDraw.push({img: tile.img, corners: cornersXYView});
        }
    	
    
    
        // draw parent tiles
        for (var k=0, len = parentTilesToDraw.length; k<len; k++) {
        	this.drawOneTile(ctx, parentTilesToDraw[k].img, parentTilesToDraw[k].corners, parentTilesToDraw[k].img.width);
        }
        
        // draw tiles
        /*
        // using loop unrolling
        var iterations = Math.ceil(tilesToDraw.length / 8);
        var startAt = tilesToDraw.length % 8;
        var i = 0;
        var theTileToDraw;
        do {
            switch(startAt){
                case 0: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 7: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 6: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 5: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 4: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 3: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 2: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
                case 1: theTileToDraw = tilesToDraw[i++]; this.drawOneTile(ctx, theTileToDraw.img, theTileToDraw.corners, theTileToDraw.img.width, alpha);
            }
            startAt = 0;
        } while (--iterations > 0);
        */
        
        // draw tiles
        ///*
        for (var k=0, len = tilesToDraw.length; k<len; k++) {
        	var alpha = null;
        	var img = tilesToDraw[k].img;
        	if (img.fadingStart) {
        		if (img.fadingEnd && now<img.fadingEnd) {
        			alpha = 0.2 + (now - img.fadingStart)/(img.fadingEnd - img.fadingStart)*0.8;
        		}
        	}
        	this.drawOneTile(ctx, img, tilesToDraw[k].corners, img.width, alpha);
        }
        //*/
    

        // demande de chargement des tuiles manquantes et mise à jour lastUpdateDateNeededTiles
        if (updateNeededTiles) {
            // demande de chargement des tuiles
            for (var k=0, len = tilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(tilesToDownload[k].img, tilesToDownload[k].url, this.useCors);
            }
            //demande de chargement des tuiles parentes
            for (var k=0, len = parentTilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(parentTilesToDownload[k].img, parentTilesToDownload[k].url, this.useCors);
            }
            this.lastUpdateDateNeededTiles = now;
        }
        if (missingTiles) {
            // callback pour redemander un display dans 1000ms
            this.view.requestRedrawAtDate(now+HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY+10);
        }
    };
    
    function dist2(x1,y1,x2,y2) {
    	return Math.pow(x2-x1, 2) + Math.pow(y2-y1, 2);
    }
    
    HpxImageSurvey.prototype.drawOneTile = function(ctx, img, cornersXYView, textureSize, alpha) {
        
        // apply CM
        var newImg = this.useCors ? this.cm.apply(img) : img;
        
        
    	// is the tile a diamond ?
    //	var round = AladinUtils.myRound;
    //	var b = cornersXYView;
    //	var flagDiamond =  round(b[0].vx - b[2].vx) == round(b[1].vx - b[3].vx)
    //    				&& round(b[0].vy - b[2].vy) == round(b[1].vy - b[3].vy); 
    	
    	                  
    	
    	
    	drawTexturedTriangle(ctx, newImg,
                cornersXYView[0].vx, cornersXYView[0].vy,
                cornersXYView[1].vx, cornersXYView[1].vy,
    	        cornersXYView[3].vx, cornersXYView[3].vy,
    	        textureSize-1, textureSize-1,
    	        textureSize-1, 0,
    	        0, textureSize-1,
    	        alpha);
        drawTexturedTriangle(ctx, newImg,
        		cornersXYView[1].vx, cornersXYView[1].vy,
        		cornersXYView[3].vx, cornersXYView[3].vy,
        		cornersXYView[2].vx, cornersXYView[2].vy,
        		textureSize-1, 0,
        		0, textureSize-1,
        		0, 0,
        		alpha);
    };
    
    
    	        
    
    // uses affine texture mapping to draw a textured triangle
    // at screen coordinates [x0, y0], [x1, y1], [x2, y2] from
    // img *pixel* coordinates [u0, v0], [u1, v1], [u2, v2]
    // code from http://www.dhteumeuleu.com/lab/image3D.html
    function drawTexturedTriangle(ctx, img, x0, y0, x1, y1, x2, y2,
                                        u0, v0, u1, v1, u2, v2, alpha) {
    	
        // ---- centroid ----
        var xc = (x0 + x1 + x2) / 3;
        var yc = (y0 + y1 + y2) / 3;
        ctx.save();
        if (alpha) {
        	ctx.globalAlpha = alpha;
        }
    
        ctx.beginPath();
        var coeff = 0.05;
        // ---- scale triangle by 1.05 to remove anti-aliasing and draw ----
        ctx.moveTo(((1+coeff) * x0 - xc * coeff), ((1+coeff) * y0 - yc * coeff));
        ctx.lineTo(((1+coeff) * x1 - xc * coeff), ((1+coeff) * y1 - yc * coeff));
        ctx.lineTo(((1+coeff) * x2 - xc * coeff), ((1+coeff) * y2 - yc * coeff));
        ctx.closePath();
        ctx.clip();
        // ---- transform texture ----
        var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
        ctx.transform(
            -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
             (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
             (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
            -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
             (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
             (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
        );
        //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); // faster ??
        ctx.drawImage(img, 0, 0); // slower ??
        
    //    ctx.globalAlpha = 1.0;
    
        ctx.restore();
    }
    
    /*
    function drawTexturedTriangle4Points(ctx, img, x0, y0, x1, y1, x2, y2,
            u0, v0, u1, v1, u2, v2) {
    
    	var x3 = x1+x2-x0;
    	var y3 = y1+y2-y0;
    // ---- centroid ----
    var xc = (x0 + x1 + x2 + x3) / 4;
    var yc = (y0 + y1 + y2 + y3) / 4;
    ctx.save();
    ctx.beginPath();
    // ---- scale triagle by 1.05 to remove anti-aliasing and draw ----
    ctx.moveTo((1.05 * x0 - xc * 0.05), (1.05 * y0 - yc * 0.05));
    ctx.lineTo((1.05 * x1 - xc * 0.05), (1.05 * y1 - yc * 0.05));
    ctx.lineTo((1.05 * x3 - xc * 0.05), (1.05 * y3 - yc * 0.05));
    ctx.lineTo((1.05 * x2 - xc * 0.05), (1.05 * y2 - yc * 0.05));
    ctx.closePath();
    ctx.clip();
    // ---- transform texture ----
    var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
    ctx.transform(
    -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
    (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
    (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
    -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
    (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
    (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
    );
    //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); // faster ??
    ctx.drawImage(img, 0, 0); // slower ??
    
    ctx.restore();
    }
    */
    
    function grow(b, val)  {
    
    	  var b1 = new Array(b.length);
    	  for( var i=0; i<4; i++ ) {
    		  b1[i] = {vx: b[i].vx, vy: b[i].vy};
    	  }
    
    	  for( var i=0; i<2; i++ ) {
    	     var a= i==1 ? 1 : 0;
    	     var c= i==1 ? 2 : 3;
    
    	     var angle = Math.atan2(b1[c].vy-b1[a].vy, b1[c].vx-b1[a].vx);
    	     var chouilla = val*Math.cos(angle);
    	     b1[a].vx -= chouilla;
    	     b1[c].vx += chouilla;
    	     chouilla = val*Math.sin(angle);
    	     b1[a].vy -= chouilla;
    	     b1[c].vy += chouilla;
    	  }
      return b1;
    }
    
    // @api
    HpxImageSurvey.prototype.setAlpha = function(alpha) {
        alpha = +alpha; // coerce to number
        this.alpha = Math.max(0, Math.min(alpha, 1));
        this.view.requestRedraw();
    };
    
    // @api
    HpxImageSurvey.prototype.getAlpha = function() {
        return this.alpha;
    }

    return HpxImageSurvey;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HealpixGrid
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HealpixGrid = (function() {
	var HealpixGrid = function() {
	};
	
	HealpixGrid.prototype.redraw = function(ctx, cornersXYViewMap, fov, norder) {
		// on dessine les lignes
		ctx.lineWidth = 1;
		ctx.strokeStyle = "rgb(150,150,220)";
		ctx.beginPath();
		var cornersXYView;
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;
			
			// draw pixel
			ctx.moveTo(cornersXYView[0].vx, cornersXYView[0].vy);
			ctx.lineTo(cornersXYView[1].vx, cornersXYView[1].vy);
			ctx.lineTo(cornersXYView[2].vx, cornersXYView[2].vy);
			//ctx.lineTo(cornersXYView[3].vx, cornersXYView[3].vy);
			

            //ctx.strokeText(ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
		
		// on dessine les numéros de pixel HEALpix
        ctx.strokeStyle="#FFDDDD";
		ctx.beginPath();
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;

            ctx.strokeText(norder + '/' + ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
	};

	
	
	return HealpixGrid;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Location = (function() {
    // constructor
    Location = function(locationDiv) {
    		this.div = $(locationDiv);
    	};
	
	Location.prototype.update = function(lon, lat, cooFrame) {
		var coo = new Coo(lon, lat, 7);
		if (cooFrame==CooFrameEnum.J2000) {
            this.div.html(coo.format('s/'));
        }
        else {
            this.div.html(coo.format('d/'));
        }
	};
	
	return Location;
})();
	
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File View.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

View = (function() {

    /** Constructor */
    function View (aladin, location, fovDiv, cooFrame, zoom) {
            this.aladin = aladin;
            this.options = aladin.options;
    		this.aladinDiv = this.aladin.aladinDiv;
            this.popup = new Popup(this.aladinDiv);

    		this.createCanvases();
    		this.location = location;
    		this.fovDiv = fovDiv;
    		this.mustClearCatalog = true;
    		this.mustRedrawReticle = true;
    		
    		this.mode = View.PAN;
    		
    		this.minFOV = this.maxFOV = null; // by default, no restriction
    		
    		this.healpixGrid = new HealpixGrid(this.imageCanvas);
    		if (cooFrame) {
                this.cooFrame = cooFrame;
            }
            else {
                this.cooFrame = CooFrameEnum.GAL;
            }
    		
    		var lon, lat;
    		lon = lat = 0;
    		
    		this.projectionMethod = ProjectionEnum.SIN;
    		this.projection = new Projection(lon, lat);
    		this.projection.setProjection(this.projectionMethod);
            this.zoomLevel = 0;
            this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
    
    		this.viewCenter = {lon: lon, lat: lat}; // position of center of view
    		
    		if (zoom) {
                this.setZoom(zoom);
            }
    		
    		// current image survey displayed
    		this.imageSurvey = null;
    		// current catalog displayed
    		this.catalogs = [];
            // overlays (footprints for instance)
    		this.overlays = [];
    		
    
    		
    		this.tileBuffer = new TileBuffer(); // tile buffer is shared across different image surveys
    		this.fixLayoutDimensions();
            
    
    		this.curNorder = 1;
    		this.realNorder = 1;
    		
    		// some variables for mouse handling
    		this.dragging = false;
    		this.dragx = null;
    		this.dragy = null;
    		this.needRedraw = true;
    
            this.downloader = new Downloader(this); // the downloader object is shared across all HpxImageSurveys
            this.flagForceRedraw = false;
    
            this.fadingLatestUpdate = null;
    		
            this.dateRequestRedraw = null;
            
    		init(this);
    		

    		// listen to window resize and reshape canvases
    		this.resizeTimer = null;
    		var self = this;
    		$(window).resize(function() {
    		    clearTimeout(self.resizeTimer);
    		    self.resizeTimer = setTimeout(function() {self.fixLayoutDimensions(self)}, 100);
    		});
    	};
	
    // différents modes
    View.PAN = 0;
    View.SELECT = 1;
    	
    	
	View.DRAW_SOURCES_WHILE_DRAGGING = true;
	
	
	// (re)create needed canvases
	View.prototype.createCanvases = function() {
	    var a = $(this.aladinDiv);
	    a.find('.aladin-imageCanvas').remove();
	    a.find('.aladin-catalogCanvas').remove();
	    a.find('.aladin-reticleCanvas').remove();
        
        // canvas to draw the images
        this.imageCanvas = $("<canvas class='aladin-imageCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the catalogs
        this.catalogCanvas = $("<canvas class='aladin-catalogCanvas'></canvas>").appendTo(this.aladinDiv)[0];
        // canvas to draw the reticle
        this.reticleCanvas = $("<canvas class='aladin-reticleCanvas'></canvas>").appendTo(this.aladinDiv)[0];
	};
	
	
	// called at startup and when window is resized
	View.prototype.fixLayoutDimensions = function() {
        Utils.cssScale = undefined;
		
        this.width = $(this.aladinDiv).width();
		this.height = $(this.aladinDiv).height();
		
		
		this.cx = this.width/2;
		this.cy = this.height/2;
		
		this.largestDim = Math.max(this.width, this.height);
		this.smallestDim = Math.min(this.width, this.height);
		this.ratio = this.largestDim/this.smallestDim;

		
		this.mouseMoveIncrement = 160/this.largestDim;

		
		// reinitialize 2D context
		this.imageCtx = this.imageCanvas.getContext("2d");
		this.catalogCtx = this.catalogCanvas.getContext("2d");
		this.reticleCtx = this.reticleCanvas.getContext("2d");
		
		this.imageCtx.canvas.width = this.width;
		this.catalogCtx.canvas.width = this.width;
        this.reticleCtx.canvas.width = this.width;

		
		this.imageCtx.canvas.height = this.height;
		this.catalogCtx.canvas.height = this.height;
        this.reticleCtx.canvas.height = this.height;
        
        this.computeNorder();
        this.requestRedraw();
	};
    

	View.prototype.setMode = function(mode) {
	    this.mode = mode;
	    if (this.mode==View.SELECT) {
	        this.setCursor('crosshair');
	    }
	    else {
	        this.setCursor('default');
	    }
	};
	
	View.prototype.setCursor = function(cursor) {
        if (this.reticleCanvas.style.cursor==cursor) {
            return;
        }
	    this.reticleCanvas.style.cursor = cursor;
	};

	
	
	/**
	 * return dataURL string corresponding to the current view
	 */
	View.prototype.getCanvasDataURL = function() {
	    var c = document.createElement('canvas');
        c.width = this.width;
        c.height = this.height;
        var ctx = c.getContext('2d');
        ctx.drawImage(this.imageCanvas, 0, 0);
        ctx.drawImage(this.catalogCanvas, 0, 0);
        ctx.drawImage(this.reticleCanvas, 0, 0);
        
	    return c.toDataURL("image/png");
	};


	/**
	 * Compute the FoV in degrees of the view and update mouseMoveIncrement
	 * 
	 * @param view
	 * @returns FoV (array of 2 elements : width and height) in degrees
	 */
	computeFov = function(view) {
		var fov = doComputeFov(view, view.zoomFactor);
		
		
		view.mouseMoveIncrement = fov/view.imageCanvas.width;
			
		return fov;
	};
	
	doComputeFov = function(view, zoomFactor) {
	 // if zoom factor < 1, we view 180°
        if (view.zoomFactor<1) {
            fov = 180;
        }
        else {
            // TODO : fov sur les 2 dimensions !!
            // to compute FoV, we first retrieve 2 points at coordinates (0, view.cy) and (width-1, view.cy)
            var xy1 = AladinUtils.viewToXy(0, view.cy, view.width, view.height, view.largestDim, zoomFactor);
            var lonlat1 = view.projection.unproject(xy1.x, xy1.y);
            
            var xy2 = AladinUtils.viewToXy(view.imageCanvas.width-1, view.cy, view.width, view.height, view.largestDim, zoomFactor);
            var lonlat2 = view.projection.unproject(xy2.x, xy2.y);
            
            
            fov = new Coo(lonlat1.ra, lonlat1.dec).distance(new Coo(lonlat2.ra, lonlat2.dec));
        }
        
        return fov;
	};
	
	updateFovDiv = function(view) {
	    if (isNaN(view.fov)) {
	        view.fovDiv.html("FoV:");
	        return;
	    }
        // màj valeur FoV
        var fovStr;
        if (view.fov>1) {
            fovStr = Math.round(view.fov*100)/100 + "°";
        }
        else if (view.fov*60>1) {
            fovStr = Math.round(view.fov*60*100)/100 + "'";
        }
        else {
            fovStr = Math.round(view.fov*3600*100)/100 + '"';
        }
        view.fovDiv.html("FoV: " + fovStr);
	};
	
	
	createListeners = function(view) {
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
            hasTouchEvents = true;
        }
        
        // various listeners
        onDblClick = function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
            try {
                var lonlat = view.projection.unproject(xy.x, xy.y);
            }
            catch(err) {
                return;
            }
            radec = [];
            // convert to J2000 if needed
            if (view.cooFrame==CooFrameEnum.GAL) {
                radec = CooConversion.GalacticToJ2000([lonlat.ra, lonlat.dec]);
            }
            else {
                radec = [lonlat.ra, lonlat.dec];
            }

            view.pointTo(radec[0], radec[1]);
        };
        if (! hasTouchEvents) {
            $(view.reticleCanvas).dblclick(onDblClick);
        }
        
        
        $(view.reticleCanvas).bind("mousedown touchstart", function(e) {
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if (e.originalEvent && e.originalEvent.targetTouches) {
                view.dragx = e.originalEvent.targetTouches[0].clientX;
                view.dragy = e.originalEvent.targetTouches[0].clientY;
            }
            else {
                /*
                view.dragx = e.clientX;
                view.dragy = e.clientY;
                */
                view.dragx = xymouse.x;
                view.dragy = xymouse.y;
            }
            view.dragging = true;
            if (view.mode==View.PAN) {
                view.setCursor('move');
            }
            else if (view.mode==View.SELECT) {
                view.selectStartCoo = {x: view.dragx, y: view.dragy};
            }
            return false; // to disable text selection
        });
        $(view.reticleCanvas).bind("mouseup mouseout touchend", function(e) {
            if (view.mode==View.SELECT && view.dragging) {
                view.aladin.fire('selectend', 
                                 view.getObjectsInBBox(view.selectStartCoo.x, view.selectStartCoo.y,
                                                       view.dragx-view.selectStartCoo.x, view.dragy-view.selectStartCoo.y));    
            }
            if (view.dragging) {
                view.setCursor('default');
                view.dragging = false;
                
            }
            view.mustClearCatalog = true;
            view.mustRedrawReticle = true; // pour effacer selection bounding box
            view.dragx = view.dragy = null;


            var xymouse = view.imageCanvas.relMouseCoords(e);
            // popup to show ?
            var objs = view.closestObjects(xymouse.x, xymouse.y, 5);
            if (objs) {
                var o = objs[0];
                // display marker
                if (o.marker) {
                    view.popup.setTitle(o.popupTitle);
                    view.popup.setText(o.popupDesc);
                    view.popup.setSource(o);
                    view.popup.show();
                }
                // show measurements
                else {
                    // TODO: show measurements
                    if (view.aladin.objClickedFunction) {
                        var ret = view.aladin.objClickedFunction(o);
                    }
                }
            }


            // TODO : remplacer par mécanisme de listeners
            // on avertit les catalogues progressifs
            if (e.type!=="mouseout") {
                view.refreshProgressiveCats();
            }
            view.requestRedraw();
        });
        $(view.reticleCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();
            var xymouse = view.imageCanvas.relMouseCoords(e);
            if (!view.dragging || hasTouchEvents) {
                    updateLocation(view, xymouse.x, xymouse.y, true);
                    /*
                    var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                    var lonlat;
                    try {
                        lonlat = view.projection.unproject(xy.x, xy.y);
                    }
                    catch(err) {
                    }
                    if (lonlat) {
                        view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, true);
                    }
                    */
                if (!view.dragging && ! view.mode==View.SELECT) {
                    // objects under the mouse ?
                    var closest = view.closestObjects(xymouse.x, xymouse.y, 5);
                    if (closest) {
                        view.setCursor('pointer');
                        if (view.aladin.objHoveredFunction) {
                            var ret = view.aladin.objHoveredFunction(closest[0]);
                        }
                    }
                    else {
                        view.setCursor('default');
                    }
                }
                if (!hasTouchEvents) return;
            }

            var xoffset, yoffset;
            var pos1, pos2;
            
            if (e.originalEvent && e.originalEvent.targetTouches) {
                // ???
                xoffset = e.originalEvent.targetTouches[0].clientX-view.dragx;
                yoffset = e.originalEvent.targetTouches[0].clientY-view.dragy;
                var xy1 = AladinUtils.viewToXy(e.originalEvent.targetTouches[0].clientX, e.originalEvent.targetTouches[0].clientY, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);
            }
            else {
                /*
                xoffset = e.clientX-view.dragx;
                yoffset = e.clientY-view.dragy;
                */
                xoffset = xymouse.x-view.dragx;
                yoffset = xymouse.y-view.dragy;
                
                var xy1 = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                
                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);
                
            }
            
            // TODO : faut il faire ce test ??
//            var distSquared = xoffset*xoffset+yoffset*yoffset;
//            if (distSquared<3) {
//                return;
//            }
            if (e.originalEvent && e.originalEvent.targetTouches) {
                view.dragx = e.originalEvent.targetTouches[0].clientX;
                view.dragy = e.originalEvent.targetTouches[0].clientY;
            }
            else {
                view.dragx = xymouse.x;
                view.dragy = xymouse.y;
                /*
                view.dragx = e.clientX;
                view.dragy = e.clientY;
                */
            }
            
            if (view.mode==View.SELECT) {
                  view.requestRedraw();
                  return;
            }

            //view.viewCenter.lon += xoffset*view.mouseMoveIncrement/Math.cos(view.viewCenter.lat*Math.PI/180.0);
            /*
            view.viewCenter.lon += xoffset*view.mouseMoveIncrement;
            view.viewCenter.lat += yoffset*view.mouseMoveIncrement;
            */
            view.viewCenter.lon += pos2.ra -  pos1.ra;
            view.viewCenter.lat += pos2.dec - pos1.dec;
            

            
            // can not go beyond poles
            if (view.viewCenter.lat>90) {
                view.viewCenter.lat = 90;
            }
            else if (view.viewCenter.lat < -90) {
                view.viewCenter.lat = -90;
            }
            
            // limit lon to [0, 360]
            if (view.viewCenter.lon < 0) {
                view.viewCenter.lon = 360 + view.viewCenter.lon;
            }
            else if (view.viewCenter.lon > 360) {
                view.viewCenter.lon = view.viewCenter.lon % 360;
            }
            view.requestRedraw();
        }); //// endof mousemove ////
        
        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }

        $(view.reticleCanvas).bind('mousewheel', function(event, delta) {
            event.preventDefault();
            event.stopPropagation();
            var level = view.zoomLevel;
            if (delta>0) {
                level += 1;
            }
            else {
                level -= 1;
            }
            view.setZoomLevel(level);
            
            return false;
        });

	};
	
	init = function(view) {
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#aladin-statsDiv').length>0) {
        	$('#aladin-statsDiv')[0].appendChild( stats.domElement );
        }
        
        view.stats = stats;

        createListeners(view);

        view.displayHpxGrid = false;
        view.displaySurvey = true;
        view.displayCatalog = false;
        view.displayReticle = true;
        
		// initial draw
		view.fov = computeFov(view);
		updateFovDiv(view);
		
		view.redraw();
	};

	function updateLocation(view, x, y, italic) {
	    if (!view.projection) {
	        return;
	    }
	    var xy = AladinUtils.viewToXy(x, y, view.width, view.height, view.largestDim, view.zoomFactor);
        var lonlat;
        try {
            lonlat = view.projection.unproject(xy.x, xy.y);
        }
        catch(err) {
        }
        if (lonlat) {
            view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, italic);
        }
	}
	
	View.prototype.requestRedrawAtDate = function(date) {
	    this.dateRequestDraw = date;
	};
	
	

	/**
	 * redraw the whole view
	 */
	View.prototype.redraw = function() {
		var saveNeedRedraw = this.needRedraw;
		requestAnimFrame(this.redraw.bind(this));

		var now = new Date().getTime();
		if (this.dateRequestDraw && now>this.dateRequestDraw) {
		    this.dateRequestDraw = null;
		} 
		else if (! this.needRedraw) {
            if ( ! this.flagForceRedraw) {
			    return;
            }
            else {
                this.flagForceRedraw = false;
            }
		}
		this.stats.update();

		var imageCtx = this.imageCtx;
		//////// 1. Draw images ////////
		
		//// clear canvas ////
		// TODO : do not need to clear if fov small enough ?
		imageCtx.clearRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
		////////////////////////
		
		// black background
        if (this.projectionMethod==ProjectionEnum.SIN) {
            if (this.fov>80) {
                imageCtx.fillStyle = "rgb(0,0,0)";
                imageCtx.beginPath();
                imageCtx.arc(this.cx, this.cy, this.cx*this.zoomFactor, 0, 2*Math.PI, true);
                imageCtx.fill();
            }
            // pour éviter les losanges blancs qui apparaissent quand les tuiles sont en attente de chargement
            else if (this.fov<60) {
                imageCtx.fillStyle = "rgb(0,0,0)";
                imageCtx.fillRect(0, 0, this.imageCanvas.width, this.imageCanvas.height);
            }
        }

        
        // TODO : voir si on doit vraiment faire ces vérifs à chaque coup
		if (!this.projection) {
			this.projection = new Projection(this.viewCenter.lon, this.viewCenter.lat);
		}
		else {
			this.projection.setCenter(this.viewCenter.lon, this.viewCenter.lat);
		}
		this.projection.setProjection(this.projectionMethod);
	

		// ************* Tracé au niveau allsky (faible résolution) *****************
		var cornersXYViewMapAllsky = this.getVisibleCells(3);
		var cornersXYViewMapHighres = null;
		if (this.curNorder>=3) {
			if (this.curNorder==3) {
				cornersXYViewMapHighres = cornersXYViewMapAllsky;
			}
			else {
				cornersXYViewMapHighres = this.getVisibleCells(this.curNorder);
			}
		}

		// redraw image survey
		if (this.imageSurvey && this.imageSurvey.isReady && this.displaySurvey) {
		    // TODO : a t on besoin de dessiner le allsky si norder>=3 ?
		    // TODO refactoring : devrait être une méthode de HpxImageSurvey
			this.imageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curNorder);
            if (this.curNorder>=3) {
                this.imageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, this.curNorder);
            }
		}
		

		// TODO : does not work if different frames 
		// TODO : does not work if norder_max is greater than norder_max from imageSurvey
		if (this.overlayImageSurvey && this.overlayImageSurvey.isReady) {
		    imageCtx.globalAlpha = this.overlayImageSurvey.getAlpha();
	        if (this.fov>50) {
		        this.overlayImageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, this.fov, this.curNorder);
	        }
	        if (this.curNorder>=3) {
	            this.overlayImageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, this.curNorder);
	        }
           imageCtx.globalAlpha = 1.0;

		}
		
		
		// redraw grid
        if( this.displayHpxGrid) {
        	if (cornersXYViewMapHighres && this.curNorder>3) {
        		this.healpixGrid.redraw(imageCtx, cornersXYViewMapHighres, this.fov, this.curNorder);
        	}
            else {
        	    this.healpixGrid.redraw(imageCtx, cornersXYViewMapAllsky, this.fov, 3);
            }
        }
 		


        
		////// 2. Draw catalogues////////
		var catalogCtx = this.catalogCtx;

		var catalogCanvasCleared = false;
        if (this.mustClearCatalog) {
            catalogCtx.clearRect(0, 0, this.width, this.height);
            catalogCanvasCleared = true;
            this.mustClearCatalog = false;
        }
		if (this.catalogs && this.catalogs.length>0 && this.displayCatalog && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
		      // TODO : ne pas effacer systématiquement
	        //// clear canvas ////
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
		    }
		    for (var i=0; i<this.catalogs.length; i++) {
		        this.catalogs[i].draw(catalogCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.cooFrame);
		    }
        }

		////// 3. Draw overlays////////
        var overlayCtx = this.catalogCtx;
		if (this.overlays && this.overlays.length>0 && (! this.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, this.width, this.height);
                catalogCanvasCleared = true;
		    }
		    for (var i=0; i<this.overlays.length; i++) {
		        this.overlays[i].draw(overlayCtx, this.projection, this.cooFrame, this.width, this.height, this.largestDim, this.zoomFactor, this.cooFrame);
		    }
        }
        
		if (this.mode==View.SELECT) {
		    mustRedrawReticle = true;
		}
		////// 4. Draw reticle ///////
		// TODO : canvas supplémentaire avec réticule uniquement ? --> mustRedrawReticle
		var reticleCtx = this.reticleCtx;
		if (this.mustRedrawReticle || this.mode==View.SELECT) {
            reticleCtx.clearRect(0, 0, this.width, this.height);
		}
		if (this.displayReticle) {
		    
		    if (! this.reticleCache) {
    		    // build reticle image
    	        var c = document.createElement('canvas');
    	        var s = this.options.reticleSize;
    	        c.width = s;
    	        c.height = s;
    	        var ctx = c.getContext('2d');
    	        ctx.lineWidth = 2;
    	        ctx.strokeStyle = this.options.reticleColor;
    	        ctx.beginPath();
    	        ctx.moveTo(s/2, s/2+(s/2-1));
    	        ctx.lineTo(s/2, s/2+2);
    	        ctx.moveTo(s/2, s/2-(s/2-1));
    	        ctx.lineTo(s/2, s/2-2);
    	        
    	        ctx.moveTo(s/2+(s/2-1), s/2);
    	        ctx.lineTo(s/2+2,  s/2);
    	        ctx.moveTo(s/2-(s/2-1), s/2);
    	        ctx.lineTo(s/2-2,  s/2);
    	        
    	        ctx.stroke();
    	        
    	        this.reticleCache = c;
		    }
    	        
	        reticleCtx.drawImage(this.reticleCache, this.width/2 - this.reticleCache.width/2, this.height/2 - this.reticleCache.height/2);
		    
    		
    		this.mustRedrawReticle = false;
		}
		
		// draw selection box
		if (this.mode==View.SELECT && this.dragging) {
		    reticleCtx.fillStyle = "rgba(100, 240, 110, 0.25)";
		    var w = this.dragx - this.selectStartCoo.x;
		    var h =  this.dragy - this.selectStartCoo.y;
		    
		    reticleCtx.fillRect(this.selectStartCoo.x, this.selectStartCoo.y, w, h);
		}
        
        
 		// TODO : est ce la bonne façon de faire ?
 		if (saveNeedRedraw==this.needRedraw) {
 			this.needRedraw = false;
 		}


        // objects lookup
        if (!this.dragging) {
            this.updateObjectsLookup();
        }
	};

    View.prototype.forceRedraw = function() {
        this.flagForceRedraw = true;
    };
    
    View.prototype.refreshProgressiveCats = function() {
        if (! this.catalogs) {
            return;
        }
        for (var i=0; i<this.catalogs.length; i++) {
            if (this.catalogs[i].type=='progressivecat') {
                this.catalogs[i].loadNeededTiles();
            }
        }
    };
	
	View.prototype.getVisibleCells = function(norder, frameSurvey) {
	    if (! frameSurvey && this.imageSurvey) {
	        frameSurvey = this.imageSurvey.cooFrame;
	    }
		var cells = []; // will be returned
		var cornersXY = [];
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		var npix = HealpixIndex.nside2Npix(nside);
		var ipixCenter = null;
		
		// build list of pixels
		var pixList;
		if (this.fov>80) {
			pixList = [];
			for (var ipix=0; ipix<npix; ipix++) {
				pixList.push(ipix);
			}
		}
		else {
			var hpxIdx = new HealpixIndex(nside);
			hpxIdx.init();
			var spatialVector = new SpatialVector();
			// si frame != frame survey image, il faut faire la conversion dans le système du survey
			var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
			var radec = this.projection.unproject(xy.x, xy.y);
			var lonlat = [];
			if (frameSurvey && frameSurvey != this.cooFrame) {
				if (frameSurvey==CooFrameEnum.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]); 
                }
                else if (frameSurvey==CooFrameEnum.GAL) {
                    lonlat = CooConversion.J2000ToGalactic([radec.ra, radec.dec]);
                }
			}
			else {
				lonlat = [radec.ra, radec.dec];
			}
			spatialVector.set(lonlat[0], lonlat[1]);
			var radius = this.fov*0.5*this.ratio;
			// we need to extend the radius
			if (this.fov>60) {
				radius *= 1.6;
			}
			else if (this.fov>12) {
				radius *=1.45;
			}
            else {
                radius *= 1.1;
            }
			
			
				
			pixList = hpxIdx.queryDisc(spatialVector, radius*Math.PI/180.0, true, true);
			// add central pixel at index 0
			var polar = Utils.radecToPolar(lonlat[0], lonlat[1]);
			ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
			pixList.unshift(ipixCenter);
		}
		
		
		var ipix;
		var lon, lat;
		for (var ipixIdx=0, len=pixList.length; ipixIdx<len; ipixIdx++) {
			ipix = pixList[ipixIdx];
			if (ipix==ipixCenter && ipixIdx>0) { 
				continue;
			}
			var cornersXYView = [];
			corners = HealpixCache.corners_nest(ipix, nside);

			for (var k=0; k<4; k++) {
				spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	            // need for frame transformation ?
	            if (frameSurvey && frameSurvey != this.cooFrame) {
	                if (frameSurvey==CooFrameEnum.J2000) {
	                    var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	                else if (frameSurvey==CooFrameEnum.GAL) {
	                    var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	            }
	            else {
	                lon = spVec.ra();
	                lat = spVec.dec();
	            }
	            
				cornersXY[k] = this.projection.project(lon, lat);
			}


			if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
	            continue;
	        }



			for (var k=0; k<4; k++) {
				cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
			}

            var indulge = 10;
            // detect pixels outside view. Could be improved !
            // we minimize here the number of cells returned
            if( cornersXYView[0].vx<0 && cornersXYView[1].vx<0 && cornersXYView[2].vx<0 &&cornersXYView[3].vx<0) {
                continue;
            }
            if( cornersXYView[0].vy<0 && cornersXYView[1].vy<0 && cornersXYView[2].vy<0 &&cornersXYView[3].vy<0) {
                continue;
            }
            if( cornersXYView[0].vx>=this.width && cornersXYView[1].vx>=this.width && cornersXYView[2].vx>=this.width &&cornersXYView[3].vx>=this.width) {
                continue;
            }
            if( cornersXYView[0].vy>=this.height && cornersXYView[1].vy>=this.height && cornersXYView[2].vy>=this.height &&cornersXYView[3].vy>=this.height) {
                continue;
            }


			// check if pixel is visible
//			if (this.fov<160) { // don't bother checking if fov is large enough
//				if ( ! AladinUtils.isHpxPixVisible(cornersXYView, this.width, this.height) ) {
//					continue;
//				}
//			}
			// check if we have a pixel at the edge of the view in AITOFF --> TO BE MODIFIED
			if (this.projection.PROJECTION==ProjectionEnum.AITOFF) {
				var xdiff = cornersXYView[0].vx-cornersXYView[2].vx;
				var ydiff = cornersXYView[0].vy-cornersXYView[2].vy;
				var distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
				xdiff = cornersXYView[1].vx-cornersXYView[3].vx;
				ydiff = cornersXYView[1].vy-cornersXYView[3].vy;
				distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
			}
			
			cornersXYView.ipix = ipix;
			cells.push(cornersXYView);
		}
		
		return cells;
	};
	
	
	
	// get position in view for a given HEALPix cell
	View.prototype.getPositionsInView = function(ipix, norder) {
		var cornersXY = [];
		var lon, lat;
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		
		
		var cornersXYView = [];  // will be returned
		var corners = HealpixCache.corners_nest(ipix, nside);

		for (var k=0; k<4; k++) {
			spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	        // need for frame transformation ?
			if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
	            if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
	                var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	            else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
	                var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	        }
	        else {
	            lon = spVec.ra();
	            lat = spVec.dec();
	        }
	            
			cornersXY[k] = this.projection.project(lon, lat);
		}
		
		if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
            return null;
        }


		for (var k=0; k<4; k++) {
			cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
		}

		return cornersXYView;
	};
	
	
	View.prototype.computeZoomFactor = function(level) {
    	if (level>0) {
    	    return AladinUtils.getZoomFactorForAngle(180/Math.pow(1.15, level), this.projectionMethod);
		}
		else {
		    return 1 + 0.1*level;
		}
	};
	
	View.prototype.setZoom = function(fovDegrees) {
	    if (fovDegrees<0 || fovDegrees>180) {
	        return;
	    }
	    var zoomLevel = Math.log(180/fovDegrees)/Math.log(1.15);
	    this.setZoomLevel(zoomLevel);
	};

	
    View.prototype.setZoomLevel = function(level) {
        if (this.minFOV || this.maxFOV) {
            var newFov = doComputeFov(this, this.computeZoomFactor(Math.max(-2, level)));
            if (this.maxFOV && newFov>this.maxFOV  ||  this.minFOV && newFov<this.minFOV)  {
                return;
            }
        }
        
        if (this.projectionMethod==ProjectionEnum.SIN) {
            this.zoomLevel = Math.max(-2, level); // TODO : canvas freezes in firefox when max level is small
        }
        else {
            this.zoomLevel = Math.max(-7, level); // TODO : canvas freezes in firefox when max level is small
        }
        
        this.zoomFactor = this.computeZoomFactor(this.zoomLevel);
        
        this.fov = computeFov(this);
        updateFovDiv(this);
        
        this.computeNorder();
        
        this.forceRedraw();
		this.requestRedraw();
		
        // on avertit les catalogues progressifs
        if (! this.debounceProgCatOnZoom) {
            var self = this;
            this.debounceProgCatOnZoom = Utils.debounce(function() {self.refreshProgressiveCats();}, 300);
        }
        this.debounceProgCatOnZoom();
		
    };
    
    /**
     * compute and set the norder corresponding to the current view resolution
     */
    View.prototype.computeNorder = function() {
        var resolution = this.fov / this.largestDim; // in degree/pixel
        var tileSize = 512;
        var nside = HealpixIndex.calculateNSide(3600*tileSize*resolution); // 512 = taille d'une image "tuile"
        var norder = Math.log(nside)/Math.log(2);
        norder = Math.max(norder, 1);
        this.realNorder = norder;

            
        // forcer le passage à norder 3 (sinon, on reste flou trop longtemps)
        if (this.fov<=50 && norder<=2) {
            norder = 3;
        }
           

        // si l'on ne souhaite pas afficher le allsky
        if (this.imageSurvey && norder<=2 && this.imageSurvey.minOrder>2) {
            norder = this.imageSurvey.minOrder;
        }
 
        if (this.imageSurvey && norder>this.imageSurvey.maxOrder) {
            norder = this.imageSurvey.maxOrder;
        }
        // should never happen, as calculateNSide will return something <=HealpixIndex.ORDER_MAX
        if (norder>HealpixIndex.ORDER_MAX) {
            norder = HealpixIndex.ORDER_MAX;
        }
            
        this.curNorder = norder;
    };
	
    View.prototype.untaintCanvases = function() {
        this.createCanvases();
        createListeners(this);
        this.fixLayoutDimensions();
    };
    
    
    View.prototype.setOverlayImageSurvey = function(overlayImageSurvey, callback) {
        if (! overlayImageSurvey) {
            this.overlayImageSurvey = null;
            this.requestRedraw();
            return;
        }
        
        // reset canvas to "untaint" canvas if needed
        // we test if the previous base image layer was using CORS or not
        if ($.support.cors && this.overlayImageSurvey && ! this.overlayImageSurvey.useCors) {
            this.untaintCanvases();
        }
        
        var newOverlayImageSurvey;
        if (typeof overlayImageSurvey == "string") {
            newOverlayImageSurvey = HpxImageSurvey.getSurveyFromId(overlayImageSurvey);
            if ( ! newOverlayImageSurvey) {
                newOverlayImageSurvey = HpxImageSurvey.getSurveyFromId(HpxImageSurvey.DEFAULT_SURVEY_ID);
            }
        }
        else {
            newOverlayImageSurvey = overlayImageSurvey;
        }
        newOverlayImageSurvey.isReady = false;
        this.overlayImageSurvey = newOverlayImageSurvey;
        
        var self = this;
        newOverlayImageSurvey.init(this, function() {
            //self.imageSurvey = newImageSurvey;
            self.computeNorder();
            newOverlayImageSurvey.isReady = true;
            self.requestRedraw();
            self.updateObjectsLookup();
            
            if (callback) {
                callback();
            }
        });
    };
    
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
	View.prototype.setImageSurvey = function(imageSurvey, callback) {
	    if (! imageSurvey) {
	        return;
	    }
	    
	    // reset canvas to "untaint" canvas if needed
	    // we test if the previous base image layer was using CORS or not
	    if ($.support.cors && this.imageSurvey && ! this.imageSurvey.useCors) {
	        this.untaintCanvases();
	    }
	    
		var newImageSurvey;
		if (typeof imageSurvey == "string") {
		    newImageSurvey = HpxImageSurvey.getSurveyFromId(imageSurvey);
		    if ( ! newImageSurvey) {
		        newImageSurvey = HpxImageSurvey.getSurveyFromId(HpxImageSurvey.DEFAULT_SURVEY_ID);
		    }
		}
		else {
		    newImageSurvey = imageSurvey;
		}
		newImageSurvey.isReady = false;
		this.imageSurvey = newImageSurvey;
		
        var self = this;
        newImageSurvey.init(this, function() {
            //self.imageSurvey = newImageSurvey;
            self.computeNorder();
            newImageSurvey.isReady = true;
            self.requestRedraw();
            self.updateObjectsLookup();
            
            if (callback) {
                callback();
            }
        });
	};
	
	View.prototype.requestRedraw = function() {
		this.needRedraw = true;
		//redraw(this);
	};
	
	View.prototype.changeProjection = function(projectionMethod) {
		this.projectionMethod = projectionMethod;
		this.requestRedraw();
	};

	View.prototype.changeFrame = function(cooFrame) {
		this.cooFrame = cooFrame;
        // recompute viewCenter
        if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = lb[0];
            this.viewCenter.lat = lb[1]; 
        }
        else if (this.cooFrame==CooFrameEnum.J2000) {
            var radec = CooConversion.GalacticToJ2000([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = radec[0];
            this.viewCenter.lat = radec[1]; 
        }
		this.requestRedraw();
	};

    View.prototype.showHealpixGrid = function(show) {
        this.displayHpxGrid = show;
        this.requestRedraw();
    };
    
    View.prototype.showSurvey = function(show) {
        this.displaySurvey = show;

        this.requestRedraw();
    };
    
    View.prototype.showCatalog = function(show) {
        this.displayCatalog = show;

        if (!this.displayCatalog) {
            this.mustClearCatalog = true;
        }
        this.requestRedraw();
    };
    
    View.prototype.showReticle = function(show) {
        this.displayReticle = show;

        this.mustRedrawReticle = true;
        this.requestRedraw();
    };

    View.prototype.pointTo = function(ra, dec) {
        ra = parseFloat(ra);
        dec = parseFloat(dec);
        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        if (this.cooFrame==CooFrameEnum.J2000) {
		    this.viewCenter.lon = ra;
		    this.viewCenter.lat = dec;
        }
        else if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([ra, dec]);
		    this.viewCenter.lon = lb[0];
		    this.viewCenter.lat = lb[1];
        }

        this.forceRedraw();
        this.requestRedraw();
        var self = this;
        setTimeout(function() {self.refreshProgressiveCats();}, 1000);

    };
    View.prototype.makeUniqLayerName = function(name) {
        if (! this.layerNameExists(name)) {
            return name;
        }
        for (var k=1;;++k) {
            var newName = name + '_' + k;
            if ( ! this.layerNameExists(newName)) {
                return newName;
            }
        }
    };
    View.prototype.layerNameExists = function(name) {
        var c = this.catalogs;
        for (var k=0; k<c.length; k++) {
            if (name==c[k].name) {
                return true;
            }
        }
        return false;
    };

    View.prototype.removeLayers = function() {
        this.catalogs = [];
        this.overlays = [];
        this.requestRedraw();
    };

    View.prototype.addCatalog = function(catalog) {
        catalog.name = this.makeUniqLayerName(catalog.name);
        this.catalogs.push(catalog);
        if (catalog.type=='catalog') {
            catalog.setView(this);
        }
        else if (catalog.type=='progressivecat') {
            catalog.init(this);
        }
    };
    View.prototype.addOverlay = function(overlay) {
        this.overlays.push(overlay);
        overlay.setView(this);
    };
    
    View.prototype.getObjectsInBBox = function(x, y, w, h) {
        if (w<0) {
            x = x+w;
            w = -w;
        }
        if (h<0) {
            y = y+h;
            h = -h;
        }
        var objList = [];
        var cat, sources, s;
        if (this.catalogs) {
            for (var k=0; k<this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l=0; l<sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }
                    if (s.x>=x && s.x<=x+w && s.y>=y && s.y<=y+h) {
                        objList.push(s);
                    }
                }
            }
        }
        return objList;
        
    };

    // update objLookup, lookup table 
    View.prototype.updateObjectsLookup = function() {
        this.objLookup = [];

        var cat, sources, s, x, y;
        if (this.catalogs) {
            for (var k=0; k<this.catalogs.length; k++) {
                cat = this.catalogs[k];
                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l=0; l<sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }
                    x = s.x;
                    y = s.y;
                    if (!this.objLookup[x]) {
                        this.objLookup[x] = [];
                    }
                    if (!this.objLookup[x][y]) {
                        this.objLookup[x][y] = [];
                    }
                    this.objLookup[x][y].push(s);
                }       
            }           
        }     
    }

    // return closest object within a radius of maxRadius pixels. maxRadius is an integer
    View.prototype.closestObjects = function(x, y, maxRadius) {
        if (!this.objLookup) {
            return null;
        }
        var closest, dist;
        for (var r=0; r<=maxRadius; r++) {
            closest = dist = null;
            for (var dx=-maxRadius; dx<=maxRadius; dx++) {
                if (! this.objLookup[x+dx]) {
                    continue;
                }
                for (var dy=-maxRadius; dy<=maxRadius; dy++) {
                    if (this.objLookup[x+dx][y+dy]) {
                        if (!closest) {
                            closest = this.objLookup[x+dx][y+dy];
                        }
                        else {
                            var d = dx*dx + dy*dy;
                            if (d<dist) {
                                dist = d;
                                closest = this.objLookup[x+dx][y+dy];
                            }
                        }
                    }
                }
            }
            if (closest) {
                return closest;
            }
        }
        return null;
    };
    
    return View;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Aladin.js (main class)
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

/** @license CDS - Centre de Données astronomiques de Strasbourg , 2013*/
Aladin = (function() {
    
    // Constructor
    var Aladin = function(aladinDiv, requestedOptions) {
	    HealpixCache.init();
        
	    var self = this;
	    // retrieve available surveys
	    $.ajax({
	        url: "http://aladin.u-strasbg.fr/java/nph-aladin.pl",
	        data: {"frame": "aladinLiteDic"},
	        method: 'GET',
	        dataType: 'jsonp',
	        success: function(data) {
                var map = {};
                for (var k=0; k<data.length; k++) {
                    map[data[k].id] = true;
                }
                // retrieve existing surveys
                for (var k=0; k<HpxImageSurvey.SURVEYS.length; k++) {
                    if (! map[HpxImageSurvey.SURVEYS[k].id]) {
                        data.push(HpxImageSurvey.SURVEYS[k]);
                    }
                }
	            HpxImageSurvey.SURVEYS = data;
	        },
	        error: function() {
	        }
	    });
	    
	    // if not options was set, try to retrieve them from the query string
	    if (requestedOptions===undefined) {
	        requestedOptions = this.getOptionsFromQueryString();
	    }
	    requestedOptions = requestedOptions || {};
	    
	    
	    // merge with default options
	    var options = {};
	    for (var key in Aladin.DEFAULT_OPTIONS) {
	        if (requestedOptions[key] !== undefined) {
	            options[key] = requestedOptions[key];
	        }
	        else {
	            options[key] = Aladin.DEFAULT_OPTIONS[key];
	        }
	    }
	    for (var key in requestedOptions) {
	        if (Aladin.DEFAULT_OPTIONS[key]===undefined) {
	            options[key] = requestedOptions[key];
	        }
	    }
	    
        this.options = options;

	    

		this.aladinDiv = aladinDiv;

		// parent div
		$(aladinDiv).addClass("aladin-container");
		
	      
		var cooFrame = CooFrameEnum.fromString(options.cooFrame, CooFrameEnum.J2000);
		// div where we write the position
		var frameInJ2000 = cooFrame==CooFrameEnum.J2000;
        
		var locationDiv = $('<div class="aladin-location">'
		                    + (options.showFrame ? '<select class="aladin-frameChoice"><option '
		                    + (frameInJ2000 ? 'selected="selected"' : '') + '>J2000</option><option '
		                    + (! frameInJ2000 ? 'selected="selected"' : '') + '>GAL</option></select>' : '')
		                    + '<span class="aladin-location-text"></span></div>')
		                    .appendTo(aladinDiv);
		// div où on écrit la FoV
		var fovDiv = $('<div class="aladin-fov"></div>').appendTo(aladinDiv);
		
		// TODO : mettre tous les styles dans un CSS !
		
		// zoom control
        if (options.showZoomControl) {
	          $('<div class="aladin-zoomControl"><a href="#" class="zoomPlus" title="Zoom in">+</a><a href="#" class="zoomMinus" title="Zoom out">&ndash;</a></div>').appendTo(aladinDiv);
	    }
        
        // maximize control
        if (options.showFullscreenControl) {
            $('<div class="aladin-fullscreenControl aladin-maximize" title="Full screen"></div>')
                .appendTo(aladinDiv);
        }
        this.fullScreenBtn = $(aladinDiv).find('.aladin-fullscreenControl')
        this.fullScreenBtn.click(function() {
            self.toggleFullscreen();
        });

        



		// Aladin logo
		$("<div class='aladin-logo-container'><a href='http://aladin.u-strasbg.fr/AladinLite/' title='Powered by Aladin Lite' target='_blank'><div class='aladin-logo'></div></a></div>").appendTo(aladinDiv);
		
		
		// we store the boxes
		this.boxes = [];

		
		
		var location = new Location(locationDiv.find('.aladin-location-text'));
        
		// set different options
		this.view = new View(this, location, fovDiv, cooFrame, options.zoom);
		
	      // layers control panel
        // TODO : valeur des checkbox en fonction des options
		// TODO : classe LayerBox
        if (options.showLayersControl) {
            var d = $('<div class="aladin-layersControl-container" title="Manage layers"><div class="aladin-layersControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var layerBox = $('<div class="aladin-box aladin-layerBox aladin-cb-list"></div>');
            layerBox.appendTo(aladinDiv);
            
            this.boxes.push(layerBox);
            
            // we return false so that the default event is not submitted, and to prevent event bubbling
            d.click(function() {self.hideBoxes();self.showLayerBox();return false;});

        }

        
        // goto control panel
        if (options.showGotoControl) {
            var d = $('<div class="aladin-gotoControl-container" title="Go to position"><div class="aladin-gotoControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var gotoBox = 
                $('<div class="aladin-box aladin-gotoBox">' +
                  '<a class="aladin-closeBtn">&times;</a>' +
                  '<div style="clear: both;"></div>' +
                  '<form class="aladin-target-form">Go to: <input type="text" placeholder="Object name/position" /></form></div>');
            gotoBox.appendTo(aladinDiv);
            this.boxes.push(gotoBox);
            
            var input = gotoBox.find('.aladin-target-form input');
            input.on("paste keydown", function() {
                $(this).removeClass('aladin-unknownObject'); // remove red border
            });
            
            // TODO : classe GotoBox
            d.click(function() {
                self.hideBoxes();
                input.val('');
                input.removeClass('aladin-unknownObject');
                gotoBox.show();
                input.focus();
                
                
                return false;
            });
            gotoBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
        }
        
        // share control panel
        if (options.showShareControl) {
            var d = $('<div class="aladin-shareControl-container" title="Share current view"><div class="aladin-shareControl"></div></div>');
            d.appendTo(aladinDiv);
            
            var shareBox = 
                $('<div class="aladin-box aladin-shareBox">' +
                  '<a class="aladin-closeBtn">&times;</a>' +
                  '<div style="clear: both;"></div>' +
                  '<b>Share</b>' +
                  '<input type="text" class="aladin-shareInput" />' +
                  '</div>');
            shareBox.appendTo(aladinDiv);
            this.boxes.push(shareBox);
            
            
            // TODO : classe GotoBox
            d.click(function() {
                self.hideBoxes();
                shareBox.show();
                
                
                return false;
            });
            shareBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
        }
		
		
        this.gotoObject(options.target);

        if (options.log) {
            var params = requestedOptions;
            params['version'] = Aladin.VERSION;
            Logger.log("startup", params);
        }
        
		this.showReticle(options.showReticle);
		
		if (options.catalogUrls) {
		    for (var k=0, len=options.catalogUrls.length; k<len; k++) {
		        this.createCatalogFromVOTable(options.catalogUrls[k]);
		    }
		}
		
		this.setImageSurvey(options.survey);
		this.view.showCatalog(options.showCatalog);
		
	    
    	var aladin = this;
    	$(aladinDiv).find('.aladin-frameChoice').change(function() {
    		aladin.setFrame($(this).val());
    	});
    	$('#projectionChoice').change(function() {
    		aladin.setProjection($(this).val());
    	});
        

        $(aladinDiv).find('.aladin-target-form').submit(function() {
            aladin.gotoObject($(this).find('input').val(), function() {
                $(aladinDiv).find('.aladin-target-form input').addClass('aladin-unknownObject');
            });
            return false;
        });
        
        var zoomPlus = $(aladinDiv).find('.zoomPlus');
        zoomPlus.click(function() {
        	aladin.increaseZoom();
        	return false;
        });
        zoomPlus.bind('mousedown', function(e) {
            e.preventDefault(); // to prevent text selection
        });
        
        var zoomMinus = $(aladinDiv).find('.zoomMinus');
        zoomMinus.click(function() {
            aladin.decreaseZoom();
            return false;
        });
        zoomMinus.bind('mousedown', function(e) {
            e.preventDefault(); // to prevent text selection
        });
        
        // go to full screen ?
        if (options.fullScreen) {
            window.setTimeout(function() {self.toggleFullscreen();}, 1000);
        }
	};
	
    /**** CONSTANTS ****/
    Aladin.VERSION = "2014-04-24"; // will be filled by the build.sh script
    
    Aladin.JSONP_PROXY = "http://alasky.u-strasbg.fr/cgi/JSONProxy";
    
    Aladin.DEFAULT_OPTIONS = {
        target:                 "0 +0",
        cooFrame:               "J2000",
        survey:                 "P/DSS2/color",
        zoom:                   60,
        showReticle:            true,
        showZoomControl:        true,
        showFullscreenControl:  true,
        showLayersControl:      true,
        showGotoControl:        true,
        showShareControl:       false,
        showCatalog:            true, // TODO: still used ??
        showFrame:              true,
        fullScreen:             false,
        reticleColor:           "rgb(178, 50, 178)",
        reticleSize:            22,
        log:                    true
    };

    
    Aladin.prototype.toggleFullscreen = function() {
        this.fullScreenBtn.toggleClass('aladin-maximize aladin-restore');
        var isInFullscreen = this.fullScreenBtn.hasClass('aladin-restore');
        this.fullScreenBtn.attr('title', isInFullscreen ? 'Restore original size' : 'Full screen');
        $(this.aladinDiv).toggleClass('aladin-fullscreen');
        
        this.view.fixLayoutDimensions();
    };
    
    Aladin.prototype.updateSurveysDropdownList = function(surveys) {
        surveys = surveys.sort(function(a, b) {
            if (! a.order) {
                return a.id > b.id;
            }
            return a.order && a.order > b.order ? 1 : -1;
        });
        var select = $(this.aladinDiv).find('.aladin-surveySelection');
        select.empty();
        for (var i=0; i<surveys.length; i++) {
            var isCurSurvey = this.view.imageSurvey.id==surveys[i].id;
            select.append($("<option />").attr("selected", isCurSurvey).val(surveys[i].id).text(surveys[i].name));
        };
    };
    
    Aladin.prototype.getOptionsFromQueryString = function() {
        var options = {};
        var requestedTarget = $.urlParam('target');
        if (requestedTarget) {
            options.target = requestedTarget;
        }
        var requestedFrame = $.urlParam('frame');
        if (requestedFrame && CooFrameEnum[requestedFrame] ) {
            options.frame = requestedFrame;
        }
        var requestedSurveyId = $.urlParam('survey');
        if (requestedSurveyId && HpxImageSurvey.getSurveyInfoFromId(requestedSurveyId)) {
            options.survey = requestedSurveyId;
        }
        var requestedZoom = $.urlParam('zoom');
        if (requestedZoom && requestedZoom>0 && requestedZoom<180) {
            options.zoom = requestedZoom;
        }
        
        var requestedShowreticle = $.urlParam('showReticle');
        if (requestedShowreticle) {
            options.showReticle = requestedShowreticle.toLowerCase()=='true';
        }
        
        var requestedCooFrame =  $.urlParam('cooFrame');
        if (requestedCooFrame) {
            options.cooFrame = requestedCooFrame;
        }
        
        var requestedFullscreen =  $.urlParam('fullScreen');
        if (requestedFullscreen !== undefined) {
            options.fullScreen = requestedFullscreen;
        }
        
        return options;
    };
	
    // TODO: rename to setFoV
	Aladin.prototype.setZoom = function(fovDegrees) {
		this.view.setZoom(fovDegrees);
	};

	Aladin.prototype.setFoV = function(fovDegrees) {
		this.view.setZoom(fovDegrees);
	};
	
    Aladin.prototype.setFrame = function(frameName) {
        if (! frameName) {
            return;
        }
        frameName = frameName.toLowerCase();
        if (frameName.indexOf('j2000')==0) {
            this.view.changeFrame(CooFrameEnum.J2000);
        }
        else if (frameName.indexOf('gal')==0) {
            this.view.changeFrame(CooFrameEnum.GAL);
        }
    };

	Aladin.prototype.setProjection = function(projectionName) {
		if (! projectionName) {
			return;
		}
		projectionName = projectionName.toLowerCase();
		switch(projectionName) {
			case "aitoff":
				this.view.changeProjection(ProjectionEnum.AITOFF);
				break;
			case "sinus":
			default:
				this.view.changeProjection(ProjectionEnum.SIN);
		}
	}
    
    // point view to a given object (resolved by Sesame) or position
    Aladin.prototype.gotoObject = function(targetName, errorCallback) {
    	var isObjectName = /[a-zA-Z]/.test(targetName);
    	
    	// try to parse as a position
    	if ( ! isObjectName) {
    		var coo = new Coo();

			coo.parse(targetName);
			var lonlat = [coo.lon, coo.lat];
			if (this.view.cooFrame == CooFrameEnum.GAL) {
				lonlat = CooConversion.GalacticToJ2000(lonlat);
			}
    		this.view.pointTo(lonlat[0], lonlat[1]);
    	}
    	// ask resolution by Sesame
    	else {
	        var self = this;
	        Sesame.resolve(targetName,
	                       function(data) {
	        					   var ra = data.Target.Resolver.jradeg;
	        					   var dec = data.Target.Resolver.jdedeg;
	        					   self.view.pointTo(ra, dec);
	        				   /*
	                           if (data.sesame.error) {
	                                if (console) console.log(data.sesame.error);
	                           }
	                           else {
	                               var radec = data.sesame.decimalPosition.split(" ");
	                               self.view.pointTo(parseFloat(radec[0]), parseFloat(radec[1]));
	                           }
	                           */
	                       },
	                       function(data) {
	                            if (console) {
	                                console.log("Could not resolve object name " + targetName);
	                                console.log(data);
	                            }
	                            if (errorCallback) {
	                                errorCallback();
	                            }
	                       });
    	}
    };
    
    
    
    // go to a given position, expressed in the current coordinate frame
    Aladin.prototype.gotoPosition = function(lon, lat) {
        var radec;
        // first, convert to J2000 if needed
        if (this.view.cooFrame==CooFrameEnum.GAL) {
            radec = CooConversion.GalacticToJ2000([lon, lat]);
        }
        else {
            radec = [lon, lat];
        }
    	this.view.pointTo(radec[0], radec[1]);
    };
    
    Aladin.prototype.gotoRaDec = function(ra, dec) {
        this.view.pointTo(ra, dec);
    };

    Aladin.prototype.showHealpixGrid = function(show) {
        this.view.showHealpixGrid(show);
    };
    
    Aladin.prototype.showSurvey = function(show) {
        this.view.showSurvey(show);
    };
    Aladin.prototype.showCatalog = function(show) {
        this.view.showCatalog(show);
    };
    Aladin.prototype.showReticle = function(show) {
        this.view.showReticle(show);
        $('#displayReticle').attr('checked', show);
    };
    Aladin.prototype.removeLayers = function() {
        this.view.removeLayers();
    }
    Aladin.prototype.addCatalog = function(catalog) {
        this.view.addCatalog(catalog);
    };
    Aladin.prototype.addOverlay = function(overlay) {
        this.view.addOverlay(overlay);
    };
    

  
    // @oldAPI
    Aladin.prototype.createImageSurvey = function(id, name, rootUrl, cooFrame, maxOrder, options) {
        return new HpxImageSurvey(id, name, rootUrl, cooFrame, maxOrder, options);        
    };
 
    // @api
    Aladin.prototype.getBaseImageLayer = function() {
        return this.view.imageSurvey;
    };
    // @param imageSurvey : HpxImageSurvey object or image survey identifier
    // @api
    // @old
    Aladin.prototype.setImageSurvey = function(imageSurvey, callback) {
    	this.view.setImageSurvey(imageSurvey, callback);
        this.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
        if (this.options.log) {
            var id = imageSurvey;
            if (typeof imageSurvey !== "string") {
                id = imageSurvey.rootUrl;
            }

            Logger.log("changeImageSurvey", id);
        }
    };
    // @api
    Aladin.prototype.setBaseImageLayer = Aladin.prototype.setImageSurvey;
    
    // @api
    Aladin.prototype.getOverlayImageLayer = function() {
        return this.view.overlayImageSurvey;
    };
    // @api
    Aladin.prototype.setOverlayImageLayer = function(imageSurvey, callback) {
        this.view.setOverlayImageSurvey(imageSurvey, callback);
    };
    
    Aladin

    Aladin.prototype.increaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel+step);
    };
    
    Aladin.prototype.decreaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel-step);
    };
    
    Aladin.prototype.createCatalog = function(options) {
        return new cds.Catalog(options);
    };

    Aladin.prototype.createProgressiveCatalog = function(url, frame, maxOrder, options) {
        return new ProgressiveCat(url, frame, maxOrder, options);
    };
    
    // @oldAPI
    Aladin.prototype.createSource = function(ra, dec, data) {
        return new cds.Source(ra, dec, data);
    };
    Aladin.prototype.createMarker = function(ra, dec, options, data) {
        options = options || {};
        options['marker'] = true;
        return new cds.Source(ra, dec, data, options);
    };

    Aladin.prototype.createOverlay = function(options) {
        return new Overlay(options);
    };

    // API
    Aladin.prototype.createFootprintsFromSTCS = function(stcs) {
        var polygons = Overlay.parseSTCS(stcs);
        var fps = [];
        for (var k=0, len=polygons.length; k<len; k++) {
            fps.push(new Footprint(polygons[k]));
        }
        return fps;
    };
    
    // API
    Aladin.prototype.createCatalogFromVOTable = function(url, options) {
        var self = this;
        var catalog = self.createCatalog(options);
        cds.Catalog.parseVOTable(url, function(sources) {
            catalog.addSources(sources);
         });
        return catalog;
     };
     
     // API
     Aladin.prototype.on = function(what, myFunction) {
         if (what==='select') {
             this.selectFunction = myFunction;
         }
         else if (what=='objectClicked') {
            this.objClickedFunction = myFunction;
         }
         else if (what=='objectHovered') {
            this.objHoveredFunction = myFunction;
         }
     };
     
     Aladin.prototype.select = function() {
         this.fire('selectstart');
     };
     
     Aladin.prototype.fire = function(what, params) {
         if (what==='selectstart') {
             this.view.setMode(View.SELECT);
         }
         else if (what==='selectend') {
             this.view.setMode(View.PAN);
             if (this.selectFunction) {
                 this.selectFunction(params);
             }
         }
     };
     
     Aladin.prototype.hideBoxes = function() {
         if (this.boxes) {
             for (var k=0; k<this.boxes.length; k++) {
                 this.boxes[k].hide();
             }
         }
     };
     
     Aladin.prototype.updateCM = function() {
         
     }
     
     Aladin.prototype.showLayerBox = function() {
         var self = this;
         
         // first, update
         var layerBox = $(this.aladinDiv).find('.aladin-layerBox');
         layerBox.empty();
         layerBox.append('<a class="aladin-closeBtn">&times;</a>' +
                 '<div style="clear: both;"></div>' +
                 '<div class="aladin-label">Base image layer</div>' +
                 '<select class="aladin-surveySelection"></select>' +
                 '<div class="aladin-cmap">Color map:' +
                 '<div><select class="aladin-cmSelection"></select><button class="aladin-btn aladin-btn-small aladin-reverseCm" type="button">Reverse</button></div></div>' +
                 '<div class="aladin-box-separator"></div>' +
                 '<div class="aladin-label">Overlay layers</div>');
         
         var cmDiv = layerBox.find('.aladin-cmap');
         
         // fill color maps options
         var cmSelect = layerBox.find('.aladin-cmSelection');
         for (var k=0; k<ColorMap.MAPS_NAMES.length; k++) {
             cmSelect.append($("<option />").text(ColorMap.MAPS_NAMES[k]));
         }
         cmSelect.val(self.getBaseImageLayer().getColorMap().map);

         
         // loop over catalogs
         var cats = this.view.catalogs;
         var str = '<ul>';
         for (var k=cats.length-1; k>=0; k--) {
             var name = cats[k].name;
             var checked = '';
             if (cats[k].isShowing) {
                 checked = 'checked="checked"';
             }
             var nbSources = cats[k].getSources().length;
             var title = nbSources + ' source' + ( nbSources>1 ? 's' : '');
             str += '<li><div class="aladin-layerIcon" style="background: ' + cats[k].color + ';"></div><input type="checkbox" ' + checked + ' id="aladin_lite_' + name + '"></input><label for="aladin_lite_' + name + '" title="' + title + '">' + name + '</label></li>'
         }
         str += '</ul>';
         layerBox.append(str);
         
         layerBox.append('<div class="aladin-blank-separator"></div>');
         
         // gestion du réticule
         var checked = '';
         if (this.view.displayReticle) {
             checked = 'checked="checked"';
         }
         var reticleCb = $('<input type="checkbox" ' + checked + ' id="displayReticle" />');
         layerBox.append(reticleCb).append('<label for="displayReticle">Reticle</label><br/>');
         reticleCb.change(function() {
             self.showReticle($(this).is(':checked'));
         });
         
         // Gestion grille Healpix
         checked = '';
         if (this.view.displayHpxGrid) {
             checked = 'checked="checked"';
         }
         var hpxGridCb = $('<input type="checkbox" ' + checked + ' id="displayHpxGrid"/>');
         layerBox.append(hpxGridCb).append('<label for="displayHpxGrid">HEALPix grid</label><br/>');
         hpxGridCb.change(function() {
             self.showHealpixGrid($(this).is(':checked'));
         });
         
         
         layerBox.append('<div class="aladin-box-separator"></div>' +
              '<div class="aladin-label">Tools</div>');
         var exportBtn = $('<button class="aladin-btn" type="button">Export view as PNG</button>');
         layerBox.append(exportBtn);
         exportBtn.click(function() {
             self.exportAsPNG();
         });
                 
                 /*
                 '<div class="aladin-box-separator"></div>' +
                 '<div class="aladin-label">Projection</div>' +
                 '<select id="projectionChoice"><option>SINUS</option><option>AITOFF</option></select><br/>'
                 */

         layerBox.find('.aladin-closeBtn').click(function() {self.hideBoxes();return false;});
         
         // update list of surveys
         this.updateSurveysDropdownList(HpxImageSurvey.getAvailableSurveys());
         var surveySelection = $(this.aladinDiv).find('.aladin-surveySelection');
         surveySelection.change(function() {
             var survey = HpxImageSurvey.getAvailableSurveys()[$(this)[0].selectedIndex];
             self.setImageSurvey(survey.id, function() {
                 var baseImgLayer = self.getBaseImageLayer();
                 
                 if (baseImgLayer.useCors) {
                     // update color map list with current value color map
                     cmSelect.val(baseImgLayer.getColorMap().map);
                     cmDiv.show();
                     
                     exportBtn.show();
                 }
                 else {
                     cmDiv.hide();
                     
                     exportBtn.hide();
                 }
             });

             
             
         });
         
         //// COLOR MAP management ////////////////////////////////////////////
         // update color map
         cmDiv.find('.aladin-cmSelection').change(function() {
             var cmName = $(this).find(':selected').val();
             self.getBaseImageLayer().getColorMap().update(cmName);
         });
         
         // reverse color map
         cmDiv.find('.aladin-reverseCm').click(function() {
             self.getBaseImageLayer().getColorMap().reverse(); 
         });
         if (this.getBaseImageLayer().useCors) {
             cmDiv.show();
             exportBtn.show();
         }
         else {
             cmDiv.hide();
             exportBtn.hide();
         }
         layerBox.find('.aladin-reverseCm').parent().attr('disabled', true);
         //////////////////////////////////////////////////////////////////////
         
         
         // handler to hide/show overlays
         $(this.aladinDiv).find('.aladin-layerBox ul input').change(function() {
             var catName = ($(this).attr('id').substr(12));
             var cat = self.layerByName(catName);
             if ($(this).is(':checked')) {
                 cat.show();
             }
             else {
                 cat.hide();
             }
         });
         
         // finally show
         layerBox.show();
         
     };
     
     Aladin.prototype.layerByName = function(name) {
         var c = this.view.catalogs;
         for (var k=0; k<this.view.catalogs.length; k++) {
             if (name==c[k].name) {
                 return c[k];
             }
         }
         return null;
     };
     
     Aladin.prototype.exportAsPNG = function() {
         var dataURL = this.view.getCanvasDataURL();
         window.open(dataURL, "Aladin Lite snapshot");

     };
     
     /** limit FOV range
      * @API
      * @param minFOV in degrees when zoom in at max
      * @param maxFOV in degreen when zoom out at max
     */
     Aladin.prototype.setFOVRange = function(minFOV, maxFOV) {
         if (minFOV>maxFOV) {
             var tmp = minFOV;
             minFOV = maxFOV;
             maxFOV = tmp;
         }
         
         this.view.minFOV = minFOV;
         this.view.maxFOV = maxFOV;
         
     };
    
	return Aladin;
})();

////Nouvelle API ////
//A.polyline = ;
//@API
A.aladin = function(divSelector, options) {
  return new Aladin($(divSelector)[0], options);
};

//@API
// TODO : lecture de properties
A.imageLayer = function(id, name, rootUrl, options) {
    return new HpxImageSurvey(id, name, rootUrl, null, null, options);
};

// @API
A.source = function(ra, dec, data) {
    return new cds.Source(ra, dec, data);
};


// conservé pour compatibilité avec existant
// @oldAPI
if ($) {
    $.aladin = A.aladin;
}
    
