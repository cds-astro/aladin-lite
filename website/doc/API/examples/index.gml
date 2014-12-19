<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <link href="/assets/css/bootstrap.min.css" rel="stylesheet">
    <&CDS.headStuff2>
    <link href="/assets/css/aladin.css" rel="stylesheet">
    <link href="css/codemirror.css" rel="stylesheet">
    <link href="css/style.css" rel="stylesheet">

    <script type="text/javascript" src="<&CDS.jquery1.10.1,u>"></script>
    <script type="text/javascript" src="js/codemirror.js"></script>
    <script type="text/javascript" src="js/javascript.js"></script>
    <script type="text/javascript" src="js/xml.js"></script>
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    <header class="subhead">
      <div class="container">
        <h1>Aladin Lite API examples list</h1>
      </div>
    </header>

    <ul class="breadcrumb">
      <li><a href="../../../">Aladin Lite</a> <span class="divider">/</span></li>
      <li><a href="../../">Documentation</a> <span class="divider">/</span></li>
      <li><a href="../">API</a><span class="divider">/</span></li>
      <li class="active title">Examples<li>
    </ul>

    <div class="container content">
        <div class="row">
            <ul id="examples-list">
            </ul>
        </div>
    </div>
    
    

  <&CDS.tailArea2 'Aladin Lite' '&rarr;  Thanks for <a href="/aladin.gml#Acknowledgement">acknowledging Aladin Sky Atlas</a>' 'aladin'>
  
  <&CDS.piwikStats "aladin">

  <script type="text/javascript">
    $.ajax({
       url: 'all-examples.json',
    }).done(function(items) {
        for (var k=0; k<items.length; k++) {
            var title = items[k].title;
            var url = items[k].url;
            $('#examples-list').append('<li><a href="' + url + '">' + title + '</a></li>');
        }
    });
  </script>

</body>
</html>
