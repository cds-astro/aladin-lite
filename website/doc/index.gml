<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <link href="/assets/css/bootstrap.min.css" rel="stylesheet">
    <&CDS.headStuff2>
    <link href="/assets/css/aladin.css" rel="stylesheet">
    
    <link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.css" />
    
    <style>
    #imageSurveysList {font-size: 13px;}
    
    #imageSurveysList tr > td:first-child {font-weight: bold;}
    </style>

    <title>Aladin Lite documentation</title>
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    <header class="subhead">
      <div class="container">
        <h1>Aladin Lite</h1>
        <p>A lightweight sky atlas running in the browser</p>
      </div>
    </header>

    <ul class="breadcrumb">
      <li><a href="../">Aladin Lite</a> <span class="divider">/</span></li>
      <li class="active">Documentation</li>
    </ul>
    
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
            <li><a href="#features">Features</a></li>
            <li><a href="#usage">Usage</a></li>
            <li><a href="#source-code">Source code</a></li>
            <li><a href="#plugins">Plugins</a></li>
            <li><a href="#release-notes">Release notes</a></li>
            <li><a href="#contact">Contact</a></li>
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
            Aladin lite is a lightweight version of the <a href="<&Ala.home,u>" title="Aladin home page">Aladin Sky Atlas</a>, running in the browser and geared towards simple visualization of a sky region.
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

          <p><b>Terms of use:</b> you are welcome to integrate Aladin Lite in your web pages and to customize its GUI to your needs, but please <b>leave the Aladin logo and link intact</b> at the bottom right of the view.</p>
        </div>
        <iframe style="width: 100%; height: 100%;" marginheight="0" marginwidth="0" src="embedding.html" width="100%" height="600px" frameborder="0" scrolling="no"></iframe>
        
        
        <!-------------------------------------------------------------------->
        <!-- Javascript API -->
        <a name="API"></a>
        <div class="page-header">
          <h1>Javascript API</h1>
        </div>
        <p>Aladin Lite comes with a <a href="API/">full-featured API</a> allowing one to customize its interface, control the view, change the image survey to display, create catalogues and overlay layers, 
           develop powerful interactions between a web page and Aladin Lite.
        </p>
        <p>
        The Aladin Lite API is described on a <a href="API/">dedicated page</a>.
        </p>

        <!-------------------------------------------------------------------->
        <!-- Features -->
        <a name="features"></a>
        <div class="page-header">
          <h1>Features</h1>
        </div>
        <p>
        To give you an overview of how Aladin Lite can be used, have a look at the <a href="API/examples/">list of examples</a>.
        </p>

        <!-------------------------------------------------------------------->
        <!-- Usage -->
        <a name="usage"></a>
        <div class="page-header">
          <h1>Aladin Lite usage</h1>
        </div>
        <p>Aladin Lite has been integrated in the main CDS services:
        <ul>
            <li>On the SIMBAD page for an individual object, it provides an interactive preview image (see <a href="http://simbad.u-strasbg.fr/simbad/sim-id?Ident=M1">example for <em>Messier 1</em></a>).</li>
            <li>The VizieR results page features a <em>start Aladin Lite</em> button to visualize the positions of listed sources (see <a href="http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-source=I/259/tyc2&-c=M45&-c.rm=30">example</a>) </li>
        </ul>
        </p>
        <p>Outside CDS, Aladin Lite is used in several projects:
            <ul>
                <li><a href="http://archives.esac.esa.int/esasky-beta/" title="ESA Sky">ESA Sky beta</li>
                <li><a href="http://darts.isas.jaxa.jp/astro/judo2/" title="JUDO2">JUDO2</a> (JAXA Universe Data Oriented)</li>
                <li><a href="http://darts.isas.jaxa.jp/astro/akari/cas/tools/explore/obj.php" title="Akari explore tool">Akari explore tool</a></li>
                <li><a href="http://cassis.sirtf.com/atlas/" title="CASSIS">CASSIS atlas of Spitzer Infrared Spectra</a></li>
                <li><a href="http://www.spitzer.caltech.edu/glimpse360/aladin" title="GLIMPSE 360 data visualized by Aladin Lite">GLIMPSE 360</a></li>
                <li><a href="http://cade.irap.omp.eu/">CADE</a> (<em>Centre d'Analyse de Données Etendues</em>) uses Aladin Lite to provide previews of the HEALPix maps they publish (<a href="http://cade.irap.omp.eu/dokuwiki/doku.php?id=cgps">Example for CGPS data</a>) </li>
                <li><a href="http://www.adsass.org/aladin/">ADS All-Sky Survey</a> makes use of Aladin Lite to display heatmaps of SIMBAD objects cited in the literature.</li>
            </ul>
        </p>
        <p>If your project is using Aladin Lite, we would be happy to have it listed here. <a href="#contact">Drop us a line !</a></p>

        <!-------------------------------------------------------------------->
        <!-- Source code -->
        <a name="source-code"></a>
        <div class="page-header">
          <h1>Source code</h1>
        </div>
        <p>Aladin Lite source code is available under GPL3 licence (<a href="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/AladinLiteSrc.tar.gz">Download Aladin Lite source code</a>).
        </p>

        <!-------------------------------------------------------------------->
        <!-- Plugins -->
        <a name="plugins"></a>
        <div class="page-header">
          <h1>Plugins</h1>
        </div>
        <p>
            <ul>
                <li><b>Region editor</b></li>
            </ul>
        </p>

        <!-------------------------------------------------------------------->
        <!-- RELEASE NOTES -->
        <a name="release-notes"></a>
        <div class="page-header">
          <h1>Release notes</h1>
        </div>

        <h4>October 2014</h4>
        <p>Various bug fixes</p>

        <h4>April 2014</h4>
        <p>New API methods: getSize, getFov, world2pix, pix2world, getFovCorners</p>

        <h4>November 2013</h4>
        <p>New features: progressive catalogues, color maps, fullScreen option at startup, PNG export</p>
        <p>New API methods: setFOVRange, listeners on objectClicked, objectHovered</p>
        <p>Bug fix: support for Firefox < 4</p>

        <h4>July 2013</h4>
        <p>Added zoom control to the UI</p>
        <p>Added method to trigger object selection</p>
        
        <h4>May 2013</h4>
        <p>First public beta release</p>
        
        <!-------------------------------------------------------------------->
        <!-- Contact -->
        <a name="contact"></a>
        <div class="page-header">
          <h1>Author</h1>
        </div>
        <p>Aladin Lite is developed and maintained by Thomas Boch.</p>
        <p>Send your feedback, comments, feature requests and bug reports <a href="<&Question,u 'Aladin Lite'>">to this email address</a>.</p>
        
                
        
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
