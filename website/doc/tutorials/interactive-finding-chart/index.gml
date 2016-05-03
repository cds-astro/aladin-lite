<?DOCTYPE>
<html>

  <head>
    <&CDS.headStuff2>

    <title>Build an interactive sky map with Aladin Lite</title>

    <link rel="stylesheet" href="css/highlight/default.min.css">
    <script src="js/highlight.min.js"></script>
    <script>hljs.initHighlightingOnLoad();</script>

    <link rel="stylesheet" href="css/foundation.css" />
    <script src="js/vendor/modernizr.js"></script>

    <script src="js/pym.js"></script>

    <style>
        a {
            background-color: white;
            padding: 2px;
        }
    </style>
  </head>

  <body>



    <div style="text-align: center;" class="row"><h1>Build an interactive sky map with Aladin Lite</h1></div>
    <div class="row" style="width: 800px; max-width: 800px;background-color: #C8E1D9; padding: 20px;">

      <p><em>The short URL for this tutorial is <span style="font-size: 20;"><a href="http://tiny.cc/AL-tutorial">http://tiny.cc/AL-tutorial</a></span></em></p>

      <h2>Introduction</h2>
        <p>In this tutorial, you will learn how to build and integrate in your web page an interactive sky map of the Trifid nebula relying on Aladin Lite.</p>

        <p><a href="http://aladin.unistra.fr/AladinLite/">Aladin Lite</a> is a lightweight version of Aladin desktop, running in the browser and geared towards simple visualization of a sky region. It can be easily embedded on any web page and is controllable from a Javascript API.</p>

        <h3>Pre-requisite</h3>
        <p>In order to complete this tutorial, you will need:
          <ul>
            <li>a text editor</li>
            <li>python. Python will be used to launch a local web server allowing your browser accessing the map we will create.</li>
          </ul>
        </p>

        <h3>How this tutorial works</h3>
        <p>We will start from a blank HTML page, and embed Aladin Lite in it. Each step, built upon the previous one, will then enrich the map and highlight a particular feature of Aladin Lite.</p>

      <h2>Tutorial steps</h2>
        <h3>1. Embed Aladin Lite</h3>
        <p>We will start from the following HTML page:</p>
<pre><code>&lt;!DOCTYPE&gt;
&lt;html&gt;
  &lt;body&gt;
    &lt;h1&gt;Trifid interactive map&lt;/h1&gt;
  &lt;/body&gt;
&lt;/html&gt;
</code></pre>
        <p>Copy the snippet above and paste it in a new <code>index.html</code> file in your working directory.</p>

        <p>From your terminal, execute <code>python -m SimpleHTTPServer</code> if you run Python 2.x,<br/>or <code>python -m http.server</code> if you have Python 3.x<br/>
        From your web browser, access <a href="http://0.0.0.0:8000/index.html">http://0.0.0.0:8000/index.html</a>. You should see a nearly-blank page with just a title.</p>
        <p>Go to <a href="http://aladin.unistra.fr/AladinLite/doc/#embedding">Aladin Lite embedding documentation</a>, and generate the embedding code for a <code>700x400 pixels</code> panel, centered on target <code>Trifid</code> and with an initial field of view of 1.5 degrees.<br>
        The generated code should look like this:
        <pre><code>&lt;!-- Aladin Lite CSS style file --&gt;
&lt;link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.css" /&gt;
 
&lt;!-- Aladin Lite has a dependency on the jQuery library --&gt;
&lt;script type="text/javascript" src="http://code.jquery.com/jquery-1.9.1.min.js" charset="utf-8"&gt;&lt;/script&gt;
 
&lt;!-- Aladin Lite container at requested dimensions --&gt;
&lt;div id="aladin-lite-div" style="width:700px;height:400px;"&gt;&lt;/div&gt;

&lt;!-- Aladin Lite JS code --&gt;
&lt;script type="text/javascript" src="http://aladin.u-strasbg.fr/AladinLite/api/v2/latest/aladin.min.js" charset="utf-8"&gt;&lt;/script&gt;

&lt;!-- Creation of Aladin Lite instance with initial parameters --&gt;
&lt;script type="text/javascript"&gt;
    var aladin = A.aladin('#aladin-lite-div', {survey: "P/DSS2/color", fov:1.5, target: "trifid"});
&lt;/script&gt;</code></pre>
        </p>

        <p>Copy this code and paste it in the body element of <code>index.html</code>. Reload the page in your browser. Zoom in, zoom out, pan around.</p>

        <p>
        You should end up with something like that:
        <div style="width: 700px;height: 400px;"><iframe src="step1-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step1-source-code.html">View complete source code</a>
        </p>

        <h3>2. Change image survey</h3>
        <p>Aladin Lite gives access to a large set of image surveys (called HiPS for Hierarchical Progressive Surveys, current list <a href="http://aladin.unistra.fr/hips/list">available here</a>). Let's complete our map to allow for selection of a few different surveys. First, add the following code the line after the <code>&lt;div id="aladin-lite-div"...&gt;</code> element:
<pre><code>&lt;input id="DSS" type="radio" name="survey" value="P/DSS2/color"&gt;&lt;label for="DSS"&gt;DSS color&lt;label&gt;
&lt;input id="DSS-blue" type="radio" name="survey" value="P/DSS2/blue"&gt;&lt;label for="DSS-blue"&gt;DSS blue&lt;label&gt;
&lt;input id="2MASS" type="radio" name="survey" value="P/2MASS/color"&gt;&lt;label for="2MASS"&gt;2MASS&lt;label&gt;
&lt;input id="allwise" type="radio" name="survey" value="P/allWISE/color"&gt;&lt;label for="allwise"&gt;AllWISE&lt;label&gt;
&lt;input id="glimpse" type="radio" name="survey" value="P/GLIMPSE360"&gt;&lt;label for="glimpse"&gt;GLIMPSE 360&lt;label&gt;

</code></pre>
This will create the radio buttons allowing the survey selection.
<br/>Now, we need to add the javascript code which will react to changes in the selection (<em>ie</em> call Aladin Lite method to set the current image survey to the one identified by the value of the radio button). Add the following code at the bottom of the <code>&lt;script&gt;&lt;/script&gt;</code> section we created earlier:
<pre><code>$('input[name=survey]').change(function() {
    aladin.setImageSurvey($(this).val());
});

</code></pre>

Reload your web page, and try to change the current survey.</p>

<p>
Our page now looks like this:
        <div style="width: 700px;height: 430px;"><iframe src="step2-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step2-source-code.html">View complete source code</a>
</p>

        <h3>3. Add markers</h3>
        <p>Markers are used to pinpoint particular positions of the sky. Clicking on a marker will open a tooltip displaying some information.</p>
        <p>Let's add some markers in our map for a few objects of interest. Add the following code at the bottom of our <code>&lt;script&gt;&lt;/script&gt;</code> tag:
        <pre><code>var marker1 = A.marker(270.332621, -23.078944, {popupTitle: 'PSR B1758-23', popupDesc: '<b>Object type:</b> Pulsar'});
var marker2 = A.marker(270.63206, -22.905550, {popupTitle: 'HD 164514', popupDesc: '<b>Object type:</b> Star in cluster'});
var marker3 = A.marker(270.598121, -23.030819, {popupTitle: 'HD 164492', popupDesc: '<b>Object type:</b> Double star'});
var markerLayer = A.catalog();
aladin.addCatalog(markerLayer);
markerLayer.addSources([marker1, marker2, marker3]);
        </code></pre>
        When creating the markers objects, we provide the position (ra, dec) along with the associated title and description appearing when triggering the popup.<br/>Reload the page and click on one of the markers.<br/>

        Our map should now look like this:
        <div style="width: 700px;height: 430px;"><iframe src="step3-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step3-source-code.html">View complete source code</a>
</p>

        <h3>4. Display SIMBAD and VizieR data</h3>
        <p>Aladin Lite provides convenient methods to access catalogue data. Let's add two layers to visualize catalogue data from SIMBAD and from a VizieR table (<a href="http://vizier.u-strasbg.fr/viz-bin/VizieR-3?-source=J/ApJ/562/446/table13">J/ApJ/562/446/table13</a>). Add the following snippet to the script:

        <pre><code>aladin.addCatalog(A.catalogFromSimbad('trifid', 0.2, {shape: 'plus', color : '#5d5', onClick: 'showTable'}));
aladin.addCatalog(A.catalogFromVizieR('J/ApJ/562/446/table13', 'trifid', 0.2, {shape: 'square', sourceSize: 8, color: 'red', onClick: 'showPopup'}));
        </code></pre>
        The parameters of <code>A.catalogFromSimbad</code>, creating a layer with Simbad data, are: <code>object name or position</code>, <code>radius in degrees</code>. The last parameter is an object specifying various objects: shape, source size, color. Feel free to customize them to your taste :-)<br/>
        <code>A.catalogFromVizieR</code> has a similar signature, but takes as first parameter the name of the VizieR table to load.<br/>
        The <code>onClick</code> attribute describes what happens when we click on a source: either display the associated measurements in a table or in a popup. If you don't specify this attribute, nothing happens when clicking on a source.
<br><br>
        Reload to test the changes. Tip: if the field gets too crowded, you can hide one or more overlay layers from the <em>Manage layers</em> menu at the top left of the interface.<br>
        Here is the expected result:
        <div style="width: 700px;height: 430px;"><iframe src="step4-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step4-source-code.html">View complete source code</a>
</p>

        <h3>5. Display a footprint overlay</h3>
        <p>Other available overlays include polylines, circles and polygons. Let's draw the overlay of a HST observation:
        <pre><code>var footprintLayer = A.graphicOverlay({color: '#2345ee', lineWidth: 3});
aladin.addOverlay(footprintLayer);
footprintLayer.addFootprints([A.polygon([[270.62172, -23.04858], [270.59267, -23.08082], [270.62702, -23.10701], [270.64113, -23.09075], [270.63242, -23.08376], [270.63868, -23.07631], [270.63131, -23.07021], [270.63867, -23.06175]])]);
        </code></pre>
        Polygons are simply described as an array of [ra, dec] couples.
        <br/>
        Reload your page and you should see this:
        <div style="width: 700px;height: 430px;"><iframe src="step5-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step5-source-code.html">View complete source code</a>
</p>

        <h3>6. Visualize an EPO image in context</h3>
        <p>Finally, let's overlay an outreach image (source: <a href="http://www.eso.org/public/images/eso0930a/">ESO outreach page</a>):
        <pre><code>aladin.displayJPG('http://cdn.eso.org/images/screen/eso0930a.jpg');
        </code></pre>
        This JPEG image can be displayed in Aladin Lite because it embeds <a href="http://www.virtualastronomy.org/avm_metadata.php">AVM tags</a>, describing notably the needed WCS information.<br>

        Reload your page. Here is our final map:
        <div style="width: 700px;height: 430px;"><iframe src="step6-page.html" style="border: none; width: 100%; height: 100%;"></iframe></div>
        <a href="step6-source-code.html">View complete source code</a>
        </p>



      <h2>Going further</h2>
      The following links will help you going further with Aladin Lite and learn more about Virtual Observatory standards used under the hood:
        <h3>Aladin Lite links</h3>
        <ul>
          <li><a href="http://aladin.unistra.fr/AladinLite/doc/">Aladin Lite general documentation</a></li>
          <li><a href="http://aladin.unistra.fr/AladinLite/doc/API/">Aladin Lite API documentation</a></li>
          <li><a href="http://aladin.unistra.fr/AladinLite/doc/API/examples/">Examples on how to use the API</a></li>
          <li><a href="http://aladin.unistra.fr/AladinLite/doc/#usage">List of sites using Aladin Lite</a></li>
        </ul>

        <h3>HiPS</h3>
        <ul>
            <li><a href="http://aladin.unistra.fr/hips/">What is a HiPS?</a></li>
            <li><a href="http://aladin.unistra.fr/hips/HipsIn10Steps.gml">Generate a HiPS from your set of images</a></li>
        </ul>

        <h3>Links to IVOA standards recommendations and notes</h3>
        <ul>
          <li><a href="http://www.ivoa.net/documents/VOTable/">VOTable</a></li>
          <li><a href="http://ivoa.net/documents/MOC/">Multi-Order Coverage maps (MOC)</a></li>
          <li><a href="http://ivoa.net/documents/Notes/HiPS/index.html">Hierarchical Progressive Surveys (HiPS)</a> (not an approved standard yet)</li>
        </ul>

        <hr>
        <p><em>This tutorial was originally developped for the <a href="https://sites.google.com/a/dotastronomy.com/wiki/dotastro7/">.Astronomy 7</a> Day Zero.</em><br>
        <span style="font-size: 12px;"><em>Thomas Boch, November 2015</em>
    </div>

  <&CDS.tailArea2 'Aladin Lite' '' 'aladin'>
  <&CDS.piwikStats "aladin">

  </body>
</html>
