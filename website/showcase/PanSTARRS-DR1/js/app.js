$('#layersControl').show();

var curSelectedSource = null;
var aladin = A.aladin('#aladin', {fov: 45, cooFrame: 'galactic'});
aladin.gotoRaDec(297.87, 25.96);

aladin.createImageSurvey('panstarrs-color', 'PanSTARRS DR1 color', 'http://alasky.u-strasbg.fr/Pan-STARRS/DR1/color-z-zg-g', 'equatorial', 11);
aladin.createImageSurvey('panstarrs-g', 'PanSTARRS DR1 g-band', 'http://alasky.u-strasbg.fr/Pan-STARRS/DR1/g', 'equatorial', 11);
aladin.createImageSurvey('panstarrs-density-map', 'PanSTARRS DR1 density map', 'http://alasky.u-strasbg.fr/footprints/hips-density-maps/II/349/ps1', 'equatorial', 4, {imgFormat: 'jpg'});

aladin.setImageSurvey('panstarrs-color');


var hipsCats = {
  'ps1': A.catalogHiPS('http://axel.u-strasbg.fr/HiPSCatService/II/349/ps1', {name: 'PanSTARRS DR1 sources', shape: 'circle', sourceSize: 8, color: '#6baed6'}),
  'simbad': A.catalogHiPS('http://axel.u-strasbg.fr/HiPSCatService/Simbad', {name: 'Simbad', color: '#ce6dbd'})
};

hipsCats['simbad'].hide();

aladin.addCatalog(hipsCats['simbad']);
aladin.addCatalog(hipsCats['ps1']);


// listen changes on HiPS image background selection
$('input[type=radio][name=img-hips]').change(function() {
  aladin.setImageSurvey(this.value);
});


// listen changes on HiPS catalogues selection
$('#overlay-form :checkbox').change(function() {
  var cat = hipsCats[this.value];

  if (this.checked) {
    cat.show();
  }
  else {
    cat.hide();
  }
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
    if (source.catalog.name !== 'Simbad') {
        html += '<h3>PanSTARRS source <em>' + source.data.objID + '</em></h3>';
        html += '<tr class="pure-table-odd"><td><b>RAJ2000</b></td><td>' + source.data.RAJ2000 + '</td><td><em>deg</em></td></tr>';
        html += '<tr><td><b>DEJ2000</b></td><td>' + source.data.DEJ2000 + '</td><td><em>deg</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>gmag</b></td><td>' + source.data['gmag'] + '</td><td><em>mag</em></td></tr>';
        html += '<tr><td><b>rmag</b></td><td>' + source.data['rmag'] + '</td><td><em>mag</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>imag</b></td><td>' + source.data.imag + '</td><td><em>mag</em></td></tr>';
        html += '<tr><td><b>zmag</b></td><td>' + source.data.zmag + '</td><td><em>mag</em></td></tr>';
        html += '<tr class="pure-table-odd"><td><b>ymag</b></td><td>' + source.data.ymag + '</td><td><em>mag</em></td></tr>';
        html += '</tbody>';
        html += '</table>';

        html += '<br/><a target="_blank" href="http://vizier.u-strasbg.fr/viz-bin/VizieR-5?-out.form=%2bH&-source=II/349/ps1' + '&-c=' + encodeURIComponent(source.ra + ',' + source.dec) + '&-c.rs=0.02&objID=' + source.data.objID + '">More details</a>';
    }
    else {
        html += '<h3>Simbad object <em>' + source.data.main_id + '</em></h3>';
        html += '<tr class="pure-table-odd"><td><b>ra</b></td><td>' + source.data.ra + '</td><td><em>deg</em></td></tr>';
        html += '<tr><td><b>dec</b></td><td>' + source.data.dec + '</td><td><em>deg</em></td></tr>';
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

