<!DOCTYPE html>
<html>
<head>
<meta name="description" content="Kepler transits in Aladin Lite">
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width">
  <title>Kepler transits in Aladin Lite</title>

<style>
#timeControl {
    padding: 5px;
    background: white;
    z-index: 10000;
}
</style>
</head>
<body>
    <h3>24 hours of Kepler transits visualised in Aladin Lite</h3>
    <p>Hit the <span id="play">&#9654;</span> button or move manually the time slider to visualise Kepler stars being transited at the corresponding epoch.<br>
Symbol size encodes the exoplanet radius whereas the color denotes its temperature.
    <br>
    <em>Inspired by <a href="https://twitter.com/bochthomas/status/1042070735332753408">Ethan Kruse tweet</a></em>
    </p>
<!-- include Aladin Lite CSS file in the head section of your page -->
<link rel="stylesheet" href="https://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.css" />
 
<!-- you can skip the following line if your page already integrates the jQuery library -->
<script type="text/javascript" src="https://code.jquery.com/jquery-1.12.1.min.js" charset="utf-8"></script>

<div id="timeControl"> 
<button id="control">&#9654;</button> <input id="slider" style="vertical-align:middle;width:60vw;" step=1 min=0 max=1440 type="range" value="0" name="">
  <div id="title"></div>
</div>
<!-- insert this snippet where you want Aladin Lite viewer to appear and after the loading of jQuery -->
<div id="aladin-lite-div" style="width:90vw;max-width: 1500px;height:85vh;max-height: 1500px;"></div>

  
<script type="text/javascript" src="https://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.js" charset="utf-8"></script>

<script type="text/javascript" src="js/js-colormaps.js"></script>
<script type="text/javascript" src="js/app.js"></script>
<&CDS.piwikStats "aladin">
<script>
$('#play').on('click', function() {
     $('#control').click();
});

</script>
</body>
</html>

