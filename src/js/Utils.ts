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
 * File Utils
 *
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import {JSONP_PROXY} from "@/js/Constants";
import { ALEvent } from './events/ALEvent';


export let Utils: any = {}

// list of URL domains that can be safely switched from HTTP to HTTPS
Utils.HTTPS_WHITELIST = ['alasky.u-strasbg.fr', 'alaskybis.u-strasbg.fr', 'alasky.unistra.fr', 'alaskybis.unistra.fr',
    'alasky.cds.unistra.fr', 'alaskybis.cds.unistra.fr', 'hips.astron.nl', 'jvo.nao.ac.jp',
    'archive.cefca.es', 'cade.irap.omp.eu', 'skies.esac.esa.int']

Utils.cssScale = undefined

Utils.relMouseCoords = function (event) {
    if (event.offsetX) {
        return {x: event.offsetX, y: event.offsetY}
    } else {
        if (!Utils.cssScale) {
            var st = window.getComputedStyle(document.body, null)
            var tr = st.getPropertyValue('-webkit-transform') ||
                st.getPropertyValue('-moz-transform') ||
                st.getPropertyValue('-ms-transform') ||
                st.getPropertyValue('-o-transform') ||
                st.getPropertyValue('transform')
            var matrixRegex = /matrix\((-?\d*\.?\d+),\s*0,\s*0,\s*(-?\d*\.?\d+),\s*0,\s*0\)/
            var matches = tr.match(matrixRegex)
            if (matches) {
                Utils.cssScale = parseFloat(matches[1])
            } else {
                Utils.cssScale = 1
            }
        }
        var e = event
        // http://www.jacklmoore.com/notes/mouse-position/
        var target = e.target || e.srcElement
        var style = target.currentStyle || window.getComputedStyle(target, null)
        var borderLeftWidth = parseInt(style['borderLeftWidth'], 10)
        var borderTopWidth = parseInt(style['borderTopWidth'], 10)
        var rect = target.getBoundingClientRect()

        var clientX = e.clientX
        var clientY = e.clientY

        if (e && e.changedTouches) {
            clientX = e.changedTouches[0].clientX
            clientY = e.changedTouches[0].clientY
        }

        var offsetX = clientX - borderLeftWidth - rect.left
        var offsetY = clientY - borderTopWidth - rect.top

        return {x: Math.round(offsetX / Utils.cssScale), y: Math.round(offsetY / Utils.cssScale)}
    }
}

//Function.prototype.bind polyfill from
//https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Function/bind
if (!Function.prototype.bind) {
    Function.prototype.bind = function (obj) {
        // closest thing possible to the ECMAScript 5 internal IsCallable function
        if (typeof this !== 'function') {
            throw new TypeError('Function.prototype.bind - what is trying to be bound is not callable')
        }

        var slice = [].slice,
            args = slice.call(arguments, 1),
            self = this,
            nop = function () {
            },
            bound = function () {
                return self.apply(this instanceof nop ? this : (obj || {}),
                    args.concat(slice.call(arguments)))
            }

        bound.prototype = this.prototype

        return bound
    }
}


/* source : http://stackoverflow.com/a/8764051 */
Utils.urlParam = function (name: string, queryString: string | undefined) {
    if (queryString === undefined) {
        queryString = location.search
    }
    return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(queryString) || [, ''])[1].replace(/\+/g, '%20')) || null
}

/* source: http://stackoverflow.com/a/1830844 */
Utils.isNumber = function (n: string | number) {
    return !isNaN(parseFloat(n as string)) && isFinite(n as number)
}

Utils.isInt = function (n: string | number) {
    return Utils.isNumber(n) && Math.floor(n as number) === n
}

// Newton-Raphson method to find the approximate inverse of f(x)
Utils.inverseNewtonRaphson = function(y: number, f: Function, fPrime: Function, tolerance=1e-6, maxIterations=100) {
    let x_guess = 0.5; // Initial guess
    let iteration = 0;

    while (iteration < maxIterations) {
        let f_x = f(x_guess);
        let error = Math.abs(f_x - y);

        if (error < tolerance) {
            return x_guess; // Found approximate inverse
        }

        let derivative = fPrime(x_guess);
        x_guess = x_guess - (f_x - y) / derivative; // Newton-Raphson update
        iteration++;
    }

    return null; // No convergence within maxIterations
}

Utils.binarySearch = function(array, value) {
    var low = 0,
        high = array.length;

    while (low < high) {
        var mid = (low + high) >>> 1;
        if (array[mid] > value) low = mid + 1;
        else high = mid;
    }
    return low;
}

/* a debounce function, used to prevent multiple calls to the same function if less than delay milliseconds have passed */
Utils.debounce = function (fn, delay) {
    var timer = null
    return function () {
        var context = this, args = arguments
        clearTimeout(timer)
        timer = setTimeout(function () {
            fn.apply(context, args)
        }, delay)
    }
}

/* return a throttled function, to rate limit the number of calls (by default, one call every 250 milliseconds) */
Utils.throttle = function (fn, threshhold, scope) {
    threshhold || (threshhold = 250)
    var last,
        deferTimer
    return function () {
        var context = scope || this

        var now = +new Date,
            args = arguments
        if (last && now < last + threshhold) {
            // hold on to it
            clearTimeout(deferTimer)
            deferTimer = setTimeout(function () {
                last = now
                fn.apply(context, args)
            }, threshhold)
        } else {
            last = now
            fn.apply(context, args)
        }
    }
}


/* A LRU cache, inspired by https://gist.github.com/devinus/409353#file-gistfile1-js */
// TODO : utiliser le LRU cache pour les tuiles images
Utils.LRUCache = function (maxsize) {
    this._keys = []
    this._items = {}
    this._expires = {}
    this._size = 0
    this._maxsize = maxsize || 1024
}

Utils.LRUCache.prototype = {
    set: function (key, value) {
        var keys = this._keys,
            items = this._items,
            expires = this._expires,
            size = this._size,
            maxsize = this._maxsize

        if (size >= maxsize) { // remove oldest element when no more room
            keys.sort(function (a, b) {
                if (expires[a] > expires[b]) return -1
                if (expires[a] < expires[b]) return 1
                return 0
            })

            size--
        }

        keys[size] = key
        items[key] = value
        expires[key] = Date.now()
        size++

        this._keys = keys
        this._items = items
        this._expires = expires
        this._size = size
    },

    get: function (key) {
        var item = this._items[key]
        if (item) {
            this._expires[key] = Date.now()
        }
        return item
    },

    keys: function () {
        return this._keys
    }
}

////////////////////////////////////////////////////////////////////////////:

/**
 Fetch an url with the method GET, given a list of potential mirrors
 An optional object can be given with the following keywords accepted:
 * data: an object storing the params associated to the URL
 * contentType: specify the content type returned from the url (no verification is done, it is not mandatory to put it)
 * timeout: A maximum request time. If exceeded, the request is aborted and the next url will be fetched
 This method assumes the URL are CORS-compatible, no proxy will be used

 A promise is returned. When all the urls fail, a rejected Promise is returned so that it can be catched afterwards
 */
Utils.loadFromUrls = function (urls, options) {
    const data = options && options.data || undefined
    const timeout = options && options.timeout || 5000
    const mode = options && options.mode || 'cors';
    const contentType = options && options.contentType || undefined;
    const dataType = (options && options.dataType) || 'text';
    const desc = options && options.desc;
    // A controller that can abort the query when a timeout is reached
    const controller = new AbortController()

    // Launch a timemout that will interrupt the fetch if it has not yet succeded:
    const timeoutId = setTimeout(() => controller.abort(), timeout)
    let url = urls[0];
    if (data) {
        url += '?' + new URLSearchParams(data)
    }

    let params = {
        url,
        // *GET, POST, PUT, DELETE, etc.
        method: 'GET',
        /*headers: {
            'Content-Type': contentType
        },*/
        // no-cors, *cors, same-origin
        mode,
        // *default, no-cache, reload, force-cache, only-if-cached
        cache: 'default',
        // Abort the request when a timeout exceeded
        signal: controller.signal,
        dataType,
        // message description
        desc
    }

    if (contentType) {
        params.headers = {
            'Content-Type': contentType
        };
    }

    return Utils.fetch({
        ...params,
        success: (resp) => {
            // completed request before timeout fired
            clearTimeout(timeoutId)
            return Promise.resolve(resp)
        },
        error: (e) => {
            // The request aborted because it was to slow, fetch the next url given recursively
            // Base case, when all urls have been fetched and failed
            if (urls && urls.length === 1) {
                let errorMsg = "Status: "+ e.status +"\nMessage: " + e.statusText;
                return Promise.reject(errorMsg);
            } else {
                return Utils.loadFromUrls(urls.slice(1), options);
            }
        }
    })
}

/*
// return the jquery ajax object configured with the requested parameters
// by default, we use the proxy (safer, as we don't know if the remote server supports CORS)
Utils.getAjaxObject = function (url, method, dataType, useProxy) {
    if (useProxy !== false) {
        useProxy = true
    }

    if (useProxy === true) {
        var urlToRequest = JSONP_PROXY + '?url=' + encodeURIComponent(url)
    } else {
        urlToRequest = url
    }
    method = method || 'GET'
    dataType = dataType || null

    return $.ajax({
        url: urlToRequest,
        method: method,
        dataType: dataType
    })
}
*/

Utils.fetch = function(params) {
    let url;
    try {
        url = new URL(params.url);
        if (params.useProxy === true) {
            url = Utils.handleCORSNotSameOrigin(url)
        }
    
        if (params.data) {
            // add the search params to the url object 
            for (const key in params.data) {
                url.searchParams.append(key, params.data[key]);
            }
        }
    } catch(e) {
        // localhost url
        url = params.url;
    }
    

    let request = new Request(url, {
        method: params.method || 'GET',
        ...params
    })

    let task = {
        message: params.desc || 'fetching: ' + url,
        id: Utils.uuidv4()
    };
    ALEvent.FETCH.dispatchedTo(document, {task});

    return fetch(request)
        .then((resp) => {
            if (params.dataType && params.dataType.includes('json')) {
                return resp.json();
            } else if (params.dataType && params.dataType.includes('blob')) {
                return resp.blob();
            } else if (params.dataType && params.dataType.includes('readableStream')) {
                return Promise.resolve(resp.body);
            } else {
                return resp.text()
            }
        })
        .then(data => {
            if (params.success) {
                return params.success(data)
            }

            return Promise.resolve(data);
        })
        .catch(e => {
            if (params.error) {
                return params.error(e)
            }

            return Promise.reject(e);
        })
        .finally(() => {
            ALEvent.RESOURCE_FETCHED.dispatchedTo(document, {task});
        })
}

Utils.on = function(element, events, callback) {
    events.split(' ')
        .forEach(e => {
            element.addEventListener(e, callback)
        })
}

Utils.isTouchScreenDevice = undefined;

Utils.hasTouchScreen = function() {
    if (Utils.isTouchScreenDevice === undefined) {
        let hasTouchScreen = false;
        if ("maxTouchPoints" in navigator) {
            hasTouchScreen = navigator.maxTouchPoints > 0;
        } else if ("msMaxTouchPoints" in navigator) {
            hasTouchScreen = navigator.msMaxTouchPoints > 0;
        } else {
            const mQ = matchMedia?.("(pointer:coarse)");
            if (mQ?.media === "(pointer:coarse)") {
                hasTouchScreen = !!mQ.matches;
            } else if ("orientation" in window) {
                hasTouchScreen = true; // deprecated, but good fallback
            } else {
                // Only as a last resort, fall back to user agent sniffing
                const UA = navigator.userAgent;
                hasTouchScreen =
                /\b(BlackBerry|webOS|iPhone|IEMobile)\b/i.test(UA) ||
                /\b(Android|Windows Phone|iPad|iPod)\b/i.test(UA);
            }
        }

        Utils.isTouchScreenDevice = hasTouchScreen;
    }

    return Utils.isTouchScreenDevice;
}



Utils.openNewTab = function(url) {
    window.open(url, '_blank').focus();
}

// return true if script is executed in a HTTPS context
// return false otherwise
Utils.isFileContext = function () {
    return (window.location.protocol === 'file:')
}

Utils.isHttpsContext = function () {
    return (window.location.protocol === 'https:')
}

Utils.isHttpContext = function () {
    return (window.location.protocol === 'http:')
}

Utils.fixURLForHTTPS = function (url) {
    const switchToHttps = Utils.HTTPS_WHITELIST.some(element => {
        return url.includes(element)
    })

    if (switchToHttps) {
        return url.replace('http://', 'https://')
    }

    return url
}

Utils.isUrl = function(url) {
    try {
        return new URL(url).href;
    } catch(e) {
        return undefined;
    }
}

// generate an absolute URL from a relative URL
// example: getAbsoluteURL('foo/bar/toto') return http://cds.unistra.fr/AL/foo/bar/toto if executed from page http://cds.unistra.fr/AL/
Utils.getAbsoluteURL = function (url) {
    var a = document.createElement('a')
    a.href = url

    return a.href
}

// generate a valid v4 UUID
Utils.uuidv4 = function () {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8)
        return v.toString(16)
    })
}

/**
 * @function
 * @description Deep clone a class instance.
 * @param {object} instance The class instance you want to clone.
 * @returns {object} A new cloned instance.
 */
Utils.clone = function (instance) {
    return Object.assign(
        Object.create(
            // Set the prototype of the new object to the prototype of the instance.
            // Used to allow new object behave like class instance.
            Object.getPrototypeOf(instance),
        ),
        // Prevent shallow copies of nested structures like arrays, etc
        JSON.parse(JSON.stringify(instance)),
    )
}

Utils.getDroppedFilesHandler = function (ev) {
    // Prevent default behavior (Prevent file from being opened)
    ev.preventDefault()

    let items
    if (ev.dataTransfer.items) {
        // Use DataTransferItemList interface to access the file(s)
        items = [...ev.dataTransfer.items]
    } else {
        // Use DataTransfer interface to access the file(s)
        items = [...ev.dataTransfer.files]
    }

    const files = items.filter((item) => {
        // If dropped items aren't files, reject them
        return item.kind === 'file'
    })
        .map((item) => item.getAsFile())

    return files
}

Utils.dragOverHandler = function (ev) {
    // Prevent default behavior (Prevent file from being opened)
    ev.preventDefault()
}

Utils.requestCORSIfNotSameOrigin = function (url) {
    return (new URL(url, window.location.href)).origin !== window.location.origin
}

// Check the protocol, for http ones, use a CORS compatible proxy
Utils.handleCORSNotSameOrigin = function (url) {
    if (Utils.requestCORSIfNotSameOrigin(url)) {
        // http(s) protocols and not in localhost
        let proxiedUrl = new URL(JSONP_PROXY)
        proxiedUrl.searchParams.append('url', url)

        url = proxiedUrl
    }

    return url
}

Utils.deepCopy = function (orig) {
    return Object.assign(Object.create(Object.getPrototypeOf(orig)), orig)
}

Utils.download = function(url, name = undefined) {
    const a = document.createElement('a')
    a.href = url
    if (name) {
        a.download = name;
    }
    a.click()
}

Utils.measureTime = function(msg, method) {
    let startTime = performance.now()
    let output = method()
    let deltaTime = performance.now() - startTime;
    console.log(msg, deltaTime);

    return output;
};
