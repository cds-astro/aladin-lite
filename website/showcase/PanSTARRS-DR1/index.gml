<!DOCTYPE html>
<html lang="en">

  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">

    <title>PanSTARRS DR1 visualized in Aladin Lite</title>

    <link rel="stylesheet" href="css/pure-min.css">
    <link rel="stylesheet" href="css/grids-responsive-min.css">

    <link rel="stylesheet" href="css/app.css">

    <link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.css" />

    <&CDS.headStuff2>

  </head>

  <body>
  
    <&CDS.headArea2 '' 'aladin'>


    <div class="pure-g">
      <div class="pure-u-1-3"><img style="width:80px;float: right;" src="http://vizier.u-strasbg.fr/vizier/logos/PanSTARRS.png"></div>
      <div class="pure-u-1-3"><h2>PanSTARRS data in Aladin Lite</h2></div>
      <h4 class="pure-u-1-3">Zoom in to make fainter sources appear.<br/>Click on a source to display its measurements.</h4>
    </div>


    <div id="aladin">
      <div id="explain" class="aladin-box"></div>
      <div id="layersControl" class="aladin-box">
        <b>Background</b>
        <form class="pure-form pure-form-stacked">
          <fieldset>
            <label for="option-ps-color" class="pure-radio">
              <input id="option-ps-color" type="radio" name="img-hips" value="panstarrs-color" checked>
              PanSTARRS DR1 color images
            </label>
            <label for="option-ps" class="pure-radio">
              <input id="option-ps" type="radio" name="img-hips" value="panstarrs-g">
              PanSTARRS DR1 g-band images
            </label>
            <label for="option-ps-dmap" class="pure-radio">
              <input id="option-ps-dmap" type="radio" name="img-hips" value="panstarrs-density-map">
              PanSTARRS DR1 catalogue density map 
            </label>
            <label for="option-DSS-dmap" class="pure-radio">
              <input id="option-DSS-dmap" type="radio" name="img-hips" value="P/DSS2/color">
              DSS Color
            </label>

          </fieldset>
        </form>
        <hr>
        <b>Overlays</b>
        <form class="pure-form pure-form-stacked" id="overlay-form">
          <fieldset>

          <label for="PS">
            <input id="PS" type="checkbox" value="ps1" checked>PanSTARRS DR1 sources
          </label>

          <label for="simbad">
            <input id="simbad" type="checkbox" value="simbad">Simbad objects
          </label>

          </fieldset>
        </form>
      </div>
    <div>

  </body>


  <script type="text/javascript" src="http://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.js" charset="utf-8"></script>
  <script src="js/app.js"></script>

  <&CDS.piwikStats "aladin">

</html>
