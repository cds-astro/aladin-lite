
     function propertiesDictFromHiPSId(hipsId, callback) {
        if (! callback) {
            return;
        }

        if (/http/i.test(hipsId)) { // if we have a URL
            HiPSDefinition.fromURL(hipsId, function(definition) {
                if (definition !== null && definition.properties) {
                    callback(definition.properties);
                }
                else {
                    console.log('HiPS properties is ' + definition + ', can not load HiPS');
                }
            });

            return;
        }

        $.ajax({
            url: 'https://alaskybis.unistra.fr/MocServer/query',
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
        var aladinParams = {fov: fov, target: target, showGotoControl:true, showFullscreenControl: true, showSimbadPointerControl: true, showShareControl: true, realFullscreen: true};

        if (!survey) {
            aladinParams.survey = curSurveyId = 'P/DSS2/color';
        }
        else if (survey!==null && HpxImageSurvey.getSurveyFromId(survey)!==null) {
            aladinParams.survey = survey;
        }
        aladin = A.aladin('#aladin-lite-div', aladinParams);
        // change link on Aladin Lite logo to point to project page
        $('.aladin-logo-container a').attr('href', 'http://aladin.u-strasbg.fr/');
        // hide goto control for now
        $('.aladin-gotoControl-container').hide();
        // listen to fullScreen toggle
        aladin.on('fullScreenToggled', function(fullScreen) {
            if (fullScreen) {
                $('.aladin-gotoControl-container').show();
            }
            else {
                $('.aladin-gotoControl-container').hide();
            }
        });

        // build layers control box
        var hipsCatsDesc = [
            {id: 'simbad', name: 'SIMBAD', url: 'https://axel.u-strasbg.fr/HiPSCatService/SIMBAD', options: {shape: 'circle', sourceSize: 8, color: '#318d80'}},
            {id: 'gaia-dr2', name: 'Gaia DR2', url: 'https://axel.u-strasbg.fr/HiPSCatService/I/345/gaia2', options: {shape: 'square', sourceSize: 8, color: '#6baed6'}},
            {id: '2mass', name: '2MASS', url: 'https://axel.u-strasbg.fr/HiPSCatService/II/246/out', options: {shape: 'plus', sourceSize: 8, color: '#dd2233'}},
        ];
        var hipsCats = {};
        var layersControl = aladin.box({title: 'Catalogues', position: 'right', css: {top: '35%', 'overflow-y': 'scroll', 'max-height': '50%'}});

        var html = '<form id="hipsCatsSelector">';
        for (var k=0 ; k<hipsCatsDesc.length; k++) {
            var cat = hipsCatsDesc[k];
            html += '<fieldset><span class="indicator right-triangle"></span><label for="hipscat-' + cat.id + '">';
            html += '<input id="hipscat-' + cat.id + '" type="checkbox" value="' + cat.id + '" >' + cat.name + '</input></label>';
            html += '<div class="cat-options" style="display: none;"><table><tr><td>Color</td><td><input type="color"></input></td></tr><tr><td>Size</td><td><input type="range" min="6" max="20" step="2"></input></td></tr><tr><td>Shape</td><td><select><option value="square">&#9633;</option><option value="circle">&#9675;</option><option value="plus">&#10133;</option><option value="cross">&#10005;</option><option value="triangle">&#9651;</option><option value="rhomb">&#8415;</option></select></td></tr></table></div>';
            html += '</fieldset>';

        }
        for (var k=hipsCatsDesc.length -1 ; k>=0; k--) {
            var cat = hipsCatsDesc[k];
            var hips = A.catalogHiPS(cat.url, {id: cat.id, name: cat.name, shape: cat.options.shape, sourceSize: cat.options.sourceSize, color: cat.options.color});
            hips.hide();
            aladin.addCatalog(hips);
            hipsCats[cat.id] = hips;
        }
        html += '</form>';


        layersControl.setContent(html);
        layersControl.show();

        $('#hipsCatsSelector :checkbox').change(function() {
            var hips = hipsCats[this.value];

            if (this.checked) {
                hips.show();
            }
            else {
                hips.hide();
            }
        });

        $("#hipsCatsSelector input[type='color']").change(function() {
            var hipsId = $(this).parents('fieldset').find("input[type='checkbox']").val();
            hipsCats[hipsId].updateShape({color: this.value});
        });

        $("#hipsCatsSelector input[type='range']").change(function() {
            var hipsId = $(this).parents('fieldset').find("input[type='checkbox']").val();
            hipsCats[hipsId].updateShape({sourceSize: parseInt(this.value)});
        });

        $("#hipsCatsSelector select").change(function() {
            var hipsId = $(this).parents('fieldset').find("input[type='checkbox']").val();
            hipsCats[hipsId].updateShape({shape: this.value});
        });

        $('.indicator').click(function() {
            var $this = $(this);
            if ($this.hasClass('right-triangle')) {
                $this.removeClass('right-triangle');
                $this.addClass('down-triangle');
                $this.parent().find('.cat-options').slideDown(300);
                var hipsId = $(this).parent().find("input[type='checkbox']").val();
                $this.parent().find("input[type='color']").val(hipsCats[hipsId].color);
                $this.parent().find("input[type='range']").val(hipsCats[hipsId].sourceSize);
                $this.parent().find("select").val(hipsCats[hipsId].shape);
            }
            else {
                $this.removeClass('down-triangle');
                $this.addClass('right-triangle');
                $this.parent().find('.cat-options').slideUp(300);
            }
        });

        // **** box to display details ****
        var curSelectedSource = null;
        var detailsBox = aladin.box({ position: 'left', css: {top: '35%', 'overflow-y': 'scroll', 'max-height': '50%'} });
        detailsBox.realHide();
        //var detailsBox = aladin.box({ position: 'left', css: {top: '35%'}, contentCss: {'overflow-y': 'scroll', 'max-height': '50%'} });
        // listen to click on objects
        aladin.on('objectClicked', function(source) {
            var html = '<table class="object-details-table">';
            if (curSelectedSource != null) {
                curSelectedSource.deselect();
            }
            if (source==null) {
                detailsBox.setContent('');
                detailsBox.realHide();
                return;
            }

            source.select();
            curSelectedSource = source;

            detailsBox.setTitle(source.catalog.name + ' source');
            html += '<tbody>';
            for ( key in source.data) {
                html += '<tr><td><b>' + key + '</b></td><td>' + source.data[key] + '</td></tr>';
            }
            html += '</tbody></table>';

            detailsBox.setContent(html);
            detailsBox.show();
        });


        // **** box to display thumbnails preview ****
        var recreateCarousel = function(ra, dec, fov) {
            thumbnailsBox.setContent('<div id="thumbnails-div" />');
            $('#thumbnails-div').empty();
            $('#thumbnails-div').slick({
                slidesToShow: 3,
                slidesToScroll: 3,
                adaptiveHeight: true,
                lazyLoad: 'ondemand',
                dots: true
            });
            var alWidth = aladin.getSize()[0];
            var alHeight = aladin.getSize()[1];
            var imgWidth = parseInt(alWidth / 6);
            var imgHeight = parseInt(alHeight / 6);
            $.ajax({
                url: "https://alaskybis.u-strasbg.fr/MocServer/query",
                    data: {dataproduct_type: 'image', client_application: 'AladinLite', fmt: 'json', RA: ra, DEC: dec, SR: fov, expr: '(ID=CDS* ||  hips_service_url=*) && hips_refpos!=* && ID!=CDS/Outreach* && ID!=JAXA/P/CONSTELLATIONS*', get: 'record'},
                    method: 'GET',
                    dataType: 'json',
                    success: function(result) {
                        result.sort(function(a, b) {
                            var a_em_avg = (parseFloat(a.em_min) + parseFloat(a.em_max)) / 2;
                            var b_em_avg = (parseFloat(b.em_min) + parseFloat(b.em_max)) / 2;

                            if (isNaN(a_em_avg)) {
                                a_em_avg = 0;
                            }
                            if (isNaN(b_em_avg)) {
                                b_em_avg = 0;
                            }

                            return a_em_avg - b_em_avg;
                        });
                        for (var k=0; k<result.length; k++) {
                            var hips = result[k];
                            var label = hips.ID.split('/').slice(1).join('/').replace("/color","").replace("/Color","").replace("P/","");
                            var surveyId = hips.ID.split('/').slice(1).join('/');
                            $('#thumbnails-div').slick('slickAdd', '<div><div class="thumbnail-label">' + label + '</div><img class="thumbnail-img" data-surveyId="' + surveyId + '" data-lazy="https://alasky.unistra.fr/hips-thumbnails/thumbnail?ra=' + ra + '&dec=' + dec + '&fov=' + fov + '&width=' + imgWidth + '&height=' + imgHeight + '&hips_kw=' + encodeURIComponent(hips.ID) + '" width=' + imgWidth + ' height= ' + imgHeight + ' /></div>');
                        }
                        // update HiPS when clicking on one thumbnail
                        $('.thumbnail-img').click(function() {
                            curSurveyId = $(this).attr('data-surveyId');
                            setSurvey(curSurveyId);
                            //updateHistory();
                            $('.survey-selected').hide();
                            // show green tick for current survey
                            $('.survey[data-surveyId="' + curSurveyId + '"]').find('.survey-selected').show();
                        });
                    }
            });
        };
        var thumbnailsUpdateInterval;
        var viewParams = {ra: null, dec: null, fov: null, timeParamsChanged: null};
        var curThumbnailsParams = {ra: null, dec:null, fov:null};
        var updateThumbnailsIfNeeded = function() {
            var raDec = aladin.getRaDec();
            var ra = raDec[0];
            var dec = raDec[1];
            var fov = aladin.getFov()[0];
            var timeNow = new Date().getTime();

            var recreate = false;

            if (viewParams.timeParamsChanged==null) {
                recreate = true;
                viewParams.timeParamsChanged = timeNow;
            }
            if (ra !== viewParams.ra || dec !== viewParams.dec || fov !== viewParams.fov) {
                viewParams.timeParamsChanged = timeNow;
            }
            viewParams.ra  = ra;
            viewParams.dec = dec;
            viewParams.fov = fov;




            if ((timeNow - viewParams.timeParamsChanged > 2500) &&
                    (ra !== curThumbnailsParams.ra ||
                    dec!==curThumbnailsParams.dec ||
                    fov!==curThumbnailsParams.fov) ) {
                recreate = true;
            }
           
            if (recreate) {
                curThumbnailsParams = {ra: ra, dec:dec, fov:fov};
                recreateCarousel(ra, dec, fov);
            } 
        };
        var openCallback = function() {
            updateThumbnailsIfNeeded();
            thumbnailsUpdateInterval = setInterval(updateThumbnailsIfNeeded, 1000);
        };
        var closeCallback = function() {
            clearInterval(thumbnailsUpdateInterval);
        };
        var thumbnailsBox = aladin.box({ position: 'bottom', title: 'Thumbnails', css: {left: '20%', width: '60%'}, openCallback: openCallback, closeCallback: closeCallback });
        thumbnailsBox.open = true;
        thumbnailsBox.hide();




        
        


        // *******************************************



        if (survey) {
            curSurveyId = survey;
        }

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
                //updateHistory();
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
       //updateHistory();
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
            if (surveys[k].ID.indexOf(s)>=0) {
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
        $('#content').html(s1.obs_title+" - <a href=\""+s1.obs_copyright_url+"\">"+s1.obs_copyright+"</a>");
    }

    function setSurvey(s) {
        aladin.setImageSurvey(s);
        setInfo(s);
     }
  

    $.ajax({
         url: "https://alaskybis.u-strasbg.fr/MocServer/query",
         data: {dataproduct_type: 'image', client_application: 'AladinLite', fmt: 'json', fields: 'ID,obs_title,client_category,client_sort_key,client_application,hips_service_url*,hips_order,hips_tile_format,hips_frame,obs_copyright,obs_copyright_url,em_min,em_max'},
         method: 'GET',
         dataType: 'json',
         success: function(data) {
             var tooltipDescriptions = {};
             var res = '<div class="surveys-list">';
             data.sort(function(a, b) {
                var a_em_avg = (parseFloat(a.em_min) + parseFloat(a.em_max)) / 2;
                var b_em_avg = (parseFloat(b.em_min) + parseFloat(b.em_max)) / 2;

                if (isNaN(a_em_avg)) {
                    a_em_avg = 0;
                }
                if (isNaN(b_em_avg)) {
                    b_em_avg = 0;
                }

                return a_em_avg - b_em_avg;
             });
             surveys = data;

             
             for (var k=0; k<data.length; k++) {
                  var id = data[k].ID;
                  if (id=='xcatdb/P/XMM/PN/color') {
                      data[k].client_category = 'Image/X';
                  }
                  var w = /^\w+\/(\w+)/.exec(data[k].client_category)[1];
                  id = id.split('/').slice(1).join('/');
                  var s1 = id.replace("/color","");
                  s1 = s1.replace("/Color","");
                  s1 = s1.replace("P/","");
                  var imgPath = 'survey-previews/' + id.replace(/\//g, "_") + '.jpg';
                  res += '<div class="survey" data-surveyId="' + id + '"><div class="survey-label">' + s1 + '</div><img class="survey-preview" src="' + imgPath + '" /><div class="survey-selected" style="display: none;"><div class="survey-selected-img"></div></div></div>';
                  tooltipDescriptions[id] = '<div>Band: ' + w + '</div><div>' + data[k].obs_title + '</div>';
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
                //updateHistory();
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
             if (shiftY>300) {
                $('.surveyDiv').animate({scrollTop: shiftY});
             }
         },
         error: function() { $('#surveys').html("Error: "+url); }
     });



