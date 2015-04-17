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



// log 
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
