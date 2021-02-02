export let DiscoveryTree = (function () {
    // Constructor
    var DiscoveryTree = function (aladin) {
        // activate Vue on the <div> that contains the component
        new Vue({
            el: '#ui',
            methods: {
                // Define the methods for the discovery-tree component
                // to interact with the aladin viewer
                getFovCorners() {
                    return aladin.getFovCorners();
                },
                getCenter() {
                    return aladin.getRaDec();
                },
                // Called when the user add a image survey
                addImage(metadata) {
                    const order = (+metadata.hips_order);
                    const hipsTileFormat = metadata.hips_tile_format.split(' ');
            
                    let tileFormat;
                    let color;
                    if (hipsTileFormat.indexOf('fits') >= 0) {
                        tileFormat = {
                            FITSImage: {
                                bitpix: parseInt(metadata.hips_pixel_bitpix)
                            }
                        };
                        color = {
                            Grayscale2Color: {
                                color: [1.0, 1.0, 1.0],
                                k: 1.0,
                                transfer: "asinh"
                            }
                        };
                    } else {
                        color = "Color";

                        if (hipsTileFormat.indexOf('png') >= 0) {
                            tileFormat = {
                                Image: {
                                    format: "png"
                                }
                            };
                        } else {
                            tileFormat = {
                                Image: {
                                    format: "jpeg"
                                }
                            };
                        }
                    }

                    let cuts = [undefined, undefined];
                    if (metadata.hips_pixel_cut) {
                        cuts = metadata.hips_pixel_cut.split(" ");
                    }
                    let tileSize = 512;
                    // Verify the validity of the tile width
                    if (metadata.hips_tile_width) {
                        let hipsTileWidth = parseInt(metadata.hips_tile_width);
                        let isPowerOfTwo = hipsTileWidth && !(hipsTileWidth & (hipsTileWidth - 1));

                        if (isPowerOfTwo === true) {
                            tileSize = hipsTileWidth;
                        }
                    }
                    let url = metadata.hips_service_url;
                    if (url.startsWith('http://alasky')) {
                        // From alasky one can directly use the https access
                        url = url.replace('http', 'https');
                    } else {
                        // Pass by a proxy for extern http urls
                        url = 'https://alasky.u-strasbg.fr/cgi/JSONProxy?url=' + url;
                    }
                    let survey = {
                        properties: {
                            url: url,
                            maxOrder:  parseInt(metadata.hips_order),
                            frame: {
                                label: "J2000",
                                system: "J2000"
                            },
                            tileSize: tileSize,
                            format: tileFormat,
                            minCutout: parseFloat(cuts[0]),
                            maxCutout: parseFloat(cuts[1]),
                        },
                        color: color
                    };

                    aladin.webglAPI.setHiPS([survey]);
                },
                // Called when the user add a catalog survey
                addCatalog(metadata, center, radius) {
                    if (metadata.hips_service_url) {
                        const hips = A.catalogHiPS(metadata.hips_service_url, {
                            onClick: 'showTable',
                            name: metadata.ID,
                        });
                        aladin.addCatalog(hips);
                    } else {
                        console.log(metadata.obs_id, "center, ", center, " radius, ", radius)
                        const catalog = A.catalogFromVizieR(
                            metadata.obs_id,
                            {
                                ra: center[0],
                                dec: center[1]
                            },
                            radius, {
                                onClick: 'showTable',
                                limit: 1000,
                            }
                        );
                        aladin.addCatalog(catalog);
                    }
                },
                // Called when the user add a HEALPix coverage
                addCoverage(metadata) {
                    const moc = A.MOCFromURL(metadata.moc_access_url);
                    aladin.addMOC(moc);
                },
            },
        });
    }

    return DiscoveryTree;
})();
