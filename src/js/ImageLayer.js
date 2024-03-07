export let ImageLayer = {};

ImageLayer.update = function (layer) {
    const foundLayer = ImageLayer.contains(layer.id)

    const options = layer.metadata;
    // The survey has not been found among the ones cached
    if (foundLayer) {
        foundLayer.options = options;
    } else {
        ImageLayer.LAYERS.push({
            id: layer.id,
            name: layer.name,
            url: layer.url,
            options,
            subtype: layer.subtype,
        });
    }
}

ImageLayer.contains = function(id) {
    return ImageLayer.LAYERS.find((layer) => layer.id.endsWith(id));
}

ImageLayer.DEFAULT_SURVEY = {
    id: "P/DSS2/color",
    name: "DSS colored",
    url: "https://alasky.cds.unistra.fr/DSS/DSSColor",
    maxOrder: 9,
    subtype: "survey",
    tileSize: 512,
    formats: ['jpeg'],
    creatorDid: "ivo://CDS/P/DSS2/color",
    dataproductSubtype: ['color'],
    frame: "ICRS"
}

ImageLayer.LAYERS = [
    ImageLayer.DEFAULT_SURVEY,
    {
        id: "P/2MASS/color",
        name: "2MASS colored",
        url: "https://alasky.cds.unistra.fr/2MASS/Color",
        maxOrder: 9,
        subtype: "survey",
    },
    {
        id: "P/DSS2/red",
        name: "DSS2 Red (F+R)",
        url: "https://alasky.cds.unistra.fr/DSS/DSS2Merged",
        maxOrder: 9,
        subtype: "survey",
        // options
        options: {
            minCut: 1000.0,
            maxCut: 10000.0,
            colormap: "magma",
            stretch: 'Linear',
            imgFormat: "fits"
        }
    },
    {
        id: "P/DM/I/350/gaiaedr3",
        name: "Density map for Gaia EDR3 (I/350/gaiaedr3)",
        url: "https://alasky.cds.unistra.fr/ancillary/GaiaEDR3/density-map",
        maxOrder: 7,
        subtype: "survey",
        // options
        options: {
            minCut: 0,
            maxCut: 12000,
            stretch: 'asinh',
            colormap: "rdylbu",
            imgFormat: "fits",
        }
    },
    {
        id: "P/PanSTARRS/DR1/g",
        name: "PanSTARRS DR1 g",
        url: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/g",
        maxOrder: 11,
        subtype: "survey",
        // options
        options: {
            minCut: -34,
            maxCut: 7000,
            stretch: 'asinh',
            colormap: "redtemperature",
            imgFormat: "fits",
        }
    },
    {
        id: "P/PanSTARRS/DR1/color-z-zg-g",
        name: "PanSTARRS DR1 color",
        url: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/color-z-zg-g",
        maxOrder: 11,
        subtype: "survey",
    },
    {
        id: "P/DECaPS/DR1/color",
        name: "DECaPS DR1 color",
        url: "https://alasky.cds.unistra.fr/DECaPS/DR1/color",
        maxOrder: 11,
        subtype: "survey",
    },
    {
        id: "P/Fermi/color",
        name: "Fermi color",
        url: "https://alasky.cds.unistra.fr/Fermi/Color",
        maxOrder: 3,
        subtype: "survey",
    },
    {
        id: "P/Finkbeiner",
        name: "Halpha",
        url: "https://alasky.cds.unistra.fr/FinkbeinerHalpha",
        maxOrder: 3,
        subtype: "survey",
        // options
        options: {
            minCut: -10,
            maxCut: 800,
            colormap: "rdbu",
            imgFormat: "fits",
        }
    },
    {
        id: "P/GALEXGR6_7/NUV",
        name: "GALEXGR6_7 NUV",
        url: "http://alasky.cds.unistra.fr/GALEX/GALEXGR6_7_NUV/",
        maxOrder: 8,
        subtype: "survey",
    },
    {
        id: "P/IRIS/color",
        name: "IRIS colored",
        url: "https://alasky.cds.unistra.fr/IRISColor",
        maxOrder: 3,
        subtype: "survey",
    },
    {
        id: "P/Mellinger/color",
        name: "Mellinger colored",
        url: "https://alasky.cds.unistra.fr/MellingerRGB",
        maxOrder: 4,
        subtype: "survey",
    },
    {
        id: "P/SDSS9/color",
        name: "SDSS9 colored",
        url: "https://alasky.cds.unistra.fr/SDSS/DR9/color",
        maxOrder: 10,
        subtype: "survey",
    },
    {
        id: "P/SDSS9/g",
        name: "SDSS9 band-g",
        url: "https://alasky.cds.unistra.fr/SDSS/DR9/band-g",
        maxOrder: 10,
        subtype: "survey",
        options: {
            stretch: 'asinh',
            colormap: "redtemperature",
            imgFormat: 'fits'
        }
    },
    {
        id: "P/SPITZER/color",
        name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
        url: "http://alasky.cds.unistra.fr/Spitzer/SpitzerI1I2I4color/",
        maxOrder: 9,
        subtype: "survey",
    },
    {
        id: "P/VTSS/Ha",
        name: "VTSS-Ha",
        url: "https://alasky.cds.unistra.fr/VTSS/Ha",
        maxOrder: 3,
        subtype: "survey",
        options: {
            minCut: -10.0,
            maxCut: 100.0,
            colormap: "grayscale",
            imgFormat: "fits"
        }
    },
    {
        id: "xcatdb/P/XMM/PN/color",
        name: "XMM PN colored",
        url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=https://saada.unistra.fr/PNColor",
        maxOrder: 7,
        subtype: "survey",
    },
    {
        id: "P/allWISE/color",
        name: "AllWISE color",
        url: "https://alasky.cds.unistra.fr/AllWISE/RGB-W4-W2-W1/",
        maxOrder: 8,
        subtype: "survey",
    },
    {
        id: "P/GLIMPSE360",
        name: "GLIMPSE360",
        // This domain is not giving the CORS headers
        // We need to query by with a proxy equipped with CORS header.
        url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=https://www.spitzer.caltech.edu/glimpse360/aladin/data",
        subtype: "survey",
        options: {
            maxOrder: 9,
            imgFormat: "jpg",
            minOrder: 3,
        }
    },
];

ImageLayer.getAvailableLayers = function () {
    return ImageLayer.LAYERS;
};
