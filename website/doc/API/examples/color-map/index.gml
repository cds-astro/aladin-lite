<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <link href="/assets/css/bootstrap.min.css" rel="stylesheet">
    <&CDS.headStuff2>
    <link href="/assets/css/aladin.css" rel="stylesheet">
    <link href="../css/codemirror.css" rel="stylesheet">
    <link href="../css/style.css" rel="stylesheet">

    <script type="text/javascript" src="<&CDS.jquery1.10.1,u>"></script>
    <script type="text/javascript" src="../js/codemirror.js"></script>
    <script type="text/javascript" src="../js/javascript.js"></script>
    <script type="text/javascript" src="../js/xml.js"></script>
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    <header class="subhead">
      <div class="container">
        <h1>Aladin Lite API example</h1>
        <p class="title"></p>
      </div>
    </header>

    <ul class="breadcrumb">
      <li><a href="../../../../">Aladin Lite</a> <span class="divider">/</span></li>
      <li><a href="../../../">Documentation</a> <span class="divider">/</span></li>
      <li><a href="../../">API</a><span class="divider">/</span></li>
      <li><a href="../">Examples</a><span class="divider">/</span></li>
      <li class="active title"><li>
    </ul>
   
    <p class="explain"></p>
    <div id="binContainer"></div>

    
    

  <&CDS.tailArea2 'Aladin Lite' '&rarr;  Thanks for <a href="/aladin.gml#Acknowledgement">acknowledging Aladin Sky Atlas</a>' 'aladin'>
  
  <&CDS.piwikStats "aladin">

  <script type="text/javascript" src="../js/example-initialization.js"></script>
  <script type="text/javascript">
    $(document).ready(function() {
        init('../configs/color-map.json', '#binContainer');
    });
  </script>
</body>
</html>
