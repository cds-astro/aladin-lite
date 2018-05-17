var a1 = A.aladin('#al1', {fov: 0.4, cooFrame: 'equatorial', realFullscreen: true, target: '13 29 52.7 +47 11 42.93', survey: 'P/SDSS9/color'});
a1.setFovRange(0.01, 175);
//a1.gotoRaDec(297.87, 25.96);
var a2 = A.aladin('#al2', {fov: 0.4, cooFrame: 'equatorial', realFullscreen: true, target: '13 29 52.7 +47 11 42.93', survey: 'P/PanSTARRS/DR1/color-z-zg-g', showFrameControl: false, showFullscreenControl: false, showGotoControl: false});
a2.setFovRange(0.01, 175);

View.CALLBACKS_THROTTLE_TIME = 30;
a1.on('positionChanged', function(params) {
    a2.gotoRaDec(params.ra, params.dec);
});
a2.on('positionChanged', function(params) {
    a1.gotoRaDec(params.ra, params.dec);
});
a1.on('zoomChanged', function(fov) {
if (Math.abs(a2.getFov()[0] - fov) / fov > 0.01) {
        a2.setFoV(fov);
    }
});
a2.on('zoomChanged', function(fov) {
    if (Math.abs(a1.getFov()[0] - fov) / fov > 0.01) {
        a1.setFoV(fov);
    }
});
setTimeout(function() {
    $(".twentytwenty-container").twentytwenty({default_offset_pct: 0.5, no_overlay: true});
}, 300);


