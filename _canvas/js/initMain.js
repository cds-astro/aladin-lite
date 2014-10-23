// Author : Thomas Boch
var increment = 1;
var canvas;
var blackboard, blackboard2;
var stats;
var textures;
var textureSize;
var redrawAfterDragging;
var forceRedraw = false;

var view;

$(document).ready(function() {
	canvas = $("#canvas")[0];
    if (canvas.getContext) {
        ctx = canvas.getContext("2d");
    }
    
    var view = new Aladin.View(ctx)
    
    stats = new Stats();
    stats.domElement.style.position = 'absolute';
    stats.domElement.style.top = '50px';
    $('#statsDiv')[0].appendChild( stats.domElement );
	
	$(document).keydown(function(event) {
	    if (event.keyCode == 37) {// left
		    lon0 -= increment;
            }
	    else if (event.keyCode == 39) { // right
	        lon0 += increment;
            }
	    else if (event.keyCode == 38) { // up
	        lat0 -= increment;
            }
	    else if (event.keyCode == 40) { // down
	        lat0 += increment;
            }
	});

        draw();
        updateTextures();	
	
 });

    var surveys = {
                  "Mellinger": "textures/allsky/mellinger.jpg",
                  "DSS2F": "textures/allsky/dss2f.jpg",
                  "IRASColor": "textures/allsky/iras-color.jpg"
                  };

    // load textures
    function updateTextures() {
        var img = new Image();
        img.onload = function() {
                textures = new Array();
                // sur ipad, le fichier qu'on récupère est 2 fois plus petit. Il faut donc déterminer la taille de la texture dynamiquement
                textureSize = img.width/27;
                
        
                // récupération des 768 textures (NSIDE=4)
                
                        for (var j=0; j<29; j++) {
                                for (var i=0; i<27; i++) {
                                var c = document.createElement('canvas');
                                c.width = c.height = textureSize;
                                var context = c.getContext('2d');
                                context.drawImage(img, i*textureSize, j*textureSize, textureSize, textureSize, 0, 0, textureSize, textureSize);
                            textures.push(c);
                        }
                }
                forceRedraw = true;   
                // quand les textures ont été chargées, on peut commencer à dessiner
            doDraw();
        
                
        }
        img.src = surveys[$('#selectSurvey').val()];


    }

/**
 * Map a texture on a 4-points polygon
 * (code taken from http://stackoverflow.com/questions/4774172/image-manipulation-in-javascript-html5-canvas/4774298#4774298 )
 *
 * @param ctx the canvas context where we draw
 * @param texture the texture to be applied
 * @param pts the 4 corners where we want to map the texture.
 *              Each corner is an object with fields x,y,u,v
 *              where x,y are pixel coordinates on the target canvas
 *              and u,v are pixel coordinates on texture
 */
function textureMap(ctx, texture, pts) {
    var tris = [[0, 1, 2], [2, 3, 0]]; // Split in two triangles
    for (var t=0; t<2; t++) {
        var pp = tris[t];
        var x0 = pts[pp[0]].x, x1 = pts[pp[1]].x, x2 = pts[pp[2]].x;
        var y0 = pts[pp[0]].y, y1 = pts[pp[1]].y, y2 = pts[pp[2]].y;
        var u0 = pts[pp[0]].u, u1 = pts[pp[1]].u, u2 = pts[pp[2]].u;
        var v0 = pts[pp[0]].v, v1 = pts[pp[1]].v, v2 = pts[pp[2]].v;

        // Set clipping area so that only pixels inside the triangle will
        // be affected by the image drawing operation
        ctx.save(); ctx.beginPath(); ctx.moveTo(x0, y0); ctx.lineTo(x1, y1);
        ctx.lineTo(x2, y2); ctx.closePath(); ctx.clip();

        // Compute matrix transform
        var delta = u0*v1 + v0*u2 + u1*v2 - v1*u2 - v0*u1 - u0*v2;
        var delta_a = x0*v1 + v0*x2 + x1*v2 - v1*x2 - v0*x1 - x0*v2;
        var delta_b = u0*x1 + x0*u2 + u1*x2 - x1*u2 - x0*u1 - u0*x2;
        var delta_c = u0*v1*x2 + v0*x1*u2 + x0*u1*v2 - x0*v1*u2
                      - v0*u1*x2 - u0*x1*v2;
        var delta_d = y0*v1 + v0*y2 + y1*v2 - v1*y2 - v0*y1 - y0*v2;
        var delta_e = u0*y1 + y0*u2 + u1*y2 - y1*u2 - y0*u1 - u0*y2;
        var delta_f = u0*v1*y2 + v0*y1*u2 + y0*u1*v2 - y0*v1*u2
                      - v0*u1*y2 - u0*y1*v2;

        // Draw the transformed image
        ctx.transform(delta_a/delta, delta_d/delta,
                      delta_b/delta, delta_e/delta,
                      delta_c/delta, delta_f/delta);
        ctx.drawImage(texture, 0, 0);
        ctx.restore();
    }
}

function grow(b, val)  {

  var b1 = new Array(b.length);
  for( var i=0; i<4; i++ ) {
	  b1[i] = {"x": b[i].x, "y": b[i].y, "u": b[i].u, "v": b[i].v};
  }

  for( var i=0; i<2; i++ ) {
     var a= i==1 ? 1 : 0;
     var c= i==1 ? 2 : 3;

     var angle = Math.atan2(b1[c].y-b1[a].y, b1[c].x-b1[a].x);
     var chouilla = val*Math.cos(angle);
     b1[a].x -= chouilla;
     b1[c].x += chouilla;
     chouilla = val*Math.sin(angle);
     b1[a].y -= chouilla;
     b1[c].y += chouilla;
  }
  return b1;
}





var colorValues = ["#9bb2ff", "#9bb2ff", "#9eb5ff", "#a3b9ff", "#aabfff", "#b2c5ff", "#bbccff", "#c4d2ff", "#ccd8ff ", "#d3ddff", "#dae2ff", "#dfe5ff", "#e4e9ff", "#e9ecff", "#eeefff", "#f3f2ff", "#f8f6ff", "#fef9ff", "#fff9fb", "#fff7f5", "#fff5ef", "#fff3ea", "#fff1e5", "#ffefe0", "#ffeddb", "#ffebd6", "#ffe9d2", "#ffe8ce", "#ffe6ca", "#ffe5c6", "#ffe3c3", "#ffe2bf", "#ffe0bb", "#ffdfb8", "#ffddb4", "#ffdbb0", "#ffdaad", "#ffd8a9", "#ffd6a5", "#ffd5a1", "#ffd29c", "#ffd096", "#ffcc8f", "#ffc885", "#ffc178", "#ffb765", "#ffa94b", "#ff9523", "#ff7b00", "#ff5200"];
var colorLimits = [-0.4, -0.35, -0.3, -0.25, -0.2, -0.15, -0.1, -0.05, 0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 1, 1.05, 1.1, 1.15, 1.2, 1.25, 1.3, 1.35, 1.4, 1.45, 1.5, 1.55, 1.6, 1.65, 1.7, 1.75, 1.8, 1.85, 1.9, 1.95, 2];
function colorFromB_V(bv) {
    if (bv<colorLimits[0] ) {
        return colorValues[0];
    }

    for (var i=0; i<colorLimits.length-1; i++) {
        if (bv>=colorLimits[i] && bv<colorLimits[i+1]) return colorValues[i+1];
    }

    return colorValues[colorValues.length-1]
}

function sizeFromVMag(mag) {
    if (mag>5) return 0.4;
    if (mag>4) return 0.7;
    if (mag>3) return 1;
    if (mag>2) return 1.5;
    if (mag>1) return 2;
    if (mag>0) return 2.5;

    return 2.9;
}


    function cooToXY(s, lon0, lat0, lon, lat, radius) {
        if (s && s.vmag>5.5) return;

        return sinusProj_cooToXY(lon0, lat0, lon, lat, radius);
    }
    
    function sinusProj_cooToXY(lon0, lat0, lon, lat, radius) {
    	var cosc = Math.sin(lat0)*Math.sin(lat)+Math.cos(lat0)*Math.cos(lat)*Math.cos(lon-lon0)
        if ( cosc<0 ) {
            return null;
            }
        var x = -radius*Math.cos(lat)*Math.sin(lon-lon0)
        var y = -radius*(Math.cos(lat0)*Math.sin(lat)-Math.sin(lat0)*Math.cos(lat)*Math.cos(lon-lon0))
        return [x, y];
    }

    var dragging = false;
    var displayBgWhenDragging = true;
    var lon0 = 120;
    var lat0 = 30;
    var lon0deg, lat0deg;
    var intervalDelay = 80; // temps en ms entre 2 rafraichissements
    var intervalId;
    var drawtextures = true;
    var drawgrid = true;
    var drawconst = true;
    var drawconstname = true;
    var drawstars = true;
    var odrawgrid = !drawgrid;
    var odrawconst = !drawconst;
    var odrawconstname = !drawconstname
    var odrawstars = !drawstars
    var odrawtextures = !drawtextures;
    var olon0 = 999
    var olat0 = 999
    var ctx;
    var width;
    var height;
    var cx;
    var cy;
    var radius;
    var dragx=null;
    var dragy=null;

    function draw() {
        // adding listeners to checkboxes    
        $("#cbGrid").change(function() {
            drawgrid = $(this).is(':checked');
            doDraw();
        });
        $("#cbConst").change(function() {
            drawconst = $(this).is(':checked');
            doDraw();
        });
        $("#cbConstName").change(function() {
            drawconstname = $(this).is(':checked');
            doDraw();
        });
        $("#cbStars").change(function() {
            drawstars = $(this).is(':checked');
            doDraw();
        });
        $("#cbTextures").change(function() {
            drawtextures = $(this).is(':checked');
            doDraw();
        });

        $("#cbDisplayBgWhenDragging").change(function() {
            displayBgWhenDragging = $(this).is(':checked');
        });


        $("#selectSurvey").change(function() {
            updateTextures();
        });
        
        
        $("#canvas").mousedown(function(e) {
            dragx = e.clientX;
            dragy = e.clientY;
            dragging = true;
        });
        $("#canvas").mouseup(function(e) {
            dragx = dragy = null;   
            dragging = false;
            redrawAfterDragging = true;
        });
        $("#canvas").mousemove(function(e) {
            if (!dragging) {
            	return;
            }

            var xoffset = e.clientX-dragx;
            var yoffset = e.clientY-dragy;
            var dist = xoffset*xoffset+yoffset*yoffset;
            if (dist<5) return;
            dragx = e.clientX;
            dragy = e.clientY;

            lon0 += xoffset*0.2;
            lat0 += yoffset*0.2;
        });
        
        $("#canvas").bind("touchstart", function(e) {
        	e.preventDefault();
        	e = e.originalEvent;
        	
            dragx = e.targetTouches[0].clientX;
            dragy = e.targetTouches[0].clientY;
            dragging = true;
        });
        $("#canvas").bind("touchend", function(e) {
            e.preventDefault();

            dragx = dragy = null;   
            dragging = false;
        });
        $("#canvas").bind("touchmove", function(e) {
            e.preventDefault();
        	
            if (!dragging) {
                return;
            }
            
            e = e.originalEvent;
            

            var xoffset = e.targetTouches[0].clientX-dragx;
            var yoffset = e.targetTouches[0].clientY-dragy;
            var dist = xoffset*xoffset+yoffset*yoffset;
            if (dist<5) return;
            dragx = e.targetTouches[0].clientX;
            dragy = e.targetTouches[0].clientY;

            lon0 += xoffset*0.2;
            lat0 += yoffset*0.2;
        });

        

        width = canvas.width;
        height = canvas.height;
        cx = width/2;
        cy = height/2;
        radius = 300;
    }

    function doDraw() {
    	requestAnimationFrame(doDraw);
    	stats.update();

        
        if (   olon0==lon0 && olat0==lat0 && odrawgrid==drawgrid && odrawconst==drawconst
            && odrawconstname==drawconstname && odrawstars==drawstars && odrawtextures==drawtextures && ! redrawAfterDragging && !forceRedraw) {
            olon0=lon0;
            olat0=lat0;
            odrawgrid=drawgrid;
            odrawconst=drawconst;
            odrawconstname=drawconstname;
            odrawstars=drawstars;
            odrawtextures=drawtextures;

            return;

        }

       
        redrawAfterDragging = false; 

            //ctx.save();
            ctx.clearRect(0,0,width,height);


            lon0deg = lon0*Math.PI/180
            lat0deg = lat0*Math.PI/180

        ctx.fillStyle = "rgb(0,0,0)";
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, 2*Math.PI, true);
        ctx.fill();
        //ctx.restore();

        if( displayBgWhenDragging || !dragging) {
            drawTextures();
        }
        
        drawConstellations();
        drawConstellationsNames();

        drawStars();

        drawGrid();

        olon0=lon0;
        olat0=lat0;
        odrawgrid=drawgrid;
        odrawconst=drawconst;
        odrawconstname=drawconstname;
        odrawstars=drawstars;
        odrawtextures=drawtextures;
        forceRedraw = false;
    }

    function drawTextures() {
    	if (!drawtextures) {
    		return;
    	}
    	
    	for (var i=0; i<768; i++) {
    	   drawOneTexture(i);
    	}
    }
    
    var xytmp = new Array(4);
    var ptstmp = new Array(4);
    function drawOneTexture(ipix) {
        for (var i=0; i<4; i++) {
        	var tmp = cooToXY(null, lon0deg, lat0deg, hpxCoo[ipix][i].ra*Math.PI/180, hpxCoo[ipix][i].dec*Math.PI/180, radius);
        	
        	if (tmp) {
        		xytmp[i] = tmp;
                //xy.push(tmp);	
        	}
        	else {
        		return;
        	}
        }
        
/*
        pts.push({"x": xy[0][0]+cx, "y": xy[0][1]+cy, "u": 64, "v": 0});
        pts.push({"x": xy[1][0]+cx, "y": xy[1][1]+cy, "u": 0, "v":  0});
        pts.push({"x": xy[2][0]+cx, "y": xy[2][1]+cy, "u": 0, "v": 64});
        pts.push({"x": xy[3][0]+cx, "y": xy[3][1]+cy, "u": 64, "v": 64});
        */
        
        ptstmp[0] = {"x": xytmp[0][0]+cx, "y": xytmp[0][1]+cy, "u": textureSize-2, "v": textureSize-2};
        ptstmp[1] = {"x": xytmp[1][0]+cx, "y": xytmp[1][1]+cy, "u": textureSize-1, "v":  0};
        ptstmp[2] = {"x": xytmp[2][0]+cx, "y": xytmp[2][1]+cy, "u": 0, "v": 0};
        ptstmp[3] = {"x": xytmp[3][0]+cx, "y": xytmp[3][1]+cy, "u": 0, "v": textureSize-1};
        
        //ptstmp = grow(ptstmp, 3);


        textureMap(ctx, textures[ipix], ptstmp);
    }
    
    function drawStars() {
    if (!drawstars) return;

    //ctx.save();
    for (var i=stars.length-1; i>=0; i--) {
            var s = stars[i];
        xy = cooToXY(s, lon0deg, lat0deg, s.ra*Math.PI/180, s.dec*Math.PI/180, radius);
        if (xy) {
        ctx.beginPath();
                ctx.fillStyle = colorFromB_V(s.bv);
            ctx.arc(cx+xy[0], cy+xy[1], sizeFromVMag(s.vmag), 0, 2*Math.PI, true);
            ctx.fill();

        // test pour rendre l'aspect blurry des étoiles
        /*
        ctx.beginPath();
                ctx.fillStyle = "rgba(255, 255, 255, 0.5)";
            ctx.arc(cx+xy[0], cy+xy[1], 2*sizeFromVMag(s.vmag), 0, 2*Math.PI, true);
        ctx.fill();
        */
        }
    }
    //ctx.restore();
    }

    function drawConstellations() {
        if (!drawconst) return;

        //ctx.save();
        ctx.strokeStyle = "rgba(100,100,200, 0.5)";
        ctx.lineWidth = 2;
        ctx.beginPath();

        var l1, l2;
        var c;
        for (var i=0; i<constellations.length; i++) {
        c = constellations[i];
                for (var j=0; j<c.lines.length; j++) {
            l1 = cooToXY(null, lon0deg, lat0deg, c.lines[j][0]*Math.PI/180, c.lines[j][1]*Math.PI/180, radius);
            l2 = cooToXY(null, lon0deg, lat0deg, c.lines[j][2]*Math.PI/180, c.lines[j][3]*Math.PI/180, radius);
            if (l1 && l2) {
            ctx.moveTo(cx+l1[0], cy+l1[1]);
            ctx.lineTo(cx+l2[0], cy+l2[1]);

            }

        }

        }
        ctx.stroke();
        //ctx.restore();


    }

    function drawConstellationsNames() {
        if (!drawconstname) return;

        // testing existence of function
        if (!ctx.fillText) return;

        //ctx.save();
        ctx.beginPath();
        ctx.fillStyle = "rgba(230,120,250, 0.5)";
        ctx.textWidth = 2.5;
        var c;
        for (var i=0; i<constellations.length; i++) {
        c = constellations[i];
        xy = cooToXY(null, lon0deg, lat0deg, c.namera*Math.PI/180.0, c.namedec*Math.PI/180.0, radius);
        if (xy) {
            ctx.fillText(c.name, cx+xy[0], cy+xy[1])
        }
        }

        ctx.stroke();
        //ctx.restore();

    }

    function drawGrid() {
        if (!drawgrid) return;

        // tracé grille
        //ctx.save();
        ctx.strokeStyle = "rgba(100,200,100, 0.5)";
        ctx.lineWidth = 1;
        ctx.fillStyle = "white";
        ctx.beginPath();
        var ox, oy; 
        var raIdx=-999;
        for (var ra=0; ra<360; ra+=20) {
        ox = oy = null;
            for (var dec=-90; dec<=90; dec=dec+5) {
            xy = cooToXY(null, lon0deg, lat0deg, ra*Math.PI/180.0, dec*Math.PI/180, radius);
            if (xy && ox && oy) {
                ctx.moveTo(cx+ox, cy+oy);
                ctx.lineTo(cx+xy[0], cy+xy[1]);
            }
            if (xy && xy[0]>60 && xy[0]<155 && xy[1]>-65 && xy[1]<65 && raIdx<0) {
                raIdx = ra;
            }



            if (xy) {
                ox = xy[0];
            oy = xy[1];
            }
            else {
            ox = oy = null;
            }
            }
        xy = cooToXY(null, lon0deg, lat0deg, ra*Math.PI/180.0, 0, radius);
        if (xy && ctx.fillText) {
            ctx.fillText(ra, cx+xy[0], cy+xy[1])
        }
            }

        
        for (var dec=-90; dec<=90; dec+=20) {
        ox = oy = null;
        var gg = cooToXY(null, lon0deg, lat0deg, raIdx*Math.PI/180.0, dec*Math.PI/180, radius);
        if (gg && ctx.fillText) ctx.fillText(dec, cx+gg[0], cy+gg[1])
            for (var ra=0; ra<=380; ra=ra+5) {
                xy = cooToXY(null, lon0deg, lat0deg, ra*Math.PI/180.0, dec*Math.PI/180, radius);
                if (xy && ox && oy) {
                    ctx.moveTo(cx+ox, cy+oy);
                    ctx.lineTo(cx+xy[0], cy+xy[1]);
                    }
                if (xy) {
                    ox = xy[0];
                    oy = xy[1];
                    }
                else {
                    ox = oy = null;
                }
            }
        }
        
        
        ctx.stroke();
        //ctx.restore();
    }
    