<!DOCTYPE html>
<html>
<!-- paulirish.com/2008/conditional-stylesheets-vs-css-hacks-answer-neither/ -->
<!--[if lt IE 7 ]> <html class="ie6" lang="en"> <![endif]-->
<!--[if IE 7 ]>    <html class="ie7" lang="en"> <![endif]-->
<!--[if IE 8 ]>    <html class="lt-ie9" lang="en"> <![endif]-->
<!--[if IE 9 ]>    <html class="lt-ie10" lang="en"> <![endif]-->
<!--[if (gte IE 9)|!(IE)]><!--> <html lang="en"> <!--<![endif]-->
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>twentytwenty</title>
  <link href="css/twentytwenty.css" rel="stylesheet" type="text/css" />
  <link rel="stylesheet" href="//aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.css" />
  <link rel="stylesheet" href="css/app.css">

  <link rel="stylesheet" href="css/pure-min.css">
  <link rel="stylesheet" href="css/grids-responsive-min.css">


  <&CDS.headStuff2>

</head>
  <body>


    <div class="pure-g">
      <div class="pure-u-1-3"></div>
      <div class="pure-u-1-3"><h2>Split views with slider</h2></div>
      <h4 class="pure-u-1-3">Share the slider to compare the two selected HiPS.<br/></h4>
    </div>


    <div class="twentytwenty-container" style="width: 1300px; height: 800px;">
        <div id="al1" class="aladin"></div>
        <div id="al2" class="aladin"></div>
    </div>

    <script
  src="https://code.jquery.com/jquery-3.2.1.js"
  crossorigin="anonymous"></script>
    <script src="js/jquery.event.move.js"></script>
    <script src="js/jquery.twentytwenty.js"></script>
    <script type="text/javascript" src="//aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.js" charset="utf-8"></script>
    <script>
        View.CALLBACKS_THROTTLE_TIME = 30;
        var a1 = A.aladin('#al1', {survey: "P/DSS2/color", fov:0.3, target: 'M1'});
        var a2 = A.aladin('#al2', {survey: "P/2MASS/color", fov:0.3, target: 'M1'});
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
    </script>
  </body>
</html>
