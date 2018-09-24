function formatDate(date) {
    var d = new Date(date),
        month = '' + (d.getMonth() + 1),
        day = '' + d.getUTCDate(),
        year = d.getFullYear();

        hour = d.getUTCHours();
        min = d.getMinutes();

        if (month.length < 2) month = '0' + month;
        if (day.length < 2) day = '0' + day;
        if (hour < 10) hour = '0' + hour;
        if (min < 10) min = '0' + min;

    return [year, month, day].join('-') + ' ' + [hour, min].join(':');
}

function jdToDate(jd) {
    var millis = (jd - 2440587.5) * 86400000;
    return new Date(millis);
}

        function enforceBounds(x) {
            if (x < 0) {
                return 0;
            } else if (x > 1){
                return 1;
            } else {
                return x;
            }
        }


        function interpolateLinearly(x, values) {

            // Split values into four lists
            var x_values = [];
            var r_values = [];
            var g_values = [];
            var b_values = [];
            for (i in values) {
                x_values.push(values[i][0]);
                r_values.push(values[i][1][0]);
                g_values.push(values[i][1][1]);
                b_values.push(values[i][1][2]);
            }

            var i = 1;
            while (x_values[i] < x) {
                i = i+1;
            }
            i = i-1;

            var width = Math.abs(x_values[i] - x_values[i+1]);
            var scaling_factor = (x - x_values[i]) / width;

            // Get the new color values though interpolation
            var r = r_values[i] + scaling_factor * (r_values[i+1] - r_values[i])
            var g = g_values[i] + scaling_factor * (g_values[i+1] - g_values[i])
            var b = b_values[i] + scaling_factor * (b_values[i+1] - b_values[i])

            return [enforceBounds(r), enforceBounds(g), enforceBounds(b)];

        }

var shapesCache = {};
var getShape = function(diam, r, g, b) {
    var key = diam + '-' + r + '-' + g + '-' + b;

    if ( shapesCache[key] === undefined ) {
        var c = document.createElement('canvas');
        c.width = c.height = diam;
        var ctx = c.getContext('2d');
        ctx.beginPath();
        var color = 'rgb(' + r + ',' + g + ',' + b + ')';
        ctx.fillStyle = color;
        ctx.fillStyle = color;
        ctx.arc(diam/2., diam/2., diam/2., 0, 2*Math.PI, true);
        ctx.fill();

        shapesCache[key] = c;
    }

    return shapesCache[key];
};

var timeStart = 2456288.5;
var timeEnd = timeStart + 1;
var curTimeJD = timeStart;
var timeStep = 1 / 24. / 60.;
var nbSteps = 1440;
var drawFunction = function(source, canvasCtx, viewParams) {

    var period = parseFloat(source.data['koi_period']);
    var durationHours = parseFloat(source.data['koi_duration']);
    var timeTransit = 2454833.0 + parseFloat(source.data['koi_time0bk']);

    var timeClosestTransit = timeTransit + period * Math.round((curTimeJD-timeTransit) / period);

    // filter out sources not in transit right now
    var inTransit =  (timeClosestTransit - 0.5 * durationHours / 24.) <= curTimeJD && (timeClosestTransit + 0.5 * durationHours / 24.) >= curTimeJD;
    if ( ! inTransit) {
        return;
    }

    var radiusMin = 0.2;
    var radiusMax = 1550;

    var radiusMinPixels = 4;
    var radiusMaxPixels = 40;

    var radius = parseFloat(source.data['koi_prad']);
    var diam = radiusMinPixels + (radiusMaxPixels - radiusMinPixels) *  (Math.log(radius + 1) - Math.log(radiusMin + 1)) / (Math.log(radiusMax+1) - Math.log(radiusMin+1));
    if (isNaN(diam)) {
        return;
    }

    var tempMin = 250.;
    var tempMax = 1250.;

    var temp = parseFloat(source.data['koi_teq']);
    if (isNaN(temp)) {
        return;
    }
    temp = Math.max(temp, tempMin);
    temp = Math.min(temp, tempMax);

    var colorArray = interpolateLinearly( 1 - (temp - tempMin) / (tempMax - tempMin), RdBu);
    var r = Math.round(255*colorArray[0]);
    var g = Math.round(255*colorArray[1]);
    var b = Math.round(255*colorArray[2]);

    canvasCtx.drawImage(getShape(diam, r, g, b), source.x-diam/2., source.y-diam/2.);
};


    var aladin = A.aladin('#aladin-lite-div', {target: '19 21 19.937 +44 29 40.67', fov: 34, survey: 'P/DSS2/red'});
    aladin.on('fullScreenToggled', function(fullScreen) {
        if (fullScreen===true) {
            setTimeout(function() {$('#timeControl').css('position', 'absolute').css('bottom', '0').css('left', '40px');}, 500);
        }
        else {
            $('#timeControl').css('position', 'relative');
        }
    });
    aladin.setFrame('galactic');

    // add Kepler footprint
    var overlay = A.graphicOverlay({color: 'white', lineWidth: 1, name: 'Kepler Fov'});
    aladin.addOverlay(overlay);
    var stcs = [
        'Polygon ICRS 298.946381 43.260571 298.178741 44.327797 299.576324 44.839333 300.324982 43.762146 298.946381 43.260571',
        'Polygon ICRS 297.375397 45.400730 298.170837 44.338558 299.568634 44.850193 298.79125999999997 45.922272 297.375397 45.400730',
        'Polygon ICRS 300.362762 46.468735 301.11731 45.386719 299.692474 44.894371 298.917358 45.967499 300.362762 46.468735',
        'Polygon ICRS 301.849945 44.288685 301.124786 45.375759 299.700134 44.883499 300.446442 43.805271 301.849945 44.288685',
        'Polygon ICRS 297.089661 45.782001 296.253845 46.836796 297.699371 47.369987 298.514832 46.304077 297.089661 45.782001',
        'Polygon ICRS 295.376556 47.896175 296.24523899999997 46.847427 297.69097899999997 47.380733 296.841675 48.440578 295.376556 47.896175',
        'Polygon ICRS 298.465637 49.010300 299.291382 47.939480 297.814087 47.426117 296.966156 48.486870 298.465637 49.010300',
        'Polygon ICRS 300.090546 46.851803 299.29953 47.928627 297.822479 47.415363 298.636444 46.348537 300.090546 46.851803',
        'Polygon ICRS 295.053558 48.267601 294.135468 49.307076 295.63122599999997 49.866013 296.527832 48.814175 295.053558 48.267601',
        'Polygon ICRS 293.168884 50.349663 294.126007 49.317547 295.621979 49.876610 294.685211 50.921169 293.168884 50.349663',
        'Polygon ICRS 296.376617 51.518166 297.28662099999997 50.460880 295.752777 49.923992 294.81915300000003 50.969917 296.376617 51.518166',
        'Polygon ICRS 298.164246 49.385677 297.295593 50.450157 295.762024 49.913383 296.65542600000003 48.860195 298.164246 49.385677',
        'Polygon ICRS 295.80523700000003 41.456608 297.214783 42.039734 297.92337 41.054253 296.527527 40.480042 295.80523700000003 41.456608',
        'Polygon ICRS 298.665131 42.610325 297.229156 42.045536 297.937592 41.059971 299.35791 41.616848 298.665131 42.610325',
        'Polygon ICRS 300.087067 40.532223 298.684387 39.984165 297.99881 40.974442 299.41796899999997 41.530514 300.087067 40.532223',
        'Polygon ICRS 297.290131 39.414249 298.670319 39.978542 297.984589 40.968735 296.589752 40.395409 297.290131 39.414249',
        'Polygon ICRS 295.5448 41.898685 294.731903 42.940697 296.065216 43.495872 296.86261 42.443825 295.5448 41.898685',
        'Polygon ICRS 293.88320899999997 43.987526 294.723572 42.951199 296.057037 43.506477 295.231171 44.552746 293.88320899999997 43.987526',
        'Polygon ICRS 296.725037 45.147991 297.531494 44.091293 296.170715 43.554081 295.347534 44.601768 296.725037 45.147991',
        'Polygon ICRS 298.316528 43.018085 297.53949 44.080585 296.178894 43.543465 296.973572 42.490009 298.316528 43.018085',
        'Polygon ICRS 293.57370000000003 44.367344 292.69277999999997 45.395187 294.064728 45.972885 294.929413 44.933964 293.57370000000003 44.367344',
        'Polygon ICRS 291.770569 46.426575 292.683716 45.405540 294.055817 45.983353 293.157837 47.015450 291.770569 46.426575',
        'Polygon ICRS 294.699921 47.635029 295.57800299999997 46.591599 294.17449999999997 46.032543 293.278748 47.065845 294.699921 47.635029',
        'Polygon ICRS 296.430298 45.530739 295.58667 46.581017 294.18338 46.022064 295.04580699999997 44.981949 296.430298 45.530739',
        'Polygon ICRS 294.391724 48.068005 292.854858 47.452583 291.978241 48.405174 293.534698 49.032036 294.391724 48.068005',
        'Polygon ICRS 291.338654 46.811111 292.839508 47.446262 291.962708 48.398735 290.44516 47.751217 291.338654 46.811111',
        'Polygon ICRS 289.43753100000004 48.762104 290.970551 49.424072 291.885132 48.479771 290.367249 47.830158 289.43753100000004 48.762104',
        'Polygon ICRS 292.561554 50.070976 290.986267 49.430653 291.900665 48.486233 293.457733 49.115135 292.561554 50.070976',
        'Polygon ICRS 292.128174 50.508759 290.542572 49.864094 289.577972 50.798695 291.183502 51.456238 292.128174 50.508759',
        'Polygon ICRS 288.98349 49.192036 290.526764 49.857471 289.561981 50.791939 288.002167 50.112785 288.98349 49.192036',
        'Polygon ICRS 286.893097 51.101574 288.469055 51.795586 289.476379 50.870117 287.915253 50.189835 286.893097 51.101574',
        'Polygon ICRS 290.110687 52.474018 288.485229 51.802486 289.492401 50.876884 291.099518 51.535488 290.110687 52.474018',
        'Polygon ICRS 292.572998 40.006577 293.921509 40.630119 294.665466 39.666309 293.32800299999997 39.051594 292.572998 40.006577',
        'Polygon ICRS 295.309509 41.243202 293.935272 40.636337 294.679108 39.672440 296.040527 40.271358 295.309509 41.243202',
        'Polygon ICRS 296.810974 39.209290 295.464386 38.619041 294.742615 39.587933 296.102875 40.186279 296.810974 39.209290',
        'Polygon ICRS 294.126465 38.007862 295.450867 38.613003 294.72900400000003 39.581806 293.392578 38.967735 294.126465 38.007862',
        'Polygon ICRS 290.540985 42.418102 291.92019700000003 43.066216 292.722504 42.115677 291.354645 41.477375 290.540985 42.418102',
        'Polygon ICRS 293.343323 43.703178 291.934296 43.072681 292.736481 42.122044 294.132111 42.743580 293.343323 43.703178',
        'Polygon ICRS 294.96048 41.696312 293.5802 41.084934 292.80413799999997 42.041630 294.198883 42.662075 294.96048 41.696312',
        'Polygon ICRS 292.2117 40.451481 293.566376 41.078674 292.790192 42.035275 291.423035 41.398125 292.2117 40.451481',
        'Polygon ICRS 288.342621 44.792770 289.752411 45.468586 290.622925 44.534000 289.224426 43.869076 288.342621 44.792770',
'Polygon ICRS 291.211212 46.132645 289.766815 45.475323 290.637238 44.540630 292.067963 45.187889 291.211212 46.132645',
        'Polygon ICRS 292.965546 44.155109 291.550873 43.518963 290.71048 44.460663 292.140106 45.106960 292.965546 44.155109',
'Polygon ICRS 290.151764 42.859638 291.536743 43.512451 290.696198 44.454041 289.298584 43.790154 290.151764 42.859638',
        'Polygon ICRS 290.801178 46.571159 289.348846 45.909824 288.437134 46.835545 289.90457200000003 47.508232 290.801178 46.571159',
        'Polygon ICRS 287.91650400000003 45.224083 289.33435099999997 45.903053 288.422485 46.828651 286.992126 46.137520 287.91650400000003 45.224083',
        'Polygon ICRS 285.952576 47.121140 287.39389 47.826099 288.341431 46.909172 286.910858 46.216274 285.952576 47.121140',
        'Polygon ICRS 288.890198 48.518703 287.408661 47.833126 288.356079 46.916080 289.824036 47.590477 288.890198 48.518703',
        'Polygon ICRS 288.440765 48.952713 286.95394899999997 48.260147 285.95513900000003 49.164665 287.456543 49.869904 288.440765 48.952713',
        'Polygon ICRS 285.492462 47.542152 286.939148 48.253052 285.940186 49.157440 284.481873 48.433121 285.492462 47.542152',
        'Polygon ICRS 283.343658 49.391384 284.813049 50.129852 285.851837 49.234936 284.392181 48.509758 283.343658 49.391384',
        'Polygon ICRS 286.343994 50.855862 284.828125 50.137215 285.86679100000003 49.242168 287.369812 49.948193 286.343994 50.855862',
        'Polygon ICRS 289.478119 38.467339 290.767334 39.127865 291.540619 38.185726 290.26000999999997 37.533852 289.478119 38.467339',
        'Polygon ICRS 292.094513 39.779831 290.780457 39.134464 291.55365 38.192245 292.857391 38.829796 292.094513 39.779831',
        'Polygon ICRS 293.66272 37.794167 292.371063 37.165359 291.619354 38.113033 292.922333 38.749863 293.66272 37.794167',
        'Polygon ICRS 291.087799 36.516697 292.358124 37.158936 291.606293 38.106525 290.326324 37.455437 291.087799 36.516697',
        'Polygon ICRS 287.375427 40.816921 288.687408 41.502712 289.51815799999997 40.575478 288.214722 39.899258 287.375427 40.816921',
        'Polygon ICRS 290.041351 42.179420 288.700806 41.509563 289.531464 40.582233 290.861511 41.243332 290.041351 42.179420',
        'Polygon ICRS 291.725647 40.221470 290.40817300000003 39.569923 289.60199 40.503517 290.931122 41.164028 291.725647 40.221470',
        'Polygon ICRS 289.102081 38.897488 290.394989 39.563263 289.588684 40.496765 288.286041 39.821198 289.102081 38.897488',
        'Polygon ICRS 287.748535 44.552113 288.702179 43.564049 287.441193 42.924862 286.477264 43.903084 287.748535 44.552113',
        'Polygon ICRS 289.633392 42.557888 288.711639 43.554024 287.450775 42.914936 288.384094 41.928791 289.633392 42.557888',
        'Polygon ICRS 287.051239 41.231171 286.108521 42.206261 287.341858 42.859291 288.274994 41.873398 287.051239 41.231171',
        'Polygon ICRS 285.127106 43.183533 286.098846 42.216076 287.332275 42.869217 286.36868300000003 43.847225 285.127106 43.183533',
        'Polygon ICRS 285.354279 46.867775 286.38870199999997 45.899918 285.10321 45.234722 284.059113 46.191669 285.354279 46.867775',
        'Polygon ICRS 287.396637 44.912758 286.398956 45.890087 285.113586 45.225002 286.122559 44.258755 287.396637 44.912758',
        'Polygon ICRS 284.766418 43.532673 283.748016 44.486305 285.00106800000003 45.166405 286.010559 44.200932 284.766418 43.532673',
        'Polygon ICRS 282.685883 45.440498 283.737579 44.495899 284.990723 45.176117 283.946259 46.132313 282.685883 45.440498',
        'Polygon ICRS 284.984772 47.281044 283.589417 46.547535 282.569244 47.424133 283.974854 48.169922 284.984772 47.281044',
        'Polygon ICRS 282.217651 45.790283 283.57550000000003 46.540039 282.55523700000003 47.416508 281.189667 46.653877 282.217651 45.790283',
        'Polygon ICRS 280.0354 47.582588 281.407867 48.358490 282.464813 47.491798 281.098053 46.728592 280.0354 47.582588',
        'Polygon ICRS 282.83752400000003 49.124596 281.421967 48.366241 282.478851 47.499428 283.885895 48.245720 282.83752400000003 49.124596',
        'Polygon ICRS 286.820099 40.577747 287.729279 39.596813 286.550995 38.948410 285.63385 39.920864 286.820099 40.577747',
        'Polygon ICRS 288.62109399999997 38.598602 287.738342 39.586864 286.56012 38.938545 287.45205699999997 37.959023 288.62109399999997 38.598602',
        'Polygon ICRS 286.207062 37.252937 285.30719 38.222366 286.46225 38.883125 287.354675 37.904217 286.207062 37.252937',
        'Polygon ICRS 284.374207 39.194782 285.297974 38.232128 286.45309399999996 38.892982 285.535583 39.864853 284.374207 39.194782',
        'Polygon ICRS 284.554413 42.868729 285.531433 41.906681 284.335602 41.234241 283.351105 42.186802 284.554413 42.868729',
        'Polygon ICRS 286.488159 40.926430 285.541138 41.896915 284.345398 41.224575 285.301208 40.263779 286.488159 40.926430',
        'Polygon ICRS 284.036072 39.531170 283.073273 40.481087 284.242859 41.166107 285.19870000000003 40.205799 284.036072 39.531170',
        'Polygon ICRS 282.073425 41.432640 283.063416 40.490646 284.233063 41.175770 283.248657 42.127872 282.073425 41.432640',
        'Polygon ICRS 282.115753 45.111557 283.170441 44.171783 281.958862 43.472847 280.897644 44.402168 282.115753 45.111557',
        'Polygon ICRS 284.201385 43.212601 283.18093899999997 44.162235 281.969421 43.463406 282.99798599999997 42.524460 284.201385 43.212601',
        'Polygon ICRS 281.718536 41.763348 280.68368499999997 42.690292 281.865326 43.402428 282.894135 42.464096 281.718536 41.763348',
        'Polygon ICRS 279.607391 43.617298 280.673065 42.699612 281.854767 43.411861 280.793427 44.340595 279.607391 43.617298',
    ];

    for (var k=0; k<stcs.length; k++) {
        overlay.addFootprints(A.footprintsFromSTCS(stcs[k]));
    }


    var transits;
    aladin.addCatalog(transits = A.catalogFromURL('filtered-transits.vot', {sourceSize:10, color: '#f08080', onClick: 'showPopup', name: 'Stars transited', shape: drawFunction}, null, false));

  


  var slider = document.getElementById('slider');
  slider.oninput = function() {
    
    var stepNb = this.value;
    curTimeJD = timeStart + stepNb * timeStep;
    setTitle(formatDate(jdToDate(curTimeJD)));
    transits.reportChange();
    
  };
  var interval;
  
  var setTitle = function(title) {
    $('#title').html(title);
  };
  
  var isPlaying = false;
  var startAnimation = function() {
    isPlaying = true;
    $('#control').html('&#9208;');
    interval = setInterval(function(){
        if (parseInt(slider.value)+1>=nbSteps) {
            pauseAnimation();
            return;
        }
    
        slider.stepUp();
        var stepNb = parseInt(slider.value);
        curTimeJD = timeStart + stepNb * timeStep;
        setTitle(formatDate(jdToDate(curTimeJD)));
        transits.reportChange();
   },20)
  };
  
  var pauseAnimation = function() {
    isPlaying = false;
    clearInterval(interval);
    $('#control').html('&#9654;');
  };
  
  var movePlanets = function(stepNb) {
    
  }
  

  $('#control').on('click', function() {
    isPlaying = ! isPlaying;
    
    if (isPlaying) {
      if (parseInt(slider.value)+1>=nbSteps) {
        slider.value = 0;
      }
      startAnimation();
    }
    else {
      pauseAnimation();
    }
  });

  setTitle(formatDate(jdToDate(curTimeJD)));
  


