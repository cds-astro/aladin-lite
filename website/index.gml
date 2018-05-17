<!DOCTYPE html>
<html>

<head>
<meta http-equiv="X-UA-Compatible" content="IE=edge" />

<&CDS.headStuff2>

    <link href="css/style.css" rel="stylesheet" />
    <link rel="stylesheet" href="http://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.css" />
    <link rel="stylesheet" type="text/css" href="css/tooltipster.css" />

    <link rel="stylesheet" type="text/css" href="./slick/slick.css" />
    <link rel="stylesheet" type="text/css" href="./slick/slick-theme.css" />

    <style type="text/css">
        .slider {
            width: 900px;
            margin: 100px auto;
        }

        .slick-slide {
            width: 200px;
            margin: 0px 10px;
        }

/*
        .slick-slide img {
            width: 100%;
        }
*/


        .slick-slide {
            transition: all ease-in-out .3s;
            opacity: .2;
        }
        .slick-active {
        opacity: 1;
        }

        .slick-current {
        opacity: 1;
        }

    </style>

    <title>Aladin Lite</title>

    <script type="text/javascript" src="js/jquery.tooltipster.min.js"></script>
</head>

<body>
    <&CDS.headArea2 'Aladin Lite' 'aladin'>

    <div id="container">
      <div id="left"> 
           <div class="targetDiv"><div class="title">Target:</div> <input id="target"></div>
           <div class="surveyDiv"><div class="title">Surveys:</div><div id="surveys"></div> </div>
       </div>
      <div id="central">
          <div id="aladin-lite-div"></div>
      </div>
    </div>
    <div class="bottom" id="content" ></div>
    <br/>
    <div class="developerInfo">Are you a developer interested in integrating Aladin Lite in your project ? Have a look at the <a href="doc/">dedicated documentation.</a></div>
    <br/><br/>
    
    <script type="text/javascript" src="http://aladin.u-strasbg.fr/AladinLite/api/v2/beta/aladin.min.js" charset="utf-8"></script>
    <script type="text/javascript" src="slick/slick.min.js"></script>
    <script type="text/javascript" src="js/app.js"></script>

    <&CDS.tailArea2 'Aladin Lite' '&rarr;  Thanks for <a href="/aladin.gml#Acknowledgement">acknowledging Aladin Sky Atlas</a>' 'aladin'>

    <&CDS.piwikStats "aladin">

</body>
</html>
