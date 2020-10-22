    let aladin;
    A.init.then(() => {
        // Start up Aladin Lite
        aladin = A.aladin('#aladin-lite-div', {showSimbadPointerControl: true, realFullscreen: true, fov: 100, allowFullZoomout: true, showReticle: false, survey: 'CDS/P/2MASS/color'});
        //aladin.setImageSurvey('CDS/P/Coronelli');



aladin.createImageSurvey('Coronelli', 'Coronelli', 'http://alasky.u-strasbg.fr/CDS_P_Coronelli', 'equatorial', 4, {imgFormat: 'jpg', longitudeReversed: true, minOrder: 3});
aladin.createImageSurvey('illenoroC', 'illenoroC', 'http://alasky.u-strasbg.fr/CDS_P_Coronelli', 'equatorial', 4, {imgFormat: 'jpg', longitudeReversed: false, minOrder: 3});
//aladin.setImageSurvey('Coronelli');

$('#layersControlLeft').show();
$('#layersCL2').show();
$('#layersControlRight').show();

var hipsCats = {
  'constellations-boundaries': A.catalogFromVizieR('VI/49/bound_20', '0 +0', 180, {color: 'red'}),
  'simbad': A.catalogHiPS('http://axel.u-strasbg.fr/HiPSCatService/Simbad', {name: 'Simbad', color: '#6dbdce'}),
  'gaia': A.catalogHiPS('http://axel.u-strasbg.fr/HiPSCatService/I/345/gaia2', {name: 'Gaia DR2', color: '#6666cc', shape: 'circle', sourceSize: 6})
};
hipsCats['simbad'].hide();
hipsCats['constellations-boundaries'].hide();
hipsCats['gaia'].hide();
aladin.addCatalog(hipsCats['simbad']);
aladin.addCatalog(hipsCats['constellations-boundaries']);
aladin.addCatalog(hipsCats['gaia']);


var coronelliStars = {
  'coronelli-stars-white': A.catalogFromURL("http://cdsweb.u-strasbg.fr/~derriere/coronelli/white.xml", {name: 'Coronelli white', color: '#ffffff', shape: 'rhomb', sourceSize: 10}),
  'coronelli-stars-yellow': A.catalogFromURL("http://cdsweb.u-strasbg.fr/~derriere/coronelli/yellow.xml", {name: 'Coronelli yellow', color: '#f6f874', shape: 'rhomb', sourceSize: 10}),
  'coronelli-stars-red': A.catalogFromURL("http://cdsweb.u-strasbg.fr/~derriere/coronelli/red.xml", {name: 'Coronelli red', color: '#ff5555', shape: 'rhomb', sourceSize: 10}),
  'coronelli-stars-blue': A.catalogFromURL("http://cdsweb.u-strasbg.fr/~derriere/coronelli/blue.xml", {name: 'Coronelli blue', color: '#1ca5ec', shape: 'rhomb', sourceSize: 10})
};
coronelliStars['coronelli-stars-white'].hide();
coronelliStars['coronelli-stars-yellow'].hide();
coronelliStars['coronelli-stars-red'].hide();
coronelliStars['coronelli-stars-blue'].hide();
aladin.addCatalog(coronelliStars['coronelli-stars-white']);
aladin.addCatalog(coronelliStars['coronelli-stars-yellow']);
aladin.addCatalog(coronelliStars['coronelli-stars-red']);
aladin.addCatalog(coronelliStars['coronelli-stars-blue']);

// ajout de nouveaux relevés custom ?
// Mellinger : P/Mellinger/color
// PanSTARRS couleur : CDS/P/PanSTARRS/DR1/color-i-r-g
// gaia flux : CDS/P/DM/flux-color-Rp-G-Bp/I/345/gaia2
aladin.createImageSurvey("dss2", "DSS color", "http://alasky.u-strasbg.fr/DSS/DSSColor/", "equatorial", 9, {imgFormat: 'jpg'});
aladin.createImageSurvey("decaps", "DECaPS DR1", "http://alasky.u-strasbg.fr/DECaPS/DR1/color/", "equatorial", 11, {imgFormat: 'png'});
aladin.createImageSurvey("gaiamap", "Gaia Flux", "http://alasky.u-strasbg.fr/ancillary/GaiaDR2/color-Rp-G-Bp-flux-map/", "equatorial", 4, {imgFormat: 'jpg'});
aladin.createImageSurvey("panstarrs", "PanSTARRS", "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/color-i-r-g/", "equatorial", 11, {imgFormat: 'jpg'});
aladin.createImageSurvey('const-outlines', 'Constellation outlines', 'http://alaskybis.u-strasbg.fr/JAXA/JAXA_P_CONSTELLATIONS5/', 'equatorial', 6, {imgFormat: 'png'});
aladin.createImageSurvey('const-jaxa', 'Constellation by JAXA', 'http://alaskybis.u-strasbg.fr/JAXA/JAXA_P_CONSTELLATIONS6/', 'equatorial', 6, {imgFormat: 'png'});



var curSelectedSource = null;
aladin.setOverlayImageLayer('P/Mellinger/color');

// listen changes on HiPS image background selection
$('.img-hips').click(function() {
  if (!$(this).hasClass("selected")) {
    $('.img-hips').removeClass("selected pure-button-active");
    $(this).addClass("selected pure-button-active");
    aladin.setOverlayImageLayer(this.id);
    aladin.getOverlayImageLayer().setAlpha(0.5);
    $('#opacity-slider').val(0.5);
  }
  else {
    $(this).removeClass("selected pure-button-active");
    // possibilité ?     aladin.setOverlayImageLayer(null);
    aladin.getOverlayImageLayer().setAlpha(0);
    $('#opacity-slider').val(0);
  }
});


$('#opacity-slider').on('input', function() {
    aladin.getOverlayImageLayer().setAlpha($(this).val());
});


// listen changes on HiPS catalogues selection
$('.catlayer').click(function() {
  var cat = hipsCats[$(this).attr('id')];

  if (!$(this).hasClass("selected")) {
    $(this).addClass("selected pure-button-active");
    cat.show();
  }
  else {
    $(this).removeClass("selected pure-button-active");
    cat.hide();
  }
});

// listen changes on Coronelli catalogues selection
$('.catcoro').click(function() {
  var cat = coronelliStars[$(this).attr('id')];

  if (!$(this).hasClass("selected")) {
    $(this).addClass("selected");
    cat.show();
  }
  else {
    $(this).removeClass("selected");
    cat.hide();
  }
});

var cooNav = {
  'coo_epoca': {ra: 4.0, dec: -30.0, time: 10},
  'coo_legende': {ra: 33.0, dec: -32.0, time: 10},
  'coo_orion': {ra: 85.2, dec: -2.5, time: 10},
  'coo_magellan': {ra: 45.0, dec: -79.0, time: 10},
  'coo_halley': {ra: 219.6, dec: 7.0, time: 10}
};

// listen click on navigation buttons
$('.nav-button').click(function() {
  var cooTarget = $(this).parent().attr('id');

  if ($(this).hasClass("nav-goto")) {
    aladin.gotoRaDec(cooNav[cooTarget].ra, cooNav[cooTarget].dec);
  }
  else if ($(this).hasClass("nav-flyto")) {
    aladin.animateToRaDec(cooNav[cooTarget].ra, cooNav[cooTarget].dec, cooNav[cooTarget].time);
  }
});

// stop animations
$('#stop').click(function() {
	aladin.stopAnimation();
});

// listen to click on objects
aladin.on('objectClicked', function(source) {
    var html = '<table class="pure-table">';

    if (curSelectedSource != null) {
        curSelectedSource.deselect();
    }
    if (source==null) {
        $('#explain').html('');
        $('#explain').hide();
        return;
    }

    source.select();
    curSelectedSource = source;
    html += '<tbody>';
    if (source.catalog.name == 'Simbad') {
        console.log(source.data);
        html += '<h3>Simbad object <em>' + source.data.main_id + '</em></h3>';
        html += '<tr class="pure-table-odd"><td><b>ra</b></td><td>' + source.data.ra + '</td><td><em>deg</em></td></tr>';
        html += '<tr><td><b>dec</b></td><td>' + source.data.dec + '</td><td><em>deg</em></td></tr>';
        html += '<tr><td><b>main_type</b></td><td>' + source.data.main_type + '</td><td><em>deg</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>pmra</b></td><td>' + source.data.pmra + '</td><td><em>mas/yr</em></td></tr>';
        html += '<tr><td><b>pmdec</b></td><td>' + source.data.pmdec + '</td><td><em>mas/yr</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>parallax</b></td><td>' + source.data.plx + '</td><td><em>mas</em></td></tr>';
        html += '<tr><td><b>B mag.</b></td><td>' + source.data.B + '</td><td><em>mag</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>V mag.</b></td><td>' + source.data.V + '</td><td><em>mag</em></td></tr>';
        html += '</tbody>';
        html += '</table>';

        html += '<br/><a target="_blank" href="http://simbad.u-strasbg.fr/simbad/sim-id?Ident=' + encodeURIComponent(source.data.main_id) + '">More details</a>';
    }


    $('#explain').html(html);
    $('#explain').show();

});

aladin.on('fullScreenToggled', function(fullScreenFlag) {
    if (fullScreenFlag) {
        $('#calibCircle').show();
	//temporaire gestion cercle
	document.getElementById("circle-checkbox").checked = true;
    }
    else {
        $('#calibCircle').hide();
	//temporaire gestion cercle
	document.getElementById("circle-checkbox").checked = false;
    }
});

//temporaire gestion cercle
document.getElementById("circle-checkbox").addEventListener('change', (event) => {
  if (event.target.checked) {
        $('#calibCircle').show();
  } else {
        $('#calibCircle').hide();
  }
})

/*
document.addEventListener('touchmove', function (event) {
  if (event.scale !== 1) { event.preventDefault(); }
}, false);

var lastTouchEnd = 0;
document.addEventListener('touchend', function (event) {
  var now = (new Date()).getTime();
  if (now - lastTouchEnd <= 300) {
    event.preventDefault();
  }
  lastTouchEnd = now;
}, false);
*/

/*
document.addEventListener("touchstart", event => {
    if(event.touches.length > 1) {
        console.log("zoom plz stahp");
        event.preventDefault();
        //event.stopPropagation(); // maybe useless
    }
}, {passive: false});
*/


    }
        );
