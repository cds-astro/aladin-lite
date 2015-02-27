<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <link href="/assets/css/bootstrap.min.css" rel="stylesheet">
    <&CDS.headStuff2>
    <link href="/assets/css/aladin.css" rel="stylesheet">
    
    
    <style>
    #imageSurveysList {font-size: 13px;}
    
    #imageSurveysList tr > td:first-child {font-weight: bold;}
    </style>
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    <header class="subhead">
      <div class="container">
        <h1>Aladin Lite API reference</h1>
      </div>
    </header>

    <ul class="breadcrumb">
      <li><a href="../">Aladin Lite</a> <span class="divider">/</span></li>
      <li><a href="../doc/">Documentation</a> <span class="divider">/</span></li>
      <li class="active">API</li>
    </ul>
    
    <div class="container content">
    
      <!-- Navigation bar -->
      <div class="row">
        <div class="span3">
          <ul class="nav nav-tabs nav-stacked aladinlite-sidenav" data-spy="affix" data-offset-top="210">
            <li><a href="#init">Initializing Aladin Lite</a></li>
            <li><a href="#view">Managing the view</a></li>
            <li><a href="#image-layers">Image layers</a></li>
            <li><a href="#catalogue-layers">Catalogue layers</a></li>
            <li><a href="#overlay-layers">Overlay layers</a></li>
            <li><a href="#listeners">Listeners</a></li>
            <li><a href="#misc">Misc</a></li>
            <li><a href="#examples-list">API examples</a></li>
          </ul>
        </div>
        
        <div class="span9">
        
        <!-------------------------------------------------------------------->
        <!-- Initializing Aladin Lite -->
        <a name="init"></a>
        <div class="page-header">
          <h1>Initializing Aladin Lite</h1>
        </div>
        <h3>Aladin Lite creation</h3>
        <p><b>A.aladin(&lt;container-div-selector&gt;, &lt;options&gt;?)</b></p>
        <p>Creating an Aladin Lite instance is quite easy: after having inserted the <a href="http://aladin.u-strasbg.fr/AladinLite/doc/#embedding">embedding code</a> in your page, 
        just call <code>var aladin = A.aladin('#aladin-lite-div');</code>. The variable <code>aladin</code> is a reference to the Aladin lite instance.
        </p>
        <p>Note that Aladin Lite view will automatically adapt if the container div is resized, as shown <a href="examples/AL-in-responsive-div/">in this example</a>.</p>
        <h3>Initialization options</h3>
        <p>The method takes an optional second argument which gives the initialization options as a key-value object. Possible options are:
        <table class="table table-striped">
            <thead>
              <tr><th>Key name</th><th>Description</th><th>Default value</th></tr>
            </thead>
            <tbody>
                <tr><td>target</td><td>Initial target, as a position or an object name resolved by Sesame</td><td>0 +0</td></tr>
                <tr><td>cooFrame</td><td>Coordinate system: "ICRS" or "galactic"</td><td>"ICRS"</td></tr>
                <tr><td>survey</td><td>Identifier of the initial image survey. See <a href="#image-layers">this section</a> for more details.</td><td>"P/DSS2/color"</td></tr>
                <tr><td>fov</td><td>Initial value of the visible field of view, in decimal degrees</td><td>60</td></tr>
                <tr><td>showReticle</td><td>If <code>true</code>, the reticle will be displayed</td><td>true</td></tr>
                <tr><td>showZoomControl</td><td>If <code>true</code>, the zoom control GUI is displayed (plus/minus buttons)</td><td>true</td></tr>
                <tr><td>showFullscreenControl</td><td>If <code>true</code>, the button to pass in full screen mode (at the top right of the interface) is displayed</td><td>true</td></tr>
                <tr><td>showLayersControl</td><td>If <code>true</code>, the icon to open the layers controls is displayed</td><td>true</td></tr>
                <tr><td>showGotoControl</td><td>If <code>true</code>, the icon to easily jump to a new position/object is displayed</td><td>true</td></tr>
                <tr><td>showFrame</td><td>If <code>true</code>, coordinate are displayed (at the top left of the interface)</td><td>true</tr>
                <tr><td>fullScreen</td><td>If <code>true</code>, Aladin Lite starts in "full screen" mode</td><td>false</td></tr>
                <tr><td>reticleColor</td><td>Color of the reticle</td><td>"rgb(178, 50, 178)"</td></tr>
                <tr><td>reticleSize</td><td>Size in pixels of the reticle</td><td>22</td></tr>
            </tbody>
          </table>
        </p>
        <p><a href="examples/AL-init-custom-options/">Example of Aladin Lite initialization with custom options</a></p>

        <h3>Customizing location of GUI elements</h3>
        <p>If you want to customize the location of the GUI elements (full screen icon, zoom controller, goto controller, layers controller), the easiest way is to 
        override the CSS properties of the corresponding class (respectively .aladin-fullscreen, .aladin-zoomControl, .aladin-gotoControl, .aladin-layersControl).
        </p>

        <!-------------------------------------------------------------------->
        <!-- Managing the view -->
        <a name="view"></a>
        <div class="page-header">
          <h1>Managing the view</h1>
        </div>

        <h3>Getting information about the view</h3>
        <p><code>aladin.getSize()</code> will return an array with the current dimension (width, height) of Aladin Lite view in pixels.</p>
        <p><code>aladin.getFov()</code> returns an array with the current dimension on the sky (size in X, size in Y) of the view in decimal degrees.</p>
        <p></p>

        <h3>Updating view properties</h3>
        <b>Setting the size of the FoV</b>
        <p>Use <code>aladin.setFov()</code>
        <p></p>

        <b>Setting the current position of the center of the view</b>
        <p>There are several methods to update the current position:</p>
        



        <!-------------------------------------------------------------------->
        <!-- Image layers -->
        <a name="image-layers"></a>
        <div class="page-header">
          <h1>Image layers</h1>
        </div>

        <h3>Retrieving the base image layer</h3>
        <p>Calling <code>aladin.getBaseImageLayer()</code> will return an ImageLayer object corresponding to the base image layer.</p>

        <h3>Updating the color map</h3>
        <p>Call <code>getColorMap()</code> on an ImageLayer object to retrieve its color map.<br/>
        Once retrieved, call <code>update(&lt;color-map-name&gt;)</code> to set a new color map. Recognized values are:
        <ul>
            <li><i>grayscale</i></li>
            <li><i>eosb</i></li>
            <li><i>rainbow</i></li>
            <li><i>native</i> to go back to the original image color map</li>
        </ul>
        </p>
        
        <p>Example: <a href="examples/color-map/">setting a rainbow color map</a></p>

        <!-------------------------------------------------------------------->
        <!-- Catalogue layers -->
        <a name="catalogue-layers"></a>
        <div class="page-header">
          <h1>Catalogue layers</h1>
        </div>

        <h3>Loading a VOTable</h3>
        <p>Calling <code>A.catalogFromURL(&lt;votable-URL&gt;, &lt;options&gt;?)</code> will return a Catalog object which can then be added to the Aladin Lite instance.</p>

        <p>Example: <a href="examples/load-votable/">loading a VOTable from VizieR</a></p>

        <h3>Creating a marker</h3>
        <p>A marker displays a position on the sky. Clicking on a marker will open a popup with a title and text set upon creation.</p>
        <p>Use <code>A.marker(ra, dec, {popupTitle: &lt;title of the popup&gt;, popupDesc: &lt;text (possibly HTML) for the popup&gt;})</code> to create a new marker. You can then add it to an existing Catalog.</p>

        <p>Example: <a href="examples/marker-creation/">Creating multiple markers</a></p>

        <!-------------------------------------------------------------------->
        <!-- Overlay layers -->
        <a name="overlay-layers"></a>
        <div class="page-header">
          <h1>Overlay layers</h1>
        </div>
        <p></p>

        <!-------------------------------------------------------------------->
        <!-- Listeners -->
        <a name="listeners"></a>
        <div class="page-header">
          <h1>Listeners</h1>
        </div>
        <p></p>

        <!-------------------------------------------------------------------->
        <!-- Misc -->
        <a name="misc"></a>
        <div class="page-header">
          <h1>Misc</h1>
        </div>
        <p></p>
        
        <!-------------------------------------------------------------------->
        <!-- Full list of examples -->
        <a name="examples-list"></a>
        <div class="page-header">
          <h1>Full list of examples</h1>
        </div>
        <p><a href="examples/">Click here</a> to access to the full list of examples illustrating Aladin Lite API.</p>
        
        </div> <!-- /span3 -->
      </div> <!-- /row -->
    
    </div>
  <&CDS.tailArea2 'Aladin Lite' '&rarr;  Thanks for <a href="/aladin.gml#Acknowledgement">acknowledging Aladin Sky Atlas</a>' 'aladin'>
  
  <script src="/assets/js/bootstrap-affix.js"></script>
  <script type="text/javascript">
  $(document).ready(function() {
      $.ajax({
        url: "http://aladin.u-strasbg.fr/java/nph-aladin.pl",
        data: {"frame": "aladinLiteDic"},
        method: 'GET',
        dataType: 'jsonp',
        success: function(surveys) {
            surveys = surveys.sort(function(a, b) {return a.order > b.order ? 1 : -1;});
            var content = "";
            for (var k=0, len=surveys.length; k<len; k++) {
                content += '<tr><td>' + surveys[k].id + '</td><td>' + surveys[k].name + '</td></tr>';
            }
            $('#imageSurveysList tbody').append(content); 
        }
      });
  });
  </script>
  <&CDS.piwikStats "aladin">
</body>
</html>
