<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <link href="css/bootstrap.min.css" rel="stylesheet">
    <&CDS.headStuff2>
    <link href="css/aladinlite.css" rel="stylesheet">
    
    <link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.css" />
    
    <style>
    #imageSurveysList {font-size: 13px;}
    
    #imageSurveysList tr > td:first-child {font-weight: bold;}
    </style>
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    <header class="subhead">
      <div class="container">
        <h1>Aladin Lite</h1>
        <p>A lightweight sky atlas running in the browser</p>
      </div>
    </header>
    
    <div class="container content">
    
      <!-- Navigation bar -->
      <div class="row">
        <div class="span3">
          <ul class="nav nav-tabs nav-stacked aladinlite-sidenav" data-spy="affix" data-offset-top="210">
            <li><a href="#overview">Overview</a></li>
            <!-- <li><a href="#features">Features</a></li> -->
            <!-- <li><a href="#requirements">Requirements and limitations</a></li> -->
            <li><a href="#embedding">Embedding in a web page</a></li>
            <li><a href="#API">Javascript API</a></li>
            <li><a href="#examples">Usage examples</a></li>
            <li><a href="#release-notes">Release notes</a></li>
            <li><a href="#authors">Author</a></li>
          </ul>
        </div>
        
        <div class="span9">
        
        <!-------------------------------------------------------------------->
        <!-- OVERVIEW -->
        <a name="overview"></a>
        <div class="page-header">
          <h1>Overview</h1>
        </div>
        
        <div class="row">
          <div class="span5">
            <p class="lead">
            Aladin lite is a lightweight version of the <a href="<&Ala.home,u>" title="Aladin home page">Aladin tool</a>, running in the browser and geared towards simple visualization of a sky region.
            </p>
            <p>It allows one to visualize image surveys (JPEG multi-resolution HEALPix all-sky surveys)
            and superimpose tabular (VOTable) and footprints (STC-S) data.</p>
            <p>Aladin lite is powered by the HTML5 canvas technology, currently supported by any modern browser</p>
            <p>Aladin lite is <a href="#embedding">easily embeddable on any web page</a> and can also be
            controlled through a <a href="#API">Javacript API</a>.</p>
            <p>It is dedicated to replace the Aladin Java applet technology in the medium term.</p>
            <br/>
            <p>The panel on the right hand is not a regular image. It is actually Aladin Lite running as an embedded widget. 
            You might try to zoom in and out using your mouse wheel, or pan the view to move around.</p>
          </div>
          <div class="span3">
              <div id="aladin-lite-div" style="width:300px;height:300px;"></div>
              <script type="text/javascript" src="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.js" charset="utf-8"></script>
              <script type="text/javascript">
                var aladin = $.aladin('#aladin-lite-div', {showControl: false, fov: 0.5, target: "20 45 38.0 +30 42 30", cooFrame: "J2000", survey: "P/DSS2/color", showFullscreenControl: false, showFrame: false, showGotoControl: false});
              </script>
              
              <script>
              </script>
          </div>
        </div>

        <!-------------------------------------------------------------------->
        <!-- Embedding -->
        <a name="embedding"></a>
        <div class="page-header">
          <h1>Embedding in a web page</h1>
        </div>
        <iframe style="width: 100%; height: 100%;" marginheight="0" marginwidth="0" src="embedding.html" width="100%" height="600px" frameborder="0" scrolling="no"></iframe>
        
        
        <!-------------------------------------------------------------------->
        <!-- Javascript API -->
        <a name="API"></a>
        <div class="page-header">
          <h1>Javascript API</h1>
        </div>
        <p>If you want to develop powerful interactions between your web page and Aladin Lite, 
        the Javascript API is meant for you. It will let you create an Aladin lite instance bound to a given div element 
        of your page, and provide with methods to interact with this instance.</p>
        
        <h3>Create an Aladin instance</h3>
        First, insert the <a href="#embedding">embedding code</a> in your page. After calling <code>var aladin = $.aladin('#aladin-lite-div');</code>, the variable <code>aladin</code> is a reference to the Aladin lite instance.
        
        <h3>Point to a given location</h3>
        2 methods are available:
        <ul>
            <li><code>gotoObject(objectName)</code> will point the view to a given object whose position is resolved by the <a href="http://cds.u-strasbg.fr/cgi-bin/Sesame">Sesame service</a><br/>eg: <code>aladin.gotoObject("Messier 81");</code><br/></li>
            <li><code>gotoPosition(ra, dec)</code> will point the view to the given position in ICRS coordinates.<br/>eg: <code>aladin.gotoPosition(83.83213, -5.42911);</code></li>
        </ul>
        
        <h3>Set zoom level</h3>
        <code>setZoom(fovInDegrees)</code> will set the zoom level to the specified value (in degrees).<br/>
        <code>aladin.setZoom(0.5)</code> will set the zoom so that the visible view is 30 arcmin (0.5 degree) large.
        
        <h3>Set image survey</h3>
        <code>setImageSurvey(surveyIdentifier)</code> will set the background image survey to the one corresponding to 
        the <code>surveyIdentifier</code> parameter.<br/>
        eg: <code>setImageSurvey("P/2MASS/color")</code><br/><br/>
        The list of the main available surveys is as follows :
        
        <table class="table table-striped" id="imageSurveysList">
          <thead>
            <tr>
              <th>Identifier</th><th>Description</th>
            </tr>
          </thead>
          <tbody>
            
          </tbody>
        </table>

        <p>You can also display any of the <a href="http://aladin.u-strasbg.fr/java/nph-aladin.pl?frame=aladinHpxList" title="List of Aladin HEALPix surveys">JPEG surveys listed on this page</a>.</p>

        <br/><br/>
        <p>If you wish to display your own HEALPix survey, <a title="How to create your own all-sky survey" href="http://aladin.u-strasbg.fr/java/FAQ.htx#ToC97">generated from Aladin Java</a>, use the following code:<br/>
        <pre>aladin.setImageSurvey(aladin.createImageSurvey(&lt;survey-id&gt;, &lt;survey-name&gt;,
 &lt;survey-root-URL&gt;, &lt;survey-frame:"galactic"|"j2000"&gt;, &lt;maximum-norder&gt;))</pre>.<br/>
        Example: <pre>aladin.setImageSurvey(aladin.createImageSurvey('IRIS12UM', 'IRIS12UM',
'http://alasky.u-strasbg.fr/IRIS_1/', 'galactic', 3));</pre>
        
        </p>

        <h3>Parse and visualize a VOTable</h3>
        <code>createCatalogFromVOTable(votableUrl)</code> will parse the VOTable at the given URL and return a Catalog instance.<br/>
        You can then call <code>addCatalog(catalog)</code> to add it to the view.

        <h3>Visualize footprints</h3>
        <code>createFootprintFromSTCS(stcString)</code> will parse the given <a href="http://www.ivoa.net/documents/Notes/STC-S/">STC-S</a> string and return an array of Footprint instances.</code><br/>
        In order to visualize them, create a new Overlay with <code>createOverlay()</code>, add it by calling <code>addOverlay(overlay)</code> and eventually add the footprints to the Overlay instance: <code>overlay.addFootprints(footprints)</code>.<br/>
        Example:

        <pre>
var overlay = aladin.createOverlay();
aladin.addOverlay(overlay);
var footprints = aladin.createFootprintsFromSTCS('Polygon J2000 180.74436 -18.90167 180.75044 -18.90167 180.78824 -18.76744 180.78823 -18.76191 180.78822 -18.76185 180.64612 -18.72635 180.64003 -18.72635 180.60216 -18.86057 180.60216 -18.86059 180.60216 -18.86613');
overlay.addFootprints(footprints);
        </pre>

        <!-------------------------------------------------------------------->
        <!-- Usage examples -->
        <a name="examples"></a>
        <div class="page-header">
          <h1>Usage examples</h1>
        Below are pointers to some example pages using Aladin Lite:
        <ul>
            <li><a href="examples/thumbnails.html">Visualizing thumbnails of galaxies</a></li>
            <li><a href="examples/sources-overlaid.html">Visualizing a list of sources overlaid on a background image</a></li>
            <li><a href="examples/footprints.html">Visualizing a list of footprints overlaid on a background image</a></li>
            <li><a href="examples/full-screen.html">Fullscreen Aladin Lite</a></li>
        </ul>
        </div>


        <!-------------------------------------------------------------------->
        <!-- RELEASE NOTES -->
        <a name="release-notes"></a>
        <div class="page-header">
          <h1>Release notes</h1>
        </div>
        <h4>July 2013</h4>
        <p>Added zoom control to the UI</p>
        
        <h4>Version 0.5 (May 2013)</h4>
        <p>First public beta release</p>
        
        <!-------------------------------------------------------------------->
        <!-- Authors -->
        <a name="authors"></a>
        <div class="page-header">
          <h1>Author</h1>
        </div>
        <p>Aladin Lite is developed and maintained by Thomas Boch.</p>
        <p>Send your feedback, comments and bug reports <a href="<&Question,u 'Aladin Lite'>">to this email address</a>.</p>
        
                
        
        </div> <!-- /span3 -->
      </div> <!-- /row -->
    
    </div>
  <&CDS.tailArea2 'Aladin Lite' '' 'aladin'>
  
  <script src="js/bootstrap-affix.js"></script>
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
