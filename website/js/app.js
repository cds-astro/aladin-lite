    function getURLParam(name, queryString){
        if (queryString===undefined) {
            queryString = location.search;
        }
        return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(queryString)||[,""])[1].replace(/\+/g, '%20'))||null;
    };

    var curSurveyId = null;
    var aladin = null;
    $(document).ready(function() {

        $(window).resize(function() {
            setSize();
        });
        setSize();
        var surveys = {};

        var survey = getURLParam('survey') || "P/DSS2/color";
        var fov = getURLParam('fov') || 3;
        if (isNaN(fov)) {
            fov = 3;
        }
        var defaultTarget = 'NGC 2024';
        var target = getURLParam('target') || 'NGC 2024';
        aladin = A.aladin('#aladin-lite-div', {survey: survey, fov: fov, target: target, showGotoControl:false, showFullscreenControl: false});
        // change link on Aladin Lite logo to point to project page
        $('.aladin-logo-container a').attr('href', 'http://aladin.u-strasbg.fr/');
        curSurveyId = survey;

        if (target!=defaultTarget) {
            $('#target').val(target);
        }
        $('#target').focus();

        $("#target").keypress(function(event) {
            if (event.which == 13) {
                goto();
            }
        });


    });

    function setSize() {
        var width = $(window).width();
        var maxWidth  = 1200;
        width = Math.min(width, maxWidth);
        var alWidth = width - 220 - 40;
        alWidth = Math.max(300, alWidth) + 'px';
        $('#aladin-lite-div').css('width', alWidth);
        $('#central').css('width', alWidth);
        //$('#container').css('width', (width-100)+'px');
        
    }


    function goto() {
       var newTarget = $("#target").val();
       aladin.gotoObject(newTarget);
       // TODO : ne mettre à jour que si le gotoObject est successful
       updateHistory();
    }

    function updateHistory() {
        if (history && history.replaceState) {
            var target = $('#target').val();
            var fov = aladin.getFov()[0].toFixed(2);
            history.replaceState(null, null, "?target=" + encodeURIComponent(target) + "&fov=" + fov + "&survey=" + encodeURIComponent(curSurveyId));
        }
    }

    function find(s) {
        for (var k=0; k<surveys.length; k++) {
            if (surveys[k].id == s) {
                return surveys[k];
            }
        }
        return null;
    }

    function setInfo(s) {
        var s1 = find(s);
        if (!s1) {
            return;
        }
        $('#content').html(s1.description+" - <a href=\""+s1.copyrightUrl+"\">"+s1.copyright+"</a>");
    }

    function setSurvey(s) {
        aladin.setImageSurvey(s);
        setInfo(s);
     }
    
    $.ajax({
         url: "http://aladin.u-strasbg.fr/java/nph-aladin.pl",
         data: {"frame": "aladinLiteDic"},
         method: 'GET',
         dataType: 'jsonp',
         success: function(data) {
             var tooltipDescriptions = {};
             var res = '<div class="surveys-list">';
             data.sort(function(a,b) { return a.order == b.order ? 0 : a.order < b.order ? -1 : 1; });
             surveys = data;
             for (var k=0; k<data.length; k++) {
                  var id = data[k].id;
                  var w = /^\w+\/(\w+)/.exec(data[k].treePath)[1];
                  var s1 = id.substring(2).replace("/color","");
                  var imgPath = 'survey-previews/' + id.replace(/\//g, "_") + '.jpg';
                  res += '<div class="survey" data-surveyId="' + id + '"><div class="survey-label">' + s1 + '</div><img class="survey-preview" src="' + imgPath + '" /><div class="survey-selected" style="display: none;"><div class="survey-selected-img"></div></div></div>';
                  tooltipDescriptions[id] = '<div>Band: ' + w + '</div><div>' + data[k].description + '</div>';
             }
             res += '</div>';
             $('#surveys').html(res);

             $('.survey').each(function() {
                 $(this).tooltipster({
                     content: $(tooltipDescriptions[$(this).attr('data-surveyId')]),
                     delay: 800,
                     position: 'right'
                 });
             });
             $('.survey').click(function() {
                curSurveyId = $(this).attr('data-surveyId');
                setSurvey(curSurveyId);
                updateHistory();
                $('.survey-selected').hide();
                $(this).find('.survey-selected').show();
             });

             // once the info about surveys retrieved, we can set the info about the current one
             setInfo(curSurveyId);
             var currentSurveyDiv = $('.survey[data-surveyId="' + curSurveyId + '"]');
             currentSurveyDiv.find('.survey-selected').show();
             // scroll to current survey if needed
             var shiftY = currentSurveyDiv.position().top - $('.surveyDiv').position().top;
             if (shiftY>400) {
                $('.surveyDiv').animate({scrollTop: shiftY});
             }
         },
         error: function() { $('#surveys').html("Error: "+url); }
     });


