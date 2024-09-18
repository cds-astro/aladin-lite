/*
 * Javascript AVM/XMP Reader 0.1.3
 * Copyright (c) 2010 Stuart Lowe http://www.strudel.org.uk/
 * Astronomy Visualization Metadata (AVM) is defined at:
 * http://www.virtualastronomy.org/avm_metadata.php
 *
 * Licensed under the MPL http://www.mozilla.org/MPL/MPL-1.1.txt
 *
 * S
 */

export let AVM = (function() {

    function AVM(input) {
        if (input instanceof HTMLImageElement) {
            this.img = input;
        } else if (input instanceof ArrayBuffer) {
            this.img = input;
        } else {
            // suppose that input is a string
            this.id = (input) ? input : "";
            this.img = { complete: false };
        }

        this.xmp = "";	// Will hold the XMP string (for test purposes)
        this.wcsdata = false;
        this.AVMdefinedTags = {
            'Creator':'photoshop:Source',
            'CreatorURL':'Iptc4xmpCore:CiUrlWork',
            'Contact.Name':'dc:creator',
            'Contact.Email':'Iptc4xmpCore:CiEmailWork',
            'Contact.Telephone':'Iptc4xmpCore:CiTelWork',
            'Contact.Address':'Iptc4xmpCore:CiAdrExtadr',
            'Contact.City':'Iptc4xmpCore:CiAdrCity',
            'Contact.StateProvince':'Iptc4xmpCore:CiAdrRegion',
            'Contact.PostalCode':'Iptc4xmpCore:CiAdrPcode',
            'Contact.Country':'Iptc4xmpCore:CiAdrCtry',
            'Rights':'xapRights:UsageTerms',
            'Title':'dc:title',
            'Headline':'photoshop:Headline',
            'Description':'dc:description',
            'Subject.Category':'avm:Subject.Category',
            'Subject.Name':'dc:subject',
            'Distance':'avm:Distance',
            'Distance.Notes':'avm:Distance.Notes',
            'ReferenceURL':'avm:ReferenceURL',
            'Credit':'photoshop:Credit',
            'Date':'photoshop:DateCreated',
            'ID':'avm:ID',
            'Type':'avm:Type',
            'Image.ProductQuality':'avm:Image.ProductQuality',
            'Facility':'avm:Facility',
            'Instrument':'avm:Instrument',
            'Spectral.ColorAssignment':'avm:Spectral.ColorAssignment',
            'Spectral.Band':'avm:Spectral.Band',
            'Spectral.Bandpass':'avm:Spectral.Bandpass',
            'Spectral.CentralWavelength':'avm:Spectral.CentralWavelength',
            'Spectral.Notes':'avm:Spectral.Notes',
            'Temporal.StartTime':'avm:Temporal.StartTime',
            'Temporal.IntegrationTime':'avm:Temporal.IntegrationTime',
            'DatasetID':'avm:DatasetID',
            'Spatial.CoordinateFrame':'avm:Spatial.CoordinateFrame',
            'Spatial.Equinox':'avm:Spatial.Equinox',
            'Spatial.ReferenceValue':'avm:Spatial.ReferenceValue',
            'Spatial.ReferenceDimension':'avm:Spatial.ReferenceDimension',
            'Spatial.ReferencePixel':'avm:Spatial.ReferencePixel',
            'Spatial.Scale':'avm:Spatial.Scale',
            'Spatial.Rotation':'avm:Spatial.Rotation',
            'Spatial.CoordsystemProjection':'avm:Spatial.CoordsystemProjection',
            'Spatial.Quality':'avm:Spatial.Quality',
            'Spatial.Notes':'avm:Spatial.Notes',
            'Spatial.FITSheader':'avm:Spatial.FITSheader',
            'Spatial.CDMatrix':'avm:Spatial.CDMatrix',
            'Publisher':'avm:Publisher',
            'PublisherID':'avm:PublisherID',
            'ResourceID':'avm:ResourceID',
            'ResourceURL':'avm:ResourceURL',
            'RelatedResources':'avm:RelatedResources',
            'MetadataDate':'avm:MetadataDate',
            'MetadataVersion':'avm:MetadataVersion'
        }
    }

    AVM.prototype.load = function(fnCallback) {
        if(!this.img && this.id) {
            this.img = document.getElementById(this.id);
        }

        if (this.img instanceof ArrayBuffer) {
            this.getData(fnCallback);
            return;
        }

        if(!this.img.complete) {
            var _obj = this;
            addEvent(this.img, "load", 
                function() {
                    _obj.getData(fnCallback);
                }
            ); 
        } else {
            this.getData(fnCallback);
        }
    }

    AVM.prototype.getData = function(fnCallback){
        if(!this.imageHasData()){
            this.getImageData(this.img, fnCallback);
        }else{
            if(typeof fnCallback=="function") fnCallback(this);
        }
        return true;
    }

    AVM.prototype.getImageData = function(oImg, fnCallback) {
        var _obj = this;

        const findAVM = (arrayBuffer) => {
            const view = new DataView(arrayBuffer);
            var oAVM = _obj.findAVMinJPEG(view);
            _obj.wcs = oAVM || {};

            _obj.wcsdata = _obj.wcs !== undefined && Object.keys(_obj.wcs).length > 0;

            if (typeof fnCallback=="function") fnCallback(_obj);
        };
        if (oImg instanceof ArrayBuffer) {
            findAVM(oImg)
        } else {
            let reqwst = new Request(oImg.src, {
                method: 'GET',
            })
            fetch(reqwst)
                .then((resp) => resp.arrayBuffer())
                .then(arrayBuffer => {
                    findAVM(arrayBuffer)
                })
        }
        
    }

    function addEvent(oElement, strEvent, fncHandler){
        if (oElement.addEventListener) oElement.addEventListener(strEvent, fncHandler, false); 
        else if (oElement.attachEvent) oElement.attachEvent("on" + strEvent, fncHandler); 
    }

    AVM.prototype.imageHasData = function() {
        return (this.img.wcsdata);
    }

    AVM.prototype.findAVMinJPEG = function(oFile) {
        if (oFile.getUint8(0) != 0xFF || oFile.getUint8(1) != 0xD8) return false; // not a valid jpeg

        var iOffset = 2;
        var iLength = oFile.byteLength;
        while (iOffset < iLength) {
            if (oFile.getUint8(iOffset) != 0xFF) return false; // not a valid marker, something is wrong
            var iMarker = oFile.getUint8(iOffset+1);

            // we could implement handling for other markers here, 
            // but we're only looking for 0xFFE1 for AVM data
            if (iMarker == 22400) {
                return this.readAVMDataAsWCS(oFile, iOffset + 4, oFile.getUint16(iOffset+2, false)-2);
                //iOffset += 2 + oFile.getUint16(iOffset+2, false);

            } else if (iMarker == 225) {
                // 0xE1 = Application-specific 1 (for AVM)
                return this.readAVMDataAsWCS(oFile, iOffset + 4, oFile.getUint16(iOffset+2, false)-2);
            } else {
                iOffset += 2 + oFile.getUint16(iOffset+2, false);
            }
        }
    }

    AVM.prototype.readAVMDataAsWCS = function(oFile) {
        var tags = undefined;

        let wcs = {};

        this.xmp = this.readXMP(oFile);

        if (this.xmp) {
            tags = this.readAVM(this.xmp);

            if (tags) {
                this.tags = tags;

                let unwindTag = (tag) => {
                    if (Array.isArray(tag)) {
                        return tag[0]
                    } else {
                        return tag;
                    }
                }

                wcs.CTYPE1 = unwindTag(tags['Spatial.CoordinateFrame']) === 'GAL' ? 'GLON-' : 'RA---';
                wcs.CTYPE1 += unwindTag(tags['Spatial.CoordsystemProjection']);
                wcs.CTYPE2 = unwindTag(tags['Spatial.CoordinateFrame']) === 'GAL' ? 'GLAT-' : 'DEC--';
                wcs.CTYPE2 += unwindTag(tags['Spatial.CoordsystemProjection']);

                if (unwindTag(tags['Spatial.Equinox']))
                    wcs.EQUINOX = +unwindTag(tags['Spatial.Equinox']);

                wcs.NAXIS1 = tags['Spatial.ReferenceDimension'] && +tags['Spatial.ReferenceDimension'][0];
                wcs.NAXIS2 = tags['Spatial.ReferenceDimension'] && +tags['Spatial.ReferenceDimension'][1];

                if (tags['Spatial.CDMatrix']) {
                    console.warn("Spatial.CDMatrix is deprecated in favor of Spatial.Scale + Spatial.Rotation");

                    wcs.CD1_1 = +tags['Spatial.CDMatrix'][0];
                    wcs.CD1_2 = +tags['Spatial.CDMatrix'][1];
                    wcs.CD2_1 = +tags['Spatial.CDMatrix'][2];
                    wcs.CD2_2 = +tags['Spatial.CDMatrix'][3];
                } else {
                    wcs.CDELT1 = tags['Spatial.Scale'] && +tags['Spatial.Scale'][0];
                    wcs.CDELT2 = tags['Spatial.Scale'] && +tags['Spatial.Scale'][1];

                    if (unwindTag(tags['Spatial.Rotation']) !== undefined) {
                        wcs.CROTA2 = +unwindTag(tags['Spatial.Rotation']);
                    }
                }

                wcs.CRPIX1 = tags['Spatial.ReferencePixel'] && +tags['Spatial.ReferencePixel'][0];
                wcs.CRPIX2 = tags['Spatial.ReferencePixel'] && +tags['Spatial.ReferencePixel'][1];

                wcs.CRVAL1 = tags['Spatial.ReferenceValue'] && +tags['Spatial.ReferenceValue'][0];
                wcs.CRVAL2 = tags['Spatial.ReferenceValue'] && +tags['Spatial.ReferenceValue'][1];
            } else {
                var equalReached = false;
                for(var key of ['NAXIS1', 'NAXIS2', 'CTYPE1', 'CTYPE2', 'CRPIX1', 'CRPIX2', 'CRVAL1', 'CRVAL2', 'LONPOLE', 'LATPOLE', 'CDELT1', 'CDELT2', 'PC1_1', 'PC2_2', 'PC1_2', 'PC2_1', 'CD1_1', 'CD2_2', 'CD1_2', 'CD2_1']) {
                    equalReached = false;
                    // try to read directly the WCS
                    let beginCard = this.xmp.slice(this.xmp.indexOf(key));
                    let values = beginCard.split(" ");
                    
                    for (var v of values) {

                        if (equalReached && v !== "") {
                            wcs[key] = parseFloat(v);
                            if (Number.isNaN(wcs[key])) {
                                if (v[0] === "'" || v[0] === "\"") {
                                    v = v.slice(1, v.length - 1)
                                }
                                wcs[key] = v;
                            }

                            break;
                        }

                        if (v === "=") {
                            equalReached = true;
                        }
                    }
                }
            }
        }
        return wcs;
    }

    AVM.prototype.readXMP = function(oFile) {
        var iEntries = oFile.byteLength;
        var prev_n_hex = '';
        var record = false;
        var recordn = 0;
        // Find the XMP packet - 8 bit encoding (UTF-8)
        // see page 34 of http://www.adobe.com/devnet/xmp/pdfs/xmp_specification.pdf
        var xmpStr = '0x3C 0x3F 0x78 0x70 0x61 0x63 0x6B 0x65 0x74 0x20 0x62 0x65 0x67 0x69 0x6E 0x3D ';
        var xmpBytes = 14;
        var byteStr = '';
        var iEntryOffset = -1;

        // Here we want to search for the XMP packet starting string
        // There is probably a more efficient way to search for a byte string
        for (var i=0;i<iEntries;i++) {

            var n = oFile.getUint8(i);
            var n_hex = n.toString(16).toUpperCase();
            if(n_hex.length == 1) n_hex = "0x0"+n_hex;
            if(n_hex.length == 2) n_hex = "0x"+n_hex;

            if(prev_n_hex == "0x3C" && n_hex == "0x3F"){
                record = true;
                recordn = xmpBytes;
                byteStr = '0x3C ';
            }
            if(record){
                byteStr += n_hex+' ';

                recordn--;
                if(recordn < 0){
                    if(byteStr == xmpStr){
                        var iEntryOffset = i-xmpBytes-1;
                        break;
                    }
                    record = false;
                }
            }
            prev_n_hex = n_hex;
        }

        if(iEntryOffset >= 0){
            var str = '';
            var i = iEntryOffset;
            while(str.indexOf('</x:xmpmeta>') < 0 && i < (iEntryOffset+20000)){
                str += String.fromCharCode(oFile.getUint8(i));
                i++;
            }
            return str;
        }
    }

    AVM.prototype.readAVM = function(str) {
        var tags = undefined;
        if(str.indexOf('xmlns:avm') >= 0){
            tags = {}
            for (var keyname in this.AVMdefinedTags) {
                var key = this.AVMdefinedTags[keyname];
                key.toLowerCase();
                var start = str.indexOf(key)+key.length+2;
                var final = str.indexOf('"',start);
                // Find out what the character is after the key
                var char = str.substring(start-2,start-1);
                if(char == "="){
                    tags[keyname] = str.substring(start,final);
                }else if(char == ">"){
                    final = str.indexOf('</'+key+'>',start);
                    // Parse out the HTML tags and build an array of the resulting values
                    var tmps = str.substring(start-1,final);

                    var tmparr = new Array(0);
                    var tmpstr = tmps.replace(/[\n\r]/g,"");
                    tmpstr = tmpstr.replace(/ +/g," ");
                    tmparr = tmpstr.split(/ ?<\/?[^>]+> ?/g);
                    var newarr = new Array(0);
                    for(var i = 0;i<tmparr.length;i++){
                        if(tmparr[i].length > 0) newarr.push(tmparr[i]);
                    }
                    tags[keyname] = newarr;
                }
            }
        }
        return tags;
    }

    return AVM;
})();