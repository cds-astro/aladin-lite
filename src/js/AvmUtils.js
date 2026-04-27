// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS

function crc32(bytes) {
    let crc = 0xffffffff;

    for (let i = 0; i < bytes.length; i++) {
        crc ^= bytes[i];

        for (let j = 0; j < 8; j++) {
            const mask = -(crc & 1);
            crc = (crc >>> 1) ^ (0xedb88320 & mask);
        }
    }

    return (crc ^ 0xffffffff) >>> 0;
}

function u32be(n) {
    return new Uint8Array([
        (n >>> 24) & 0xff,
        (n >>> 16) & 0xff,
        (n >>> 8) & 0xff,
        n & 0xff,
    ]);
}

function concatUint8Arrays(arrays) {
    const total = arrays.reduce((sum, array) => sum + array.length, 0);
    const out = new Uint8Array(total);
    let offset = 0;

    for (const array of arrays) {
        out.set(array, offset);
        offset += array.length;
    }

    return out;
}

function escapeXml(value) {
    return String(value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

function renderAvmValue(value) {
    if (Array.isArray(value)) {
        return [
            "<rdf:Seq>",
            ...value.map((item) => `  <rdf:li>${escapeXml(item)}</rdf:li>`),
            "</rdf:Seq>",
        ].join("\n");
    }

    return escapeXml(value);
}

function serializeFitsHeaderValue(value) {
    if (typeof value === "string") {
        return `'${value}'`;
    }

    if (typeof value === "boolean") {
        return value ? "T" : "F";
    }

    return `${value}`;
}

function buildFitsHeaderFromWcs(wcs) {
    const orderedKeys = [
        "NAXIS",
        "NAXIS1",
        "NAXIS2",
        "CTYPE1",
        "CTYPE2",
        "CRPIX1",
        "CRPIX2",
        "CRVAL1",
        "CRVAL2",
        "CUNIT1",
        "CUNIT2",
        "CD1_1",
        "CD1_2",
        "CD2_1",
        "CD2_2",
        "PC1_1",
        "PC1_2",
        "PC2_1",
        "PC2_2",
        "CDELT1",
        "CDELT2",
        "CROTA2",
        "LONPOLE",
        "LATPOLE",
        "EQUINOX",
        "RADESYS",
    ];

    return orderedKeys
        .filter((key) => wcs[key] !== undefined && wcs[key] !== null)
        .map((key) => `${key.padEnd(8, " ")}= ${serializeFitsHeaderValue(wcs[key])}`)
        .join("\n");
}

function getCoordinateFrameFromWcs(wcs) {
    const ctype1 = `${wcs.CTYPE1 || ""}`.trim().toUpperCase();
    const ctype2 = `${wcs.CTYPE2 || ""}`.trim().toUpperCase();
    const radesys = `${wcs.RADESYS || ""}`.trim().toUpperCase();

    if (ctype1.startsWith("GLON-") && ctype2.startsWith("GLAT-")) {
        return "GAL";
    }

    if (ctype1.startsWith("RA---") && ctype2.startsWith("DEC--")) {
        return radesys || "ICRS";
    }

    return null;
}

function getProjectionFromWcs(wcs) {
    const ctype1 = `${wcs.CTYPE1 || ""}`.trim().toUpperCase();
    const projection = ctype1.slice(-3);

    return /^[A-Z0-9]{3}$/.test(projection) ? projection : null;
}

export function buildAvmFromWcs(wcs, options = {}) {
    if (!wcs || typeof wcs !== "object") {
        return null;
    }

    const coordinateFrame = getCoordinateFrameFromWcs(wcs);
    const projection = getProjectionFromWcs(wcs);

    if (!coordinateFrame || !projection) {
        return null;
    }

    const avm = {
        "MetadataVersion": "1.2",
        "Spatial.CoordinateFrame": coordinateFrame,
        "Spatial.ReferenceValue": [wcs.CRVAL1, wcs.CRVAL2],
        "Spatial.ReferenceDimension": [wcs.NAXIS1, wcs.NAXIS2],
        "Spatial.ReferencePixel": [wcs.CRPIX1, wcs.CRPIX2],
        "Spatial.CoordsystemProjection": projection,
        "Spatial.FITSheader": buildFitsHeaderFromWcs(wcs),
    };

    if (wcs.EQUINOX !== undefined && wcs.EQUINOX !== null) {
        avm["Spatial.Equinox"] = wcs.EQUINOX;
    }

    if (
        wcs.CD1_1 !== undefined &&
        wcs.CD1_2 !== undefined &&
        wcs.CD2_1 !== undefined &&
        wcs.CD2_2 !== undefined
    ) {
        avm["Spatial.CDMatrix"] = [wcs.CD1_1, wcs.CD1_2, wcs.CD2_1, wcs.CD2_2];
    } else if (wcs.CDELT1 !== undefined && wcs.CDELT2 !== undefined) {
        avm["Spatial.Scale"] = [wcs.CDELT1, wcs.CDELT2];

        const rotation = options.rotation ?? wcs.CROTA2;
        if (rotation !== undefined && rotation !== null) {
            avm["Spatial.Rotation"] = rotation;
        }
    }

    return avm;
}

export function buildAvmXmpPacket(avm) {
    const entries = Object.entries(avm)
        .map(([key, value]) => `    <avm:${key}>${renderAvmValue(value)}</avm:${key}>`)
        .join("\n");

    return [
        `<?xpacket begin="${"\uFEFF"}" id="W5M0MpCehiHzreSzNTczkc9d"?>`,
        '<x:xmpmeta xmlns:x="adobe:ns:meta/">',
        ' <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">',
        '  <rdf:Description rdf:about="" xmlns:avm="http://www.communicatingastronomy.org/avm/1.0/">',
        entries,
        '  </rdf:Description>',
        ' </rdf:RDF>',
        '</x:xmpmeta>',
        '<?xpacket end="w"?>',
    ].join("\n");
}

function makePngChunk(type4, data) {
    const encoder = new TextEncoder();
    const type = encoder.encode(type4);

    if (type.length !== 4) {
        throw new Error("PNG chunk type must be 4 bytes");
    }

    const crcInput = concatUint8Arrays([type, data]);
    const crc = crc32(crcInput);

    return concatUint8Arrays([
        u32be(data.length),
        type,
        data,
        u32be(crc),
    ]);
}

function makeXmpITXtChunk(xmpString) {
    const encoder = new TextEncoder();

    const keyword = encoder.encode("XML:com.adobe.xmp");
    const compressionFlag = new Uint8Array([0]);
    const compressionMethod = new Uint8Array([0]);
    const languageTag = new Uint8Array([0]);
    const translatedKeyword = new Uint8Array([0]);
    const text = encoder.encode(xmpString);

    const data = concatUint8Arrays([
        keyword,
        new Uint8Array([0]),
        compressionFlag,
        compressionMethod,
        languageTag,
        translatedKeyword,
        text,
    ]);

    return makePngChunk("iTXt", data);
}

export function insertXmpIntoPng(pngBytes, xmpString) {
    if (!(pngBytes instanceof Uint8Array)) {
        throw new Error("pngBytes must be a Uint8Array");
    }

    const pngSignature = new Uint8Array([
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
    ]);

    for (let i = 0; i < 8; i++) {
        if (pngBytes[i] !== pngSignature[i]) {
            throw new Error("Not a valid PNG");
        }
    }

    const xmpChunk = makeXmpITXtChunk(xmpString);
    let pos = 8;
    let insertPos = -1;

    while (pos + 12 <= pngBytes.length) {
        const length =
            (pngBytes[pos] << 24) |
            (pngBytes[pos + 1] << 16) |
            (pngBytes[pos + 2] << 8) |
            pngBytes[pos + 3];

        const type = String.fromCharCode(
            pngBytes[pos + 4],
            pngBytes[pos + 5],
            pngBytes[pos + 6],
            pngBytes[pos + 7],
        );

        const chunkEnd = pos + 12 + length;
        if (chunkEnd > pngBytes.length) {
            throw new Error("Corrupt PNG");
        }

        if (type === "IDAT" || type === "IEND") {
            insertPos = pos;
            break;
        }

        pos = chunkEnd;
    }

    if (insertPos === -1) {
        throw new Error("Could not find insertion point in PNG");
    }

    return concatUint8Arrays([
        pngBytes.subarray(0, insertPos),
        xmpChunk,
        pngBytes.subarray(insertPos),
    ]);
}

export async function injectAvmIntoPngBlob(blob, avm) {
    const pngBytes = new Uint8Array(await blob.arrayBuffer());
    const xmp = buildAvmXmpPacket(avm);
    const outBytes = insertXmpIntoPng(pngBytes, xmp);

    return new Blob([outBytes], { type: "image/png" });
}

export function blobToDataURL(blob) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onloadend = () => resolve(reader.result);
        reader.onerror = () => reject(new Error("Error reading blob as data URL"));
        reader.readAsDataURL(blob);
    });
}

export function blobToArrayBuffer(blob) {
    return blob.arrayBuffer();
}
