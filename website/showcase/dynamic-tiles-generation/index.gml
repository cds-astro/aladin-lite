<!DOCTYPE html>
<html>
<head>
<meta name="description" content="Kepler transits in Aladin Lite">
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width">
  <title>HiPS tiles generation</title>

</head>
<body>
    <h3>HiPS tiles generation on the fly</h3>
    <p>Play with the sliders or change the color map to see the tiles being updated</p>
    </p>
<!-- include Aladin Lite CSS file in the head section of your page -->
<link rel="stylesheet" href="https://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.css" />
 
<!-- you can skip the following line if your page already integrates the jQuery library -->
<script type="text/javascript" src="https://code.jquery.com/jquery-1.12.1.min.js" charset="utf-8"></script>

<!-- insert this snippet where you want Aladin Lite viewer to appear and after the loading of jQuery -->
<div id="aladin-lite-div" style="width:60vw;max-width: 1500px;height:70vh;max-height: 1500px;"></div>

<label for="stretch">Stretch: </label>
<select id="stretch">
  <option value="power">Pow</option>
  <option value="linear">Linear</option>
  <option value="sqrt">Square root</option>
  <option value="log">Log</option>
  <option value="asinh" selected>Asinh</option>
</select>

<label for="cmap">Color map: </label>
<select id="cmap">
</select>
<br/>

<label for="min_cut">Min cut: </label>
<input id="min_cut" style="vertical-align: middle; width:20vw;" step="0.05" min="-0.5" max="10" type="range" value="-0.2" />
<br>
<label for="max_cut">Max cut: </label>
<input id="max_cut" style="vertical-align: middle; width:20vw;" step="0.05" min="-0.5" max="10" type="range" value="6" />
  
<script type="text/javascript" src="https://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.js" charset="utf-8"></script>

<script type="text/javascript" src="js/app.js"></script>
<&CDS.piwikStats "aladin">
</body>
</html>

