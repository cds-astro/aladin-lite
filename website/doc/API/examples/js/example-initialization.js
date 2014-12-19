var init = function(configFile, parentDivSelector) {
    var div = $(parentDivSelector);
    $('  <div class="parent" >\n' +
      '  <div class="halfWidth">\n' +
      '  <h3>Javascript</h3><div class="bordered" id="js"></div>\n' +
      '  <h3>HTML</h3><div class="bordered" id="html"></div>\n' +
      '  </div>\n' +
      '  <div class="halfWidth">\n' +
      '  <h3>Result</h3>\n' +
      '  <iframe id="result" frameBorder="0"></iframe>\n' +
      '  </div>\n' +
      '<div style="clear: both;"></div>\n' +
      '\n' +
      '\n' 
      )
      .appendTo(div);

   $.ajax({
       url: configFile,
   }).done(function(data) {
        if (data.title) {
            $('.title').html(data.title);
        }
        if (data.explain) {
            $('.explain').html(data.explain);
        }

        var jsMirror = CodeMirror(document.getElementById('js'), {
            value: data.js,
            mode:  "javascript"
        });
        var htmlMirror = CodeMirror(document.getElementById('html'), {
            value: data.html,
            mode: "xml"
        });
        htmlMirror.setSize(null, "180");

        var update = function() {
            var js = jsMirror.getValue();
            var html = htmlMirror.getValue();

            updateIFrame('result', html, js);
        }

        update();

        jsMirror.on('change', function(e) {
            update();
        });
        htmlMirror.on('change', function(e) {
            update();
        });
   });

}

var updateIFrame = function(iframeId, html, js) {
    var iframe = $('#' + iframeId)[0];
    if (iframe.contentDocument) {
        doc = iframe.contentDocument;
    }
    else if (iframe.contentWindow) {
        doc = iframe.contentWindow.document;
    }
    else {
        doc = iframe.document;
    }

    var content = '<html><head></head><body>' + html +  '<script type="text/javascript">' + js + '</script></body></html>';
    doc.open();
    doc.writeln(content);
    doc.close();
}
