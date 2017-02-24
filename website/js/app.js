     function propertiesDictFromHiPSId(hipsId, callback) {
        if (! callback) {
            return;
        }

        $.ajax({
            url: 'http://alasky.unistra.fr/MocServer/query',
            data: {ID: '*' + hipsId + '*', fmt: 'json', get: 'properties', dataproduct_type: 'image', casesensitive: 'false'},
            method: 'GET',
            dataType: 'json',
            success: function(result) {
                if (result.length==0) {
                    callback(null);
                }
                else if (result.length==1) {
                    callback(result[0]);
                }
                else {
                    console.log('Warning, multiple HiPS match the requested ID, returning first one');
                    callback(result[0]);
                }
            },
            error: function() {
                callback(null);
            }
        });
    };

    function getAlaskyServiceURL(hipsProperties) {
        if (hipsProperties.hasOwnProperty('hips_service_url') && hipsProperties.hips_service_url.indexOf('alasky')>0) {
            return hipsProperties.hips_service_url;
        }
        if (hipsProperties.hasOwnProperty('hips_service_url_1') && hipsProperties.hips_service_url_1.indexOf('alasky')>0) {
            return hipsProperties.hips_service_url_1;
        }
        if (hipsProperties.hasOwnProperty('hips_service_url_2') && hipsProperties.hips_service_url_2.indexOf('alasky')>0) {
            return hipsProperties.hips_service_url_2;
        }

        return hipsProperties.hips_service_url;
    }

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

        var survey = getURLParam('survey');
        var fov = getURLParam('fov') || 3;
        if (isNaN(fov)) {
            fov = 3;
        }
        var defaultTarget = 'NGC 2024';
        var target = getURLParam('target') || 'NGC 2024';
        var aladinParams = {fov: fov, target: target, showGotoControl:false, showFullscreenControl: false};
        if (!survey) {
            aladinParams.survey = 'P/DSS2/color';
        }
        else if (survey!==null && HpxImageSurvey.getSurveyFromId(survey)!==null) {
            aladinParams.survey = survey;
        }
        aladin = A.aladin('#aladin-lite-div', aladinParams);
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

        if (survey && HpxImageSurvey.getSurveyFromId(survey)==null) {
            var hipsId = survey;
            propertiesDictFromHiPSId(hipsId, function(hipsProperties) {
                if (hipsProperties===null || curSurveyId!=hipsId) {
                    console.error('Unknown HiPS ' + hipsId);
                    return;
                }
                var p = hipsProperties;
                var imgFormat = 'jpg';
                if (p.hasOwnProperty('hips_tile_format') && p.hips_tile_format.indexOf('png')>=0) {
                    imgFormat = 'png';
                }
                var hips_url = getAlaskyServiceURL(p);
                aladin.setImageSurvey(new HpxImageSurvey(p.ID, p.obs_title, hips_url, p.hips_frame || 'equatorial', p.hips_order, {imgFormat: imgFormat}));

                curSurveyId = p.ID;
                updateHistory();
            });
        }


    });

    function setSize() {
        var width = $(window).width();
        var maxWidth  = 2000;
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
             data.push();
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
             if (currentSurveyDiv.length === 0) {
                 return;
             }

             currentSurveyDiv.find('.survey-selected').show();
             // scroll to current survey if needed
             var shiftY = currentSurveyDiv.position().top - $('.surveyDiv').position().top;
             if (shiftY>400) {
                $('.surveyDiv').animate({scrollTop: shiftY});
             }
         },
         error: function() { $('#surveys').html("Error: "+url); }
     });



