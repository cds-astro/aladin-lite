<html>
  <head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <&CDS.headStuff2>
    
    
  </head>
<body>
  <&CDS.headArea2 '' 'aladin'>
    
    <div class="container content" style="margin: 20px" id="container">
    </div>
    
  <&CDS.tailArea2 'Aladin Lite' '' 'aladin'>
  
  <script type="text/javascript">
  $ = $ || jqMenu;
  $.urlParam = function(name, queryString) {
      if (queryString===undefined) {
          queryString = location.search;
      }
      return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(queryString)||[,""])[1].replace(/\+/g, '%20'))||null;
  };

  $(document).ready(function() {
      var width = $.urlParam('width') || 500;
      var height = $.urlParam('height') || 500;

      // other parameters (survey, zoom, target) will be processed by Aladin Lite itself

      $('<link rel="stylesheet" href="http:\/\/aladin.u-strasbg.fr\/AladinLite\/api\/v2\/latest\/aladin.min.css" \/><div id="aladin-lite-div" style="width:' + width + 'px;height:' + height + 'px;"><\/div><script type="text\/javascript" src="http:\/\/aladin.u-strasbg.fr\/AladinLite\/api\/v2\/latest\/aladin.min.js" charset="utf-8"><\/script><script type="text\/javascript">var aladin = $.aladin("#aladin-lite-div");<\/script>').appendTo($("#container"));
  });
  </script>
      
  <&CDS.piwikStats "aladin">
</body>
</html>
