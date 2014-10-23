// log 
Logger = {};

Logger.log = function(action) {
    var logUrl = "http://alasky.u-strasbg.fr/";

};
/*! Copyright (c) 2011 Brandon Aaron (http://brandonaaron.net)
 * Licensed under the MIT License (LICENSE.txt).
 *
 * Thanks to: http://adomas.org/javascript-mouse-wheel/ for some pointers.
 * Thanks to: Mathias Bank(http://www.mathias-bank.de) for a scope bug fix.
 * Thanks to: Seamus Leahy for adding deltaX and deltaY
 *
 * Version: 3.0.6
 * 
 * Requires: 1.2.2+
 */
(function(a){function d(b){var c=b||window.event,d=[].slice.call(arguments,1),e=0,f=!0,g=0,h=0;return b=a.event.fix(c),b.type="mousewheel",c.wheelDelta&&(e=c.wheelDelta/120),c.detail&&(e=-c.detail/3),h=e,c.axis!==undefined&&c.axis===c.HORIZONTAL_AXIS&&(h=0,g=-1*e),c.wheelDeltaY!==undefined&&(h=c.wheelDeltaY/120),c.wheelDeltaX!==undefined&&(g=-1*c.wheelDeltaX/120),d.unshift(b,e,g,h),(a.event.dispatch||a.event.handle).apply(this,d)}var b=["DOMMouseScroll","mousewheel"];if(a.event.fixHooks)for(var c=b.length;c;)a.event.fixHooks[b[--c]]=a.event.mouseHooks;a.event.special.mousewheel={setup:function(){if(this.addEventListener)for(var a=b.length;a;)this.addEventListener(b[--a],d,!1);else this.onmousewheel=d},teardown:function(){if(this.removeEventListener)for(var a=b.length;a;)this.removeEventListener(b[--a],d,!1);else this.onmousewheel=null}},a.fn.extend({mousewheel:function(a){return a?this.bind("mousewheel",a):this.trigger("mousewheel")},unmousewheel:function(a){return this.unbind("mousewheel",a)}})})(jQuery)
// requestAnimationFrame() shim by Paul Irish
// http://paulirish.com/2011/requestanimationframe-for-smart-animating/
window.requestAnimFrame = (function() {
	return  window.requestAnimationFrame       || 
			window.webkitRequestAnimationFrame || 
			window.mozRequestAnimationFrame    || 
			window.oRequestAnimationFrame      || 
			window.msRequestAnimationFrame     || 
			function(/* function */ callback, /* DOMElement */ element){
				window.setTimeout(callback, 1000 / 60);
			};
})();// stats.js r6 - http://github.com/mrdoob/stats.js
var Stats=function(){function s(a,g,d){var f,c,e;for(c=0;c<30;c++)for(f=0;f<73;f++)e=(f+c*74)*4,a[e]=a[e+4],a[e+1]=a[e+5],a[e+2]=a[e+6];for(c=0;c<30;c++)e=(73+c*74)*4,c<g?(a[e]=b[d].bg.r,a[e+1]=b[d].bg.g,a[e+2]=b[d].bg.b):(a[e]=b[d].fg.r,a[e+1]=b[d].fg.g,a[e+2]=b[d].fg.b)}var r=0,t=2,g,u=0,j=(new Date).getTime(),F=j,v=j,l=0,w=1E3,x=0,k,d,a,m,y,n=0,z=1E3,A=0,f,c,o,B,p=0,C=1E3,D=0,h,i,q,E,b={fps:{bg:{r:16,g:16,b:48},fg:{r:0,g:255,b:255}},ms:{bg:{r:16,g:48,b:16},fg:{r:0,g:255,b:0}},mb:{bg:{r:48,g:16,
b:26},fg:{r:255,g:0,b:128}}};g=document.createElement("div");g.style.cursor="pointer";g.style.width="80px";g.style.opacity="0.9";g.style.zIndex="10001";g.addEventListener("click",function(){r++;r==t&&(r=0);k.style.display="none";f.style.display="none";h.style.display="none";switch(r){case 0:k.style.display="block";break;case 1:f.style.display="block";break;case 2:h.style.display="block"}},!1);k=document.createElement("div");k.style.backgroundColor="rgb("+Math.floor(b.fps.bg.r/2)+","+Math.floor(b.fps.bg.g/
2)+","+Math.floor(b.fps.bg.b/2)+")";k.style.padding="2px 0px 3px 0px";g.appendChild(k);d=document.createElement("div");d.style.fontFamily="Helvetica, Arial, sans-serif";d.style.textAlign="left";d.style.fontSize="9px";d.style.color="rgb("+b.fps.fg.r+","+b.fps.fg.g+","+b.fps.fg.b+")";d.style.margin="0px 0px 1px 3px";d.innerHTML='<span style="font-weight:bold">FPS</span>';k.appendChild(d);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";k.appendChild(a);
m=a.getContext("2d");m.fillStyle="rgb("+b.fps.bg.r+","+b.fps.bg.g+","+b.fps.bg.b+")";m.fillRect(0,0,a.width,a.height);y=m.getImageData(0,0,a.width,a.height);f=document.createElement("div");f.style.backgroundColor="rgb("+Math.floor(b.ms.bg.r/2)+","+Math.floor(b.ms.bg.g/2)+","+Math.floor(b.ms.bg.b/2)+")";f.style.padding="2px 0px 3px 0px";f.style.display="none";g.appendChild(f);c=document.createElement("div");c.style.fontFamily="Helvetica, Arial, sans-serif";c.style.textAlign="left";c.style.fontSize=
"9px";c.style.color="rgb("+b.ms.fg.r+","+b.ms.fg.g+","+b.ms.fg.b+")";c.style.margin="0px 0px 1px 3px";c.innerHTML='<span style="font-weight:bold">MS</span>';f.appendChild(c);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";f.appendChild(a);o=a.getContext("2d");o.fillStyle="rgb("+b.ms.bg.r+","+b.ms.bg.g+","+b.ms.bg.b+")";o.fillRect(0,0,a.width,a.height);B=o.getImageData(0,0,a.width,a.height);try{performance&&performance.memory&&performance.memory.totalJSHeapSize&&
(t=3)}catch(G){}h=document.createElement("div");h.style.backgroundColor="rgb("+Math.floor(b.mb.bg.r/2)+","+Math.floor(b.mb.bg.g/2)+","+Math.floor(b.mb.bg.b/2)+")";h.style.padding="2px 0px 3px 0px";h.style.display="none";g.appendChild(h);i=document.createElement("div");i.style.fontFamily="Helvetica, Arial, sans-serif";i.style.textAlign="left";i.style.fontSize="9px";i.style.color="rgb("+b.mb.fg.r+","+b.mb.fg.g+","+b.mb.fg.b+")";i.style.margin="0px 0px 1px 3px";i.innerHTML='<span style="font-weight:bold">MB</span>';
h.appendChild(i);a=document.createElement("canvas");a.width=74;a.height=30;a.style.display="block";a.style.marginLeft="3px";h.appendChild(a);q=a.getContext("2d");q.fillStyle="#301010";q.fillRect(0,0,a.width,a.height);E=q.getImageData(0,0,a.width,a.height);return{domElement:g,update:function(){u++;j=(new Date).getTime();n=j-F;z=Math.min(z,n);A=Math.max(A,n);s(B.data,Math.min(30,30-n/200*30),"ms");c.innerHTML='<span style="font-weight:bold">'+n+" MS</span> ("+z+"-"+A+")";o.putImageData(B,0,0);F=j;if(j>
v+1E3){l=Math.round(u*1E3/(j-v));w=Math.min(w,l);x=Math.max(x,l);s(y.data,Math.min(30,30-l/100*30),"fps");d.innerHTML='<span style="font-weight:bold">'+l+" FPS</span> ("+w+"-"+x+")";m.putImageData(y,0,0);if(t==3)p=performance.memory.usedJSHeapSize*9.54E-7,C=Math.min(C,p),D=Math.max(D,p),s(E.data,Math.min(30,30-p/2),"mb"),i.innerHTML='<span style="font-weight:bold">'+Math.round(p)+" MB</span> ("+Math.round(C)+"-"+Math.round(D)+")",q.putImageData(E,0,0);v=j;u=0}}}};

Constants={},Constants.PI=Math.PI,Constants.C_PR=Math.PI/180,Constants.VLEV=2,Constants.EPS=1e-7,Constants.c=.105,Constants.LN10=Math.log(10),Constants.PIOVER2=Math.PI/2,Constants.TWOPI=2*Math.PI,Constants.TWOTHIRD=2/3,Constants.ARCSECOND_RADIAN=484813681109536e-20,SpatialVector=function(){function t(t,s,i){"use strict";this.x=t,this.y=s,this.z=i,this.ra_=0,this.dec_=0,this.okRaDec_=!1}return t.prototype.setXYZ=function(t,s,i){this.x=t,this.y=s,this.z=i,this.okRaDec_=!1},t.prototype.length=function(){"use strict";return Math.sqrt(this.lengthSquared())},t.prototype.lengthSquared=function(){"use strict";return this.x*this.x+this.y*this.y+this.z*this.z},t.prototype.normalized=function(){"use strict";var t=this.length();this.x/=t,this.y/=t,this.z/=t},t.prototype.set=function(t,s){"use strict";this.ra_=t,this.dec_=s,this.okRaDec_=!0,this.updateXYZ()},t.prototype.angle=function(t){"use strict";var s=this.y*t.z-this.z*t.y,i=this.z*t.x-this.x*t.z,n=this.x*t.y-this.y*t.x,a=Math.sqrt(s*s+i*i+n*n);return Math.abs(Math.atan2(a,dot(t)))},t.prototype.get=function(){"use strict";return[x,y,z]},t.prototype.toString=function(){"use strict";return"SpatialVector["+this.x+", "+this.y+", "+this.z+"]"},t.prototype.cross=function(s){"use strict";return new t(this.y*s.z-s.y*this.z,this.z*s.x-s.z*this.x,this.x*s.y-s.x()*this.y)},t.prototype.equal=function(t){"use strict";return this.x==t.x&&this.y==t.y&&this.z==t.z()?!0:!1},t.prototype.mult=function(s){"use strict";return new t(s*this.x,s*this.y,s*this.z)},t.prototype.dot=function(t){"use strict";return this.x*t.x+this.y*t.y+this.z*t.z},t.prototype.add=function(s){"use strict";return new t(this.x+s.x,this.y+s.y,this.z+s.z)},t.prototype.sub=function(s){"use strict";return new t(this.x-s.x,this.y-s.y,this.z-s.z)},t.prototype.dec=function(){"use strict";return this.okRaDec_||(this.normalized(),this.updateRaDec()),this.dec_},t.prototype.ra=function(){"use strict";return this.okRaDec_||(this.normalized(),this.updateRaDec()),this.ra_},t.prototype.updateXYZ=function(){"use strict";var t=Math.cos(this.dec_*Constants.C_PR);this.x=Math.cos(this.ra_*Constants.C_PR)*t,this.y=Math.sin(this.ra_*Constants.C_PR)*t,this.z=Math.sin(this.dec_*Constants.C_PR)},t.prototype.updateRaDec=function(){"use strict";this.dec_=Math.asin(this.z)/Constants.C_PR;var t=Math.cos(this.dec_*Constants.C_PR);this.ra_=t>Constants.EPS||-Constants.EPS>t?this.y>Constants.EPS||this.y<-Constants.EPS?0>this.y?360-Math.acos(this.x/t)/Constants.C_PR:Math.acos(this.x/t)/Constants.C_PR:0>this.x?180:0:0,this.okRaDec_=!0},t.prototype.toRaRadians=function(){"use strict";var t=0;return(0!=this.x||0!=this.y)&&(t=Math.atan2(this.y,this.x)),0>t&&(t+=2*Math.PI),t},t.prototype.toDeRadians=function(){var t=z/this.length(),s=Math.acos(t);return Math.PI/2-s},t}(),AngularPosition=function(){return AngularPosition=function(t,s){"use strict";this.theta=t,this.phi=s},AngularPosition.prototype.toString=function(){"use strict";return"theta: "+this.theta+", phi: "+this.phi},AngularPosition}(),LongRangeSetBuilder=function(){function t(){this.items=[]}return t.prototype.appendRange=function(t,s){for(var i=t;s>=i;i++)i in this.items||this.items.push(i)},t}(),HealpixIndex=function(){function t(t){"use strict";this.nside=t}return t.NS_MAX=8192,t.ORDER_MAX=13,t.NSIDELIST=[1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192],t.JRLL=[2,2,2,2,3,3,3,3,4,4,4,4],t.JPLL=[1,3,5,7,0,2,4,6,1,3,5,7],t.XOFFSET=[-1,-1,0,1,1,1,0,-1],t.YOFFSET=[0,1,1,1,0,-1,-1,-1],t.FACEARRAY=[[8,9,10,11,-1,-1,-1,-1,10,11,8,9],[5,6,7,4,8,9,10,11,9,10,11,8],[-1,-1,-1,-1,5,6,7,4,-1,-1,-1,-1],[4,5,6,7,11,8,9,10,11,8,9,10],[0,1,2,3,4,5,6,7,8,9,10,11],[1,2,3,0,0,1,2,3,5,6,7,4],[-1,-1,-1,-1,7,4,5,6,-1,-1,-1,-1],[3,0,1,2,3,0,1,2,4,5,6,7],[2,3,0,1,-1,-1,-1,-1,0,1,2,3]],t.SWAPARRAY=[[0,0,0,0,0,0,0,0,3,3,3,3],[0,0,0,0,0,0,0,0,6,6,6,6],[0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,5,5,5,5],[0,0,0,0,0,0,0,0,0,0,0,0],[5,5,5,5,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0],[6,6,6,6,0,0,0,0,0,0,0,0],[3,3,3,3,0,0,0,0,0,0,0,0]],t.Z0=Constants.TWOTHIRD,t.prototype.init=function(){"use strict";var s=256;this.ctab=Array(s),this.utab=Array(s);for(var i=0;256>i;++i)this.ctab[i]=1&i|(2&i)<<7|(4&i)>>1|(8&i)<<6|(16&i)>>2|(32&i)<<5|(64&i)>>3|(128&i)<<4,this.utab[i]=1&i|(2&i)<<1|(4&i)<<2|(8&i)<<3|(16&i)<<4|(32&i)<<5|(64&i)<<6|(128&i)<<7;this.nl2=2*this.nside,this.nl3=3*this.nside,this.nl4=4*this.nside,this.npface=this.nside*this.nside,this.ncap=2*this.nside*(this.nside-1),this.npix=12*this.npface,this.fact2=4/this.npix,this.fact1=(this.nside<<1)*this.fact2,this.order=t.nside2order(this.nside)},t.calculateNSide=function(s){for(var i=0,n=s*s,a=180/Constants.PI,e=3600*3600*4*Constants.PI*a*a,h=Utils.castToInt(e/n),r=h/12,o=Math.sqrt(r),c=t.NS_MAX,u=0,p=0;t.NSIDELIST.length>p;p++)if(c>=Math.abs(o-t.NSIDELIST[p])&&(c=Math.abs(o-t.NSIDELIST[p]),i=t.NSIDELIST[p],u=p),o>i&&t.NS_MAX>o&&(i=t.NSIDELIST[u+1]),o>t.NS_MAX)return console.log("nside cannot be bigger than "+t.NS_MAX),t.NS_MAX;return i},t.nside2order=function(s){"use strict";return(s&s-1)>0?-1:Utils.castToInt(t.log2(s))},t.log2=function(t){"use strict";return Math.log(t)/Math.log(2)},t.prototype.ang2pix_nest=function(s,i){"use strict";var n,a,e,h,r,o,c,u,p,l,d,f,I;if(i>=Constants.TWOPI&&(i-=Constants.TWOPI),0>i&&(i+=Constants.TWOPI),s>Constants.PI||0>s)throw{name:"Illegal argument",message:"theta must be between 0 and "+Constants.PI};if(i>Constants.TWOPI||0>i)throw{name:"Illegal argument",message:"phi must be between 0 and "+Constants.TWOPI};if(a=Math.cos(s),e=Math.abs(a),h=i/Constants.PIOVER2,t.Z0>=e){var M=this.nside*(.5+h),y=this.nside*.75*a,u=M-y,p=M+y;o=u>>this.order,c=p>>this.order,d=o==c?4==o?4:o+4:c>o?o:c+8,f=Utils.castToInt(p&this.nside-1),I=Utils.castToInt(this.nside-(u&this.nside-1)-1)}else{l=Utils.castToInt(h),l>=4&&(l=3),r=h-l;var g=this.nside*Math.sqrt(3*(1-e));u=Utils.castToInt(r*g),p=Utils.castToInt((1-r)*g),u=Math.min(t.NS_MAX-1,u),p=Math.min(t.NS_MAX-1,p),a>=0?(d=l,f=Utils.castToInt(this.nside-p-1),I=Utils.castToInt(this.nside-u-1)):(d=l+8,f=u,I=p)}return n=this.xyf2nest(f,I,d)},t.prototype.xyf2nest=function(t,s,i){"use strict";return(i<<2*this.order)+(this.utab[255&t]|this.utab[255&t>>8]<<16|this.utab[255&t>>16]<<32|this.utab[255&t>>24]<<48|this.utab[255&s]<<1|this.utab[255&s>>8]<<17|this.utab[255&s>>16]<<33|this.utab[255&s>>24]<<49)},t.prototype.nest2xyf=function(t){"use strict";var s={};s.face_num=t>>2*this.order;var i=t&this.npface-1,n=(93823560581120&i)>>16|(614882086624428e4&i)>>31|21845&i|(1431633920&i)>>15;return s.ix=this.ctab[255&n]|this.ctab[255&n>>8]<<4|this.ctab[255&n>>16]<<16|this.ctab[255&n>>24]<<20,i>>=1,n=(93823560581120&i)>>16|(614882086624428e4&i)>>31|21845&i|(1431633920&i)>>15,s.iy=this.ctab[255&n]|this.ctab[255&n>>8]<<4|this.ctab[255&n>>16]<<16|this.ctab[255&n>>24]<<20,s},t.prototype.pix2ang_nest=function(s){"use strict";if(0>s||s>this.npix-1)throw{name:"Illegal argument",message:"ipix out of range"};var i,n,a,e=this.nest2xyf(s),h=e.ix,r=e.iy,o=e.face_num,c=(t.JRLL[o]<<this.order)-h-r-1;this.nside>c?(i=c,n=1-i*i*this.fact2,a=0):c>this.nl3?(i=this.nl4-c,n=i*i*this.fact2-1,a=0):(i=this.nside,n=(this.nl2-c)*this.fact1,a=1&c-this.nside);var u=Math.acos(n),p=(t.JPLL[o]*i+h-r+1+a)/2;p>this.nl4&&(p-=this.nl4),1>p&&(p+=this.nl4);var l=(p-.5*(a+1))*(Constants.PIOVER2/i);return{theta:u,phi:l}},t.nside2Npix=function(s){"use strict";if(0>this.NSIDELIST.indexOf(s))throw{name:"Illegal argument",message:"nside should be >0, power of 2, <"+t.NS_MAX};var i=12*s*s;return i},t.prototype.xyf2ring=function(s,i,n){"use strict";var a,e,h,r=t.JRLL[n]*this.nside-s-i-1;this.nside>r?(a=r,h=2*a*(a-1),e=0):r>3*this.nside?(a=this.nl4-r,h=this.npix-2*(a+1)*a,e=0):(a=this.nside,h=this.ncap+(r-this.nside)*this.nl4,e=1&r-this.nside);var o=(t.JPLL[n]*a+s-i+1+e)/2;return o>this.nl4?o-=this.nl4:1>o&&(o+=this.nl4),h+o-1},t.prototype.nest2ring=function(t){"use strict";var s=this.nest2xyf(t),i=this.xyf2ring(s.ix,s.iy,s.face_num);return i},t.prototype.corners_nest=function(t,s){"use strict";var i=this.nest2ring(t);return this.corners_ring(i,s)},t.prototype.pix2ang_ring=function(t){"use strict";var s,i,n,a,e,h,r,o,c;if(0>t||t>this.npix-1)throw{name:"Illegal argument",message:"ipix out of range"};return h=t+1,this.ncap>=h?(o=h/2,c=Utils.castToInt(o),n=Utils.castToInt(Math.sqrt(o-Math.sqrt(c)))+1,a=h-2*n*(n-1),s=Math.acos(1-n*n*this.fact2),i=(a-.5)*Constants.PI/(2*n)):this.npix-this.ncap>t?(e=t-this.ncap,n=e/this.nl4+this.nside,a=e%this.nl4+1,r=(1&n+this.nside)>0?1:.5,s=Math.acos((this.nl2-n)*this.fact1),i=(a-r)*Constants.PI/this.nl2):(e=this.npix-t,n=Utils.castToInt(.5*(1+Math.sqrt(2*e-1))),a=4*n+1-(e-2*n*(n-1)),s=Math.acos(-1+Math.pow(n,2)*this.fact2),i=(a-.5)*Constants.PI/(2*n)),[s,i]},t.prototype.ring=function(t){"use strict";var s,i,n=0,a=t+1,e=0;return this.ncap>=a?(i=a/2,e=Utils.castToInt(i),n=Utils.castToInt(Math.sqrt(i-Math.sqrt(e)))+1):this.nl2*(5*this.nside+1)>=a?(s=Utils.castToInt(a-this.ncap-1),n=Utils.castToInt(s/this.nl4+this.nside)):(s=this.npix-a+1,i=s/2,e=Utils.castToInt(i),n=Utils.castToInt(Math.sqrt(i-Math.sqrt(e)))+1,n=this.nl4-n),n},t.prototype.integration_limits_in_costh=function(t){"use strict";var s,i,n,a;return a=1*this.nside,this.nside>=t?(i=1-Math.pow(t,2)/3/this.npface,n=1-Math.pow(t-1,2)/3/this.npface,s=t==this.nside?2*(this.nside-1)/3/a:1-Math.pow(t+1,2)/3/this.npface):this.nl3>t?(i=2*(2*this.nside-t)/3/a,n=2*(2*this.nside-t+1)/3/a,s=2*(2*this.nside-t-1)/3/a):(n=t==this.nl3?2*(-this.nside+1)/3/a:-1+Math.pow(4*this.nside-t+1,2)/3/this.npface,s=-1+Math.pow(this.nl4-t-1,2)/3/this.npface,i=-1+Math.pow(this.nl4-t,2)/3/this.npface),[n,i,s]},t.prototype.pixel_boundaries=function(t,s,i,n){var a,e,h,r,o,c,u,p,l=1*this.nside;if(Math.abs(n)>=1-1/3/this.npface)return u=i*Constants.PIOVER2,p=(i+1)*Constants.PIOVER2,[u,p];if(1.5*n>=1)a=Math.sqrt(3*(1-n)),e=1/l/a,h=s,r=h-1,o=t-s,c=o+1,u=Constants.PIOVER2*(Math.max(r*e,1-c*e)+i),p=Constants.PIOVER2*(Math.min(1-o*e,h*e)+i);else if(1.5*n>-1){var d=.5*(1-1.5*n),f=d+1,I=this.nside+t%2;h=s-(I-t)/2,r=h-1,o=(I+t)/2-s,c=o+1,u=Constants.PIOVER2*(Math.max(f-c/l,-d+r/l)+i),p=Constants.PIOVER2*(Math.min(f-o/l,-d+h/l)+i)}else{a=Math.sqrt(3*(1+n)),e=1/l/a;var M=2*this.nside;h=t-M+s,r=h-1,o=M-s,c=o+1,u=Constants.PIOVER2*(Math.max(1-(M-r)*e,(M-c)*e)+i),p=Constants.PIOVER2*(Math.min(1-(M-h)*e,(M-o)*e)+i)}return[u,p]},t.vector=function(t,s){"use strict";var i=1*Math.sin(t)*Math.cos(s),n=1*Math.sin(t)*Math.sin(s),a=1*Math.cos(t);return new SpatialVector(i,n,a)},t.prototype.corners_ring=function(s,i){"use strict";var n=2*i+2,a=Array(n),e=this.pix2ang_ring(s),h=Math.cos(e[0]),r=e[0],o=e[1],c=Utils.castToInt(o/Constants.PIOVER2),u=this.ring(s),p=Math.min(u,Math.min(this.nside,this.nl4-u)),l=0,d=Constants.PIOVER2/p;l=u>=this.nside&&this.nl3>=u?Utils.castToInt(o/d+u%2/2)+1:Utils.castToInt(o/d)+1,l-=c*p;var f=n/2,I=this.integration_limits_in_costh(u),M=Math.acos(I[0]),y=Math.acos(I[2]),g=this.pixel_boundaries(u,l,c,I[0]);if(a[0]=l>p/2?t.vector(M,g[1]):t.vector(M,g[0]),g=this.pixel_boundaries(u,l,c,I[2]),a[f]=l>p/2?t.vector(y,g[1]):t.vector(y,g[0]),1==i){var x=Math.acos(I[1]);g=this.pixel_boundaries(u,l,c,I[1]),a[1]=t.vector(x,g[0]),a[3]=t.vector(x,g[1])}else for(var P=I[2]-I[0],C=P/(i+1),v=1;i>=v;v++)h=I[0]+C*v,r=Math.acos(h),g=this.pixel_boundaries(u,l,c,h),a[v]=t.vector(r,g[0]),a[n-v]=t.vector(r,g[1]);return a},t.vec2Ang=function(t){"use strict";var s=t.z/t.length(),i=Math.acos(s),n=0;return(0!=t.x||0!=t.y)&&(n=Math.atan2(t.y,t.x)),0>n&&(n+=2*Math.PI),[i,n]},t.prototype.queryDisc=function(s,i,n,a){"use strict";if(0>i||i>Constants.PI)throw{name:"Illegal argument",message:"angular radius is in RADIAN and should be in [0,pi]"};var e,h,r,o,c,u,p,l,d,f,I,M,y,g,x,P,C,v,_,T=new LongRangeSetBuilder,R=null,c=i;if(a&&(c+=Constants.PI/this.nl4),R=t.vec2Ang(s),u=R[0],p=R[1],I=this.fact2,M=this.fact1,o=Math.cos(u),_=1/Math.sqrt((1-o)*(1+o)),g=u-c,x=u+c,l=Math.cos(c),C=Math.cos(g),e=this.ringAbove(C)+1,P=Math.cos(x),h=this.ringAbove(P),e>h&&0==h&&(h=e),0>=g)for(var m=1;e>m;++m)this.inRing(m,0,Math.PI,T);for(r=e;h>=r;++r)v=this.nside>r?1-r*r*I:this.nl3>=r?(this.nl2-r)*M:-1+(this.nl4-r)*(this.nl4-r)*I,d=(l-v*o)*_,f=1-v*v-d*d,y=Math.atan2(Math.sqrt(f),d),isNaN(y)&&(y=c),this.inRing(r,p,y,T);if(x>=Math.PI)for(var m=h+1;this.nl4>m;++m)this.inRing(m,0,Math.PI,T,!1);var b;if(n){for(var S=T.items,U=[],O=0;S.length>O;O++){var A=this.ring2nest(S[O]);U.indexOf(A)>=0||U.push(A)}b=U}else b=T.items;return b},t.prototype.inRing=function(t,s,i,n,a){"use strict";var e,h,r,o,c=!1,u=!1,p=1e-12,l=0,d=0,f=0,I=0,M=(s-i)%Constants.TWOPI-p,y=s+i+p,g=(s+i)%Constants.TWOPI+p;if(p>Math.abs(i-Constants.PI)&&(c=!0),t>=this.nside&&this.nl3>=t?(d=t-this.nside+1,r=this.ncap+this.nl4*(d-1),o=r+this.nl4-1,e=d%2,h=this.nl4):(this.nside>t?(d=t,r=2*d*(d-1),o=r+4*d-1):(d=4*this.nside-t,r=this.npix-2*d*(d+1),o=r+4*d-1),h=4*d,e=1),c)return n.appendRange(r,o),void 0;if(l=e/2,a)f=Math.round(h*M/Constants.TWOPI-l),I=Math.round(h*y/Constants.TWOPI-l),f%=h,I>h&&(I%=h);else{if(f=Math.ceil(h*M/Constants.TWOPI-l),I=Utils.castToInt(h*g/Constants.TWOPI-l),f>I&&1==t&&(I=Utils.castToInt(h*y/Constants.TWOPI-l)),f==I+1&&(f=I),1==f-I&&Constants.PI>i*h)return console.log("the interval is too small and avay from center"),void 0;f=Math.min(f,h-1),I=Math.max(I,0)}if(f>I&&(u=!0),u)f+=r,I+=r,n.appendRange(r,I),n.appendRange(f,o);else{if(0>f)return f=Math.abs(f),n.appendRange(r,r+I),n.appendRange(o-f+1,o),void 0;f+=r,I+=r,n.appendRange(f,I)}},t.prototype.ringAbove=function(t){"use strict";var s=Math.abs(t);if(s>Constants.TWOTHIRD){var i=Utils.castToInt(this.nside*Math.sqrt(3*(1-s)));return t>0?i:4*this.nside-i-1}return Utils.castToInt(this.nside*(2-1.5*t))},t.prototype.ring2nest=function(t){"use strict";var s=this.ring2xyf(t);return this.xyf2nest(s.ix,s.iy,s.face_num)},t.prototype.ring2xyf=function(s){"use strict";var i,n,a,e,h={};if(this.ncap>s){i=Utils.castToInt(.5*(1+Math.sqrt(1+2*s))),n=s+1-2*i*(i-1),a=0,e=i,h.face_num=0;var r=n-1;r>=2*i&&(h.face_num=2,r-=2*i),r>=i&&++h.face_num}else if(this.npix-this.ncap>s){var o=s-this.ncap;this.order>=0?(i=(o>>this.order+2)+this.nside,n=(o&this.nl4-1)+1):(i=o/this.nl4+this.nside,n=o%this.nl4+1),a=1&i+this.nside,e=this.nside;var c,u,p=i-this.nside+1,l=this.nl2+2-p;this.order>=0?(c=n-Utils.castToInt(p/2)+this.nside-1>>this.order,u=n-Utils.castToInt(l/2)+this.nside-1>>this.order):(c=(n-Utils.castToInt(p/2)+this.nside-1)/this.nside,u=(n-Utils.castToInt(l/2)+this.nside-1)/this.nside),h.face_num=u==c?4==u?4:Utils.castToInt(u)+4:c>u?Utils.castToInt(u):Utils.castToInt(c)+8}else{var o=this.npix-s;i=Utils.castToInt(.5*(1+Math.sqrt(2*o-1))),n=4*i+1-(o-2*i*(i-1)),a=0,e=i,i=2*this.nl2-i,h.face_num=8;var r=n-1;r>=2*e&&(h.face_num=10,r-=2*e),r>=e&&++h.face_num}var d=i-t.JRLL[h.face_num]*this.nside+1,f=2*n-t.JPLL[h.face_num]*e-a-1;return f>=this.nl2&&(f-=8*this.nside),h.ix=f-d>>1,h.iy=-(f+d)>>1,h},t}(),Utils=function(){},Utils.radecToPolar=function(t,s){return{theta:Math.PI/2-s/180*Math.PI,phi:t/180*Math.PI}},Utils.polarToRadec=function(t,s){return{ra:180*s/Math.PI,dec:180*(Math.PI/2-t)/Math.PI}},Utils.castToInt=function(t){return t>0?Math.floor(t):Math.ceil(t)};//=================================
//            AstroMath
//=================================

// Class AstroMath having 'static' methods
function AstroMath() {}

// Constant for conversion Degrees => Radians (rad = deg*AstroMath.D2R)
AstroMath.D2R = Math.PI/180.0;
// Constant for conversion Radians => Degrees (deg = rad*AstroMath.R2D)
AstroMath.R2D = 180.0/Math.PI;
/**
 * Function sign
 * @param x value for checking the sign
 * @return -1, 0, +1 respectively if x < 0, = 0, > 0
 */
AstroMath.sign = function(x) { return x > 0 ? 1 : (x < 0 ? -1 : 0 ); };

/**
 * Function cosd(degrees)
 * @param x angle in degrees
 * @returns the cosine of the angle
 */
AstroMath.cosd = function(x) {
	if (x % 90 == 0) {
		var i = Math.abs(Math.floor(x / 90 + 0.5)) % 4;
		switch (i) {
			case 0:	return 1;
			case 1:	return 0;
			case 2:	return -1;
			case 3:	return 0;
		}
	}
	return Math.cos(x*AstroMath.D2R);
};

/**
 * Function sind(degrees)
 * @param x angle in degrees
 * @returns the sine of the angle
 */
AstroMath.sind = function(x) {
	if (x % 90 === 0) {
		var i = Math.abs(Math.floor(x / 90 - 0.5)) % 4;
		switch (i) {
			case 0:	return 1;
			case 1:	return 0;
			case 2:	return -1;
			case 3:	return 0;
		}
	}

	return Math.sin(x*AstroMath.D2R);
};

/**
 * Function tand(degrees)
 * @param x angle in degrees
 * @returns the tangent of the angle
 */
AstroMath.tand = function(x) {
	var resid;

	resid = x % 360;
	if (resid == 0 || Math.abs(resid) == 180) {
		return 0;
	} else if (resid == 45 || resid == 225) {
		return 1;
	} else if (resid == -135 || resid == -315) {
		return -1
	}

	return Math.tan(x * AstroMath.D2R);
};

/**
 * Function asin(degrees)
 * @param sine value [0,1]
 * @return the angle in degrees
 */
AstroMath.asind = function(x) { return Math.asin(x)*AstroMath.R2D; };

/**
 * Function acos(degrees)
 * @param cosine value [0,1]
 * @return the angle in degrees
 */
AstroMath.acosd = function(x) { return Math.acos(x)*AstroMath.R2D; };

/**
 * Function atan(degrees)
 * @param tangent value
 * @return the angle in degrees
 */
AstroMath.atand = function(x) { return Math.atan(x)*AstroMath.R2D; };

/**
 * Function atan2(y,x)
 * @param y y component of the vector
 * @param x x component of the vector
 * @return the angle in radians
 */
AstroMath.atan2 = function(y,x) {
	if (y != 0.0) {
		var sgny = AstroMath.sign(y);
		if (x != 0.0) {
			var phi = Math.atan(Math.abs(y/x));
			if (x > 0.0) return phi*sgny;
			else if (x < 0) return (Math.PI-phi)*sgny;
		} else return (Math.PI/2)*sgny;
	} else {
		return x > 0.0 ? 0.0 : (x < 0 ? Math.PI : 0.0/0.0);
	}
}  

/**
 * Function atan2d(y,x)
 * @param y y component of the vector
 * @param x x component of the vector
 * @return the angle in degrees
 */
AstroMath.atan2d = function(y,x) {
	return AstroMath.atan2(y,x)*AstroMath.R2D;
}

/*=========================================================================*/
/**
 * Computation of hyperbolic cosine
 * @param x argument
 */
AstroMath.cosh = function(x) {
	return (Math.exp(x)+Math.exp(-x))/2;
}

/**
 * Computation of hyperbolic sine
 * @param x argument
 */
AstroMath.sinh = function(x) {
	return (Math.exp(x)-Math.exp(-x))/2;
}

/**
 * Computation of hyperbolic tangent
 * @param x argument
 */
AstroMath.tanh = function(x) {
	return (Math.exp(x)-Math.exp(-x))/(Math.exp(x)+Math.exp(-x));
}

/**
 * Computation of Arg cosh
 * @param x argument in degrees. Must be in the range [ 1, +infinity ]
 */
AstroMath.acosh = function(x) {
	return(Math.log(x+Math.sqrt(x*x-1.0)));
}

/**
 * Computation of Arg sinh
 * @param x argument in degrees
 */
AstroMath.asinh = function(x) {
	return(Math.log(x+Math.sqrt(x*x+1.0)));
}

/**
 * Computation of Arg tanh
 * @param x argument in degrees. Must be in the range ] -1, +1 [
 */
AstroMath.atanh = function(x) {
	return(0.5*Math.log((1.0+x)/(1.0-x)));
}

//=============================================================================
//      Special Functions using trigonometry
//=============================================================================
/**
 * Computation of sin(x)/x
 *	@param x in degrees.
 * For small arguments x <= 0.001, use approximation 
 */
AstroMath.sinc = function(x) {
	var ax = Math.abs(x);
	var y;

	if (ax <= 0.001) {
		ax *= ax;
		y = 1 - ax*(1.0-ax/20.0)/6.0;
	} else {
		y = Math.sin(ax)/ax;
	}

	return y;
}

/**
 * Computes asin(x)/x
 * @param x in degrees.
 * For small arguments x <= 0.001, use an approximation
 */
AstroMath.asinc = function(x) {
	var ax = Math.abs(x);
	var y;

	if (ax <= 0.001) {
		ax *= ax; 
		y = 1 + ax*(6.0 + ax*(9.0/20.0))/6.0;
	} else {
		y = Math.asin(ax)/ax;	// ???? radians ???
	}

	return (y);
}


//=============================================================================
/**
 * Computes the hypotenuse of x and y
 * @param x value
 * @param y value
 * @return sqrt(x*x+y*y)
 */
AstroMath.hypot = function(x,y) {
	return Math.sqrt(x*x+y*y);
}

/** Generate the rotation matrix from the Euler angles
 * @param z	Euler angle
 * @param theta	Euler angle
 * @param zeta	Euler angles
 * @return R [3][3]		the rotation matrix
 * The rotation matrix is defined by:<pre>
 *    R =      R_z(-z)      *        R_y(theta)     *     R_z(-zeta)
 *   |cos.z -sin.z  0|   |cos.the  0 -sin.the|   |cos.zet -sin.zet 0|
 * = |sin.z  cos.z  0| x |   0     1     0   | x |sin.zet  cos.zet 0|
 *   |   0      0   1|   |sin.the  0  cos.the|   |   0        0    1|
 * </pre>
 */
AstroMath.eulerMatrix = function(z, theta, zeta) {
	var R = new Array(3);
	R[0] = new Array(3);
	R[1] = new Array(3);
	R[2] = new Array(3);
	var cosdZ = AstroMath.cosd(z);
	var sindZ = AstroMath.sind(z);
	var cosdTheta = AstroMath.cosd(theta);
	var w = AstroMath.sind(theta) ;
	var cosdZeta = AstroMath.cosd(zeta);
	var sindZeta = AstroMath.sind(zeta);

	R[0][0] = cosdZeta*cosdTheta*cosdZ - sindZeta*sindZ;
	R[0][1] = -sindZeta*cosdTheta*cosdZ - cosdZeta*sindZ;
	R[0][2] = -w*cosdZ;

	R[1][0] = cosdZeta*cosdTheta*sindZ + sindZeta*cosdZ;
	R[1][1] = -sindZeta*cosdTheta*sindZ + cosdZeta*cosdZ;
	R[1][2] = -w*sindZ;

	R[2][0] = -w*cosdZeta;
	R[2][1] = -w*cosdZ;
	R[2][2] = cosdTheta;
	return R ;
};


AstroMath.displayMatrix = function(m) {
	// Number of rows
	var nbrows = m.length;
	// Max column count
	var nbcols = 0
	for (var i=0; i<nbrows; i++) {
		if (m[i].length > nbcols) nbcols = m[i].length;
	}
	var str = '<table>\n';
	for (var i=0; i<nbrows; i++) {
		str += '<tr>';
		for (var j=0; j<nbrows; j++) {
			str += '<td>';
			if (i < m[i].length)
				str += (m[i][j]).toString();
			str += '</td>';
		}
		str += '</td>\n';
	}
	str += '</table>\n';

	return str;
}
function Projection(lon0, lat0) {
	this.PROJECTION = Projection.PROJ_TAN;
	this.ROT = this.tr_oR(lon0, lat0);
}

//var ROT;
//var PROJECTION = Projection.PROJ_TAN;	// Default projection


Projection.PROJ_TAN = 1;	/* Gnomonic projection*/
Projection.PROJ_TAN2 = 2;	/* Stereographic projection*/
Projection.PROJ_STG = 2;	
Projection.PROJ_SIN = 3;	/* Orthographic		*/
Projection.PROJ_SIN2 = 4;	/* Equal-area 		*/
Projection.PROJ_ZEA = 4;	/* Zenithal Equal-area 	*/
Projection.PROJ_ARC = 5;	/* For Schmidt plates	*/
Projection.PROJ_SCHMIDT = 5;	/* For Schmidt plates	*/
Projection.PROJ_AITOFF = 6;	/* Aitoff Projection	*/
Projection.PROJ_AIT = 6;	/* Aitoff Projection	*/
Projection.PROJ_GLS = 7;	/* Global Sin (Sanson)	*/
Projection.PROJ_MERCATOR = 8;
Projection.PROJ_MER = 8;	
Projection.PROJ_LAM = 9;	/* Lambert Projection	*/
Projection.PROJ_LAMBERT = 9;	
Projection.PROJ_TSC = 10;	/* Tangent Sph. Cube	*/
Projection.PROJ_QSC = 11;	/* QuadCube Sph. Cube	*/

Projection.PROJ_LIST = [
	"Mercator",Projection.PROJ_MERCATOR,
	"Gnomonic",Projection.PROJ_TAN,
	"Stereographic",Projection.PROJ_TAN2,
	"Orthographic",Projection.PROJ_SIN,
	"Zenithal",Projection.PROJ_ZEA,
	"Schmidt",Projection.PROJ_SCHMIDT,
	"Aitoff",Projection.PROJ_AITOFF,
	"Lambert",Projection.PROJ_LAMBERT,
//	"Tangential",Projection.PROJ_TSC,
//	"Quadrilaterized",Projection.PROJ_QSC,
];
Projection.PROJ_NAME = [
	'-', 'Gnomonic', 'Stereographic', 'Orthographic', 'Equal-area', 'Schmidt plates',
	'Aitoff', 'Global sin', 'Mercator', 'Lambert'
];

Projection.prototype = { 
	
	/** Set the center of the projection
	 * 
	 * (ajout T. Boch, 19/02/2013)
	 * 
	 * */
	setCenter: function(lon0, lat0) {
		this.ROT = this.tr_oR(lon0, lat0);
	},
	
	/**
	 * Set the projection to use
	 * p = projection code
	 */
	setProjection: function(p) {
		this.PROJECTION = p;
	},


	/**
	 * Computes the projection of 1 point : ra,dec => X,Y
	 * alpha, delta = longitude, lattitude
	 */
	project: function(alpha, delta) {
		var u1 = this.tr_ou(alpha, delta);	// u1[3]
		var u2 = this.tr_uu(u1, this.ROT);	// u2[3]
		var P = this.tr_up(this.PROJECTION, u2);	// P[2] = [X,Y]
		if (P == null) {
			return null;
		}

		return { X: -P[0], Y: -P[1] };
	},

	/**
	 * Computes the coordinates from a projection point : X,Y => ra,dec
	 * return o = [ ra, dec ]
	 */
	unproject: function(X,Y) {
		X = -X; Y = -Y;
		var u1 = this.tr_pu(this.PROJECTION, X, Y);	// u1[3]
		var u2 = this.tr_uu1(u1, this.ROT);	// u2[3]
		var o = this.tr_uo(u2);	// o[2]

		return { ra: o[0], dec: o[1] };
	},

	/**
	 * Compute projections from unit vector
	 * The center of the projection correspond to u = [1, 0, 0)
	 * proj = projection system (integer code like _PROJ_MERCATOR_
	 * u[3] = unit vector
	 * return: an array [x,y] or null
	 */
	tr_up: function(proj, u) {
		var x = u[0]; var y = u[1]; var z = u[2];
		var r, den;
		var pp;
		var X,Y;

		r = AstroMath.hypot(x,y);			// r = cos b
		if (r == 0.0 && z == 0.0) return null;

		switch(proj) {
			default:
				pp = null;
				break;

			case Projection.PROJ_AITOFF:
				den = Math.sqrt(r*(r+x)/2.0); 		// cos b . cos l/2
				X = Math.sqrt(2.0*r*(r-x));
				den = Math.sqrt((1.0 + den)/2.0); 
				X = X / den;
				Y = z / den;
				if (y < 0.0) X = -X;
				pp = [ X, Y];
				break;

			case Projection.PROJ_GLS:
				Y = Math.asin(z);				// sin b
				X = (r != 0) ? Math.atan2(y,x)*r : 0.0;
				pp = [ X, Y];
				break;

			case Projection.PROJ_MERCATOR:
				if (r != 0) {
					X = Math.atan2(y,x);
					Y = AstroMath.atanh(z);
					pp = [ X, Y];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_TAN:
				if (x > 0.0) {
					X = y/x;
					Y = z/x;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_TAN2:
				den = (1.0 + x)/2.0;
				if (den > 0.0)	{
					X = y/den;
					Y = z/den;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
			 	break;

			case Projection.PROJ_ARC:
				if (x <= -1.0) {
					// Distance of 180 degrees
					X = Math.PI
					Y = 0.0;
				} else {
					// Arccos(x) = Arcsin(r)
					r = AstroMath.hypot(y,z);
					if (x > 0.0) den = AstroMath.asinc(r);
					else den = Math.acos(x)/r;
					X = y * den;
					Y = z * den;
				}
				pp = [ X, Y ];
				break;

			case Projection.PROJ_SIN:
				if (x >= 0.0) {
					X = y;
					Y = z;
					pp = [ X, Y ];
				} else {
					pp = null;
				}
				break;

			case Projection.PROJ_SIN2:	// Always possible
				den = Math.sqrt((1.0 + x)/2.0);
				if (den != 0)	{
					X = y / den;
					Y = z / den;
				} else {
					// For x = -1
					X = 2.0;
					Y = 0.0;
				}
				pp = [ X, Y ];
				break;

			case Projection.PROJ_LAMBERT:	// Always possible
				Y = z;
				X = 0;
				if (r != 0)	X = Math.atan2(y,x);
				pp = [ X, Y ];
				break;
	  }
	  return pp;
	},

	/**
	 * Computes Unit vector from a position in projection centered at position (0,0).
	 * proj = projection code
	 * X,Y : coordinates of the point in the projection
	 * returns : the unit vector u[3] or a face number for cube projection. 
	 *           null if the point is outside the limits, or if the projection is unknown.
	 */
	tr_pu: function( proj, X, Y ) {
		var r,s,x,y,z;

		switch(proj) {
			default:
			return null;

			case Projection.PROJ_AITOFF:
				// Limit is ellipse with axises 
				// a = 2 * sqrt(2) ,  b = sqrt(2)
				// Compute dir l/2, b
				r = X*X/8.e0 + Y*Y/2.e0; 	// 1 - cos b . cos l/2
				if (r > 1.0) {
	  				// Test outside domain */
					return null;
				}
				x = 1.0 - r ;	// cos b . cos l/2
				s = Math.sqrt(1.0 - r/2.0) ;	// sqrt(( 1 + cos b . cos l/2)/2)
				y = X * s / 2.0;
				z = Y * s ;
				// From (l/2,b) to (l,b)
				r = AstroMath.hypot( x, y ) ;	// cos b
				if (r != 0.0) {
					s = x;
					x = (s*s - y*y) /r;
					y = 2.0 * s * y/r;
				}
				break;

			case Projection.PROJ_GLS:
				// Limit is |Y| <= pi/2
				z = Math.sin(Y);
				r = 1 - z*z;		// cos(b) ** 2
				if (r < 0.0) {
					return null;
				}
				r = Math.sqrt(r);		// cos b
				if (r != 0.0) {
					s = X/r;	// Longitude
				} else {
					s = 0.0;	// For poles
				}
				x = r * Math.cos(s);
				y = r * Math.sin(s);
				break;

			case Projection.PROJ_MERCATOR:
				z = AstroMath.tanh(Y);
				r = 1.0/AstroMath.cosh(Y);
				x = r * Math.cos(X);
				y = r * Math.sin(X);
				break;

			case Projection.PROJ_LAMBERT:
				// Always possible
				z = Y;
				r = 1 - z*z;		// cos(b) ** 2
				if (r < 0.0) {
					return null;
				}
				r = Math.sqrt(r);		// cos b
				x = r * Math.cos(X);
				y = r * Math.sin(X);
				break;
	
			case Projection.PROJ_TAN:
				// No limit
				x = 1.0 / Math.sqrt(1.0 + X*X + Y*Y);
				y = X * x;
				z = Y * x;
				break;

			case Projection.PROJ_TAN2:
				// No limit
				r = (X*X + Y*Y)/4.0;
				s = 1.0 + r;
				x = (1.0 - r)/s;
				y = X / s;
				z = Y / s;
				break;

			case Projection.PROJ_ARC:
				// Limit is circle, radius PI
				r = AstroMath.hypot(X, Y);
				if (r > Math.PI) {
					return null;
				}
				s = AstroMath.sinc(r);
				x = Math.cos(r);
				y = s * X;
				z = s * Y;
				break;

			case Projection.PROJ_SIN:
				// Limit is circle, radius 1
				s = 1.0 - X*X - Y*Y;
				if (s < 0.0) {
					return null;
				}
				x = Math.sqrt(s);
				y = X;
				z = Y;
				break;

			case Projection.PROJ_SIN2:
				// Limit is circle, radius 2	*/
				r = (X*X + Y*Y)/4.e0;
				if (r > 1.0) {
					return null;
				}
				s = Math.sqrt(1.0 - r);
				x = 1.0 - 2.0 * r;
				y = s * X;
				z = s * Y;
				break;
	  }
	  return [ x,y,z ];
	},

	/**
	 * Creates the rotation matrix R[3][3] defined as
	 * R[0] (first row) = unit vector towards Zenith
	 * R[1] (second row) = unit vector towards East
	 * R[2] (third row) = unit vector towards North
	 * o[2] original angles
	 * @return rotation matrix
	 */
	tr_oR: function(lon, lat) {
		var R = new Array(3);
		R[0] = new Array(3);
		R[1] = new Array(3);
		R[2] = new Array(3);
		R[2][2] =  AstroMath.cosd(lat);
		R[0][2] =  AstroMath.sind(lat);
		R[1][1] =  AstroMath.cosd(lon);
		R[1][0] =  -AstroMath.sind(lon);
		R[1][2] =  0.0;
		R[0][0] =  R[2][2] * R[1][1];  
		R[0][1] = -R[2][2] * R[1][0];
		R[2][0] = -R[0][2] * R[1][1];
		R[2][1] =  R[0][2] * R[1][0];
		return R;
	},

	/**
	 * Transformation from polar coordinates to Unit vector
	 * @return U[3]
	 */
	tr_ou: function(ra, dec) {
		var u = new Array(3);
		var cosdec = AstroMath.cosd(dec);

		u[0] = cosdec * AstroMath.cosd(ra);
		u[1] = cosdec * AstroMath.sind(ra);
		u[2] = AstroMath.sind(dec);

		return u;
	},

	/**
	 * Rotates the unit vector u1 using the rotation matrix
	 * u1[3] unit vector
	 * R[3][3] rotation matrix
	 * return resulting unit vector u2[3]
	 */
	tr_uu: function( u1, R ) {
		var u2 = new Array(3);
		var x = u1[0];
		var y = u1[1];
		var z = u1[2];

		u2[0] = R[0][0]*x + R[0][1]*y + R[0][2]*z ;
		u2[1] = R[1][0]*x + R[1][1]*y + R[1][2]*z ;
		u2[2] = R[2][0]*x + R[2][1]*y + R[2][2]*z ;

		return u2;
	},

	/**
	 * reverse rotation the unit vector u1 using the rotation matrix
	 * u1[3] unit vector
	 * R[3][3] rotation matrix
	 * return resulting unit vector u2[3]
	 */
	tr_uu1: function( u1 , R) {
		var u2 = new Array(3);
		var x = u1[0];
		var y = u1[1];
		var z = u1[2];

		u2[0] = R[0][0]*x + R[1][0]*y + R[2][0]*z;
		u2[1] = R[0][1]*x + R[1][1]*y + R[2][1]*z;
		u2[2] = R[0][2]*x + R[1][2]*y + R[2][2]*z;

		return u2;
	},

	/**
	 * Computes angles from direction cosines
	 * u[3] = direction cosines vector
	 * return o = [ ra, dec ]
	 */
	tr_uo: function(u) {
		var x = u[0]; var y = u[1]; var z = u[2];  
		var r2 = x*x + y*y;
		var ra, dec;
		if (r2  == 0.0) {
	 		// in case of poles
			if (z == 0.0) {
				return null;
			}
			ra = 0.0;
			dec = z > 0.0 ? 90.0 : -90.0;
		} else {
			dec = AstroMath.atand( z / Math.sqrt(r2));
			ra  = AstroMath.atan2d (y , x );
			if (ra < 0.0) ra += 360.0;
		}

		return [ ra, dec ];
	}
}//=================================
// Class Coo
//=================================

/**
 * Constructor
 * @param longitude longitude (decimal degrees)
 * @param latitude latitude (decimal degrees)
 * @param prec precision
 * (8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 */
function Coo(longitude, latitude, prec) {
	this.lon = longitude;
	this.lat = latitude;
	this.prec = prec;
	this.frame = null;

	this.computeDirCos();
}

Coo.factor = [ 3600.0, 60.0, 1.0 ];

Coo.prototype = {
	/**
	 * Set the frame for the coordinates
	 * @param astroframe frame code
	 */
	setFrame: function(astroframe) {
		this.frame = astroframe;
	},

	/**
	 * Compute the direction cosine of these coordinates
	 */
	computeDirCos: function() {
		var coslat = AstroMath.cosd(this.lat);

		this.x = coslat*AstroMath.cosd(this.lon);
		this.y = coslat*AstroMath.sind(this.lon);
		this.z = AstroMath.sind(this.lat);	
	}, 

	/**
	 * Compute the coordinates from the direction cosine
	 */
	computeLonLat: function() {
		var r2 = this.x*this.x+this.y*this.y;
		this.lon = 0.0;
		if (r2 == 0.0) {
			// In case of poles
			if (this.z == 0.0) {
				this.lon = 0.0/0.0;
				this.lat = 0.0/0.0;
			} else {
				this.lat = (this.z > 0.0) ? 90.0 : -90.0;
			}
		} else {
			this.lon = AstroMath.atan2d(this.y, this.x);
			this.lat = AstroMath.atan2d(this.z, Math.sqrt(r2));
			if (this.lon < 0) this.lon += 360.0;
		}
	},

   /**
    * Distance between 2 points on the sphere.
    * @param  pos another position on the sphere
    * @return distance in degrees in range [0, 180]
   **/
    distance: function(pos) {
      // Take care of NaN:
    	if ((pos.x==0)&&(pos.y==0)&&(pos.z==0)) return(0./0.);
    	if ((this.x==0)&&(this.y==0)&&(this.z==0)) return(0./0.);
      return (2. * AstroMath.asind(0.5 * Math.sqrt(this.dist2(pos))));
    },

  /**
    * Squared distance between 2 points (= 4.sin<sup>2</sup>(r/2))
    * @param  pos      another position on the sphere
    * @return ||pos-this||<sup>2</sup> = 4.sin<sup>2</sup>(r/2)
   **/
   dist2: function(pos) {
//    	if ((this.x==0)&&(this.y==0)&&(this.z==0)) return(0./0.);
//    	if ((pos.x==0)&&(pos.y==0)&&(pos.z==0)) return(0./0.);
	var w = pos.x - this.x;
	var r2 = w * w;
	w = pos.y - this.y; r2 += w * w;
	w = pos.z - this.z; r2 += w * w;
	return r2;
   },

   /**
    * Transform the position into another frame.
    * @param new_frame	The frame of the resulting position.
   **/
   convertTo: function(new_frame) {
		// Verify first if frames identical -- then nothing to do !
		if (this.frame.equals(new_frame)) {
	    		return;
		}

		// Move via ICRS
		this.frame.toICRS(this.coo);	// Position now in ICRS
		new_frame.fromICRS(this.coo);	// Position now in new_frame
		this.frame = new_frame;
		this.lon = this.lat = 0./0.;	// Actual angles not recomputed
   },

    /**
     * Rotate a coordinate (apply a rotation to the position).
     * @param R [3][3] Rotation Matrix
     */
    rotate: function(R) {
      var X, Y, Z;
		if (R == Umatrix3) return;
		X = R[0][0]*this.x + R[0][1]*this.y + R[0][2]*this.z;
		Y = R[1][0]*this.x + R[1][1]*this.y + R[1][2]*this.z;
		Z = R[2][0]*this.x + R[2][1]*this.y + R[2][2]*this.z;
    	// this.set(X, Y, Z); Not necessary to compute positions each time.
		this.x = X; this.y = Y; this.z = Z;
		this.lon = this.lat = 0./0.;
    },

    /**
     * Rotate a coordinate (apply a rotation to the position) in reverse direction.
     * The method is the inverse of rotate.
     * @param R [3][3] Rotation Matrix
     */
    rotate_1: function(R) {
      var X, Y, Z;
      if (R == Umatrix3) return;
		X = R[0][0]*this.x + R[1][0]*this.y + R[2][0]*this.z;
		Y = R[0][1]*this.x + R[1][1]*this.y + R[2][1]*this.z;
		Z = R[0][2]*this.x + R[1][2]*this.y + R[2][2]*this.z;
    	// this.set(X, Y, Z); Not necessary to compute positions each time.
		this.x = X; this.y = Y; this.z = Z;
		this.lon = this.lat = 0./0.;
    },


    /**
     * Test equality of Coo.
     * @param coo Second coordinate to compare with
     * @return  True if the two coordinates are equal
     */
    equals: function(coo) {
		return this.x == coo.x && this.y == coo.y && this.z == coo.z;
    },

	/**
	 * parse a coordinate string. The coordinates can be in decimal or sexagesimal
	 * @param str string to parse
	 */
	parse: function(str) {
		var p = str.indexOf('+');
		if (p < 0) p = str.indexOf('-');
		if (p < 0) {
			this.lon = 0.0/0.0;
			this.lat = 0.0/0.0;
			this.prec = 0;
			return;
		}
		var strlon = str.substring(0,p);
		var strlat = str.substring(p);
		
		this.lon = this.parseLon(strlon);	// sets the precision parameter
		this.lat = this.parseLat(strlat);	// sets the precision parameter
	},

	/**
	 * Parse a longitude
	 */
	parseLon: function(str) {
		var str = Strings.trim(str, ' ');
		if (str.indexOf(' ') < 0) {
			// The longitude is a integer or decimal number
			var p = str.indexOf('.');
			this.prec = p < 0 ? 0 : str.length - p - 1;
			return parseFloat(str);
		} else {
			var stok = new Tokenizer(str,' ');
			var i = 0;
			var l = 0;
			var pr = 0;
			while (stok.hasMore()) {
				var tok = stok.nextToken();
				var dec = tok.indexOf('.');
				l += parseFloat(tok)*Coo.factor[i];
//				pr = dec < 0 ? 1 : 2;
				switch (i) {
					case 0: pr = dec < 0 ? 1 : 2; break;
					case 1: pr = dec < 0 ? 3 : 4; break;
					case 2: pr = dec < 0 ? 5 : 4+tok.length-dec;
					default: break;
				}
				i++;
			}
			this.prec = pr;
			return l*15/3600.0;	
		}
	},

	/**
	 * Parse a latitude
	 */
	parseLat: function(str) {
		var str = Strings.trim(str, ' ');
		var sign = str.charAt(0) == '+' ? 1 : -1;
		str = str.substring(1);
		if (str.indexOf(' ') < 0) {
			// The longitude is a integer or decimal number
			var p = str.indexOf('.');
			this.prec = p < 0 ? 0 : str.length - p - 1;
			return parseFloat(str)*sign;
		} else {
			var stok = new Tokenizer(str,' ');
			var i = 0;
			var l = 0;
			var pr = 0;
			while (stok.hasMore()) {
				var tok = stok.nextToken();
				var dec = tok.indexOf('.');
				l += parseFloat(tok)*Coo.factor[i];
				switch (i) {
					case 0: pr = dec < 0 ? 1 : 2; break;
					case 1: pr = dec < 0 ? 3 : 4; break;
					case 2: pr = dec < 0 ? 5 : 4+tok.length-dec;
					default: break;
				}
				i++;
			}
			this.prec = pr;
			return l*sign/3600.0;	
		}
	},

	/**
	 * Format coordinates according to the options
	 * @param options 'd': decimal, 's': sexagésimal, '/': space separated, '2': return [ra,dec] in an array
	 * @return the formatted coordinates
	 */
	format: function(options) {
		if (isNaN(this.lon)) this.computeLonLat();
		var strlon = "", strlat = "";
		if (options.indexOf('d') >= 0) {
			// decimal display
			strlon = Numbers.format(this.lon, this.prec);
			strlat = Numbers.format(this.lat, this.prec);
		} else {
			// sexagesimal display
			var hlon = this.lon/15.0;
			var strlon = Numbers.toSexagesimal(hlon, this.prec, false);
			var strlat = Numbers.toSexagesimal(this.lat, this.prec, true);
		}

		if (options.indexOf('/') >= 0) {
			return strlon+' '+strlat;
		} else if (options.indexOf('2') >= 0) {
			return [strlon, strlat];
		}
		return strlon+strlat;
	}
		
}


//===================================
// Class Tokenizer (similar to Java)
//===================================

/**
 * Constructor
 * @param str String to tokenize
 * @param sep token separator char
 */
function Tokenizer(str, sep) {
	this.string = Strings.trim(str, sep);
	this.sep = sep;
	this.pos = 0;
}

Tokenizer.prototype = {
	/**
	 * Check if the string has more tokens
	 * @return true if a token remains (read with nextToken())
	 */
	hasMore: function() {
		return this.pos < this.string.length;
	},

	/**
	 * Returns the next token (as long as hasMore() is true)
	 * @return the token string
	 */
	nextToken: function() {
		// skip all the separator chars
		var p0 = this.pos;
		while (p0 < this.string.length && this.string.charAt(p0) == this.sep) p0++;
		var p1 = p0;
		// get the token
		while (p1 < this.string.length && this.string.charAt(p1) != this.sep) p1++;
		this.pos = p1;
		return this.string.substring(p0, p1);
	},
}

//================================
// Class Strings (static methods)
//================================
function Strings() {}

/**
 * Removes a given char at the beginning and the end of a string
 * @param str string to trim
 * @param c char to remove
 * @return the trimmed string
 */

Strings.trim = function(str, c) {
	var p0=0, p1=str.length-1;
	while (p0 < str.length && str.charAt(p0) == c) p0++;
	if (p0 == str.length) return "";
	while (p1 > p0 && str.charAt(p1) == c) p1--;
	return str.substring(p0, p1+1);
}

//================================
// Class Numbers (static methods)
//================================
function Numbers() {}
//                0  1   2    3     4      5       6        7         8          9
Numbers.pow10 = [ 1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
//      10           11            12             13              14
	10000000000, 100000000000, 1000000000000, 10000000000000, 100000000000000 ];
//                 0    1     2      3       4        5         6          7
Numbers.rndval = [ 0.5, 0.05, 0.005, 0.0005, 0.00005, 0.000005, 0.0000005, 0.00000005,
//      8            9             10             11              12
	0.000000005, 0.0000000005, 0.00000000005, 0.000000000005, 0.0000000000005,
//      13                14
	0.00000000000005, 0.00000000000005 ];
/**
 * Format a integer or decimal number, adjusting the value with 'prec' decimal digits
 * @param num number (integer or decimal)
 * @param prec precision (= number of decimal digit to keep or append)
 * @return a string with the formatted number
 */
Numbers.format = function(num, prec) {
		if (prec <= 0) {
			// Return an integer number
			return (Math.round(num)).toString();
		}
		var str = num.toString();
		var p = str.indexOf('.');
		var nbdec = p >= 0 ? str.length-p-1 : 0;
		if (prec >= nbdec) {
			if (p < 0) str += '.';
			for (var i=0; i<prec-nbdec; i++)
				str += '0';
			return str;
		}
		// HERE: prec > 0 and prec < nbdec
		str = (num+Numbers.rndval[prec]).toString();
		return str.substr(0, p+prec+1);
}


/**
 * Convert a decimal coordinate into sexagesimal string, according to the given precision<br>
 * 8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 * @param num number (integer or decimal)
 * @param prec precision (= number of decimal digit to keep or append)
 * @param plus if true, the '+' sign is displayed
 * @return a string with the formatted sexagesimal number
 */
Numbers.toSexagesimal = function(num, prec, plus) {
	var resu = "";
	var sign = num < 0 ? '-' : (plus ? '+' : '');
	var n = Math.abs(num);

	switch (prec) {
		case 1:	// deg
			var n1 = Math.round(n);
			return sign+n1.toString();
		case 2:	// deg.d
			return sign+Numbers.format(n, 1);
		case 3:	// deg min
			var n1 = Math.floor(n);
			var n2 = Math.round((n-n1)*60);
			return sign+n1+' '+n2;
		case 4:	// deg min.d
			var n1 = Math.floor(n);
			var n2 = (n-n1)*60;
			return sign+n1+' '+Numbers.format(n2, 1);
		case 5:	// deg min sec
			var n1 = Math.floor(n);	// d
			var n2 = (n-n1)*60;		// M.d
			var n3 = Math.floor(n2);// M
			var n4 = Math.round((n2-n3)*60);	// S
			return sign+n1+' '+n3+' '+n4;
		case 6:	// deg min sec.d
		case 7:	// deg min sec.dd
		case 8:	// deg min sec.ddd
			var n1 = Math.floor(n);	// d
			if (n1<10) n1 = '0' + n1;
			var n2 = (n-n1)*60;		// M.d
			var n3 = Math.floor(n2);// M
			if (n3<10) n3 = '0' + n3;
			var n4 = (n2-n3)*60;		// S.ddd
			return sign+n1+' '+n3+' '+Numbers.format(n4, prec-5);
		default:
			return sign+Numbers.format(n, 1);
	}
}
CooConversion = (function() {

    var CooConversion = {};
    
    CooConversion.GALACTIC_TO_J2000 = [
       -0.0548755604024359,  0.4941094279435681, -0.8676661489811610,
       -0.8734370902479237, -0.4448296299195045, -0.1980763734646737,
       -0.4838350155267381,  0.7469822444763707,  0.4559837762325372 ];
    
    CooConversion.J2000_TO_GALACTIC = [
        -0.0548755604024359, -0.873437090247923, -0.4838350155267381,
         0.4941094279435681, -0.4448296299195045, 0.7469822444763707,
        -0.8676661489811610, -0.1980763734646737, 0.4559837762325372 ];
    
    // adapted from www.robertmartinayers.org/tools/coordinates.html
    // radec : array of ra, dec in degrees
    // return coo in degrees
    CooConversion.Transform = function( radec, matrix ) {// returns a radec array of two elements
        radec[0] = radec[0]*Math.PI/180;
        radec[1] = radec[1]*Math.PI/180;
      var r0 = new Array ( 
       Math.cos(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[0]) * Math.cos(radec[1]),
       Math.sin(radec[1]) );
        
     var s0 = new Array (
       r0[0]*matrix[0] + r0[1]*matrix[1] + r0[2]*matrix[2], 
       r0[0]*matrix[3] + r0[1]*matrix[4] + r0[2]*matrix[5], 
       r0[0]*matrix[6] + r0[1]*matrix[7] + r0[2]*matrix[8] ); 
     
      var r = Math.sqrt ( s0[0]*s0[0] + s0[1]*s0[1] + s0[2]*s0[2] ); 
    
      var result = new Array ( 0.0, 0.0 );
      result[1] = Math.asin ( s0[2]/r ); // New dec in range -90.0 -- +90.0 
      // or use sin^2 + cos^2 = 1.0  
      var cosaa = ( (s0[0]/r) / Math.cos(result[1] ) );
      var sinaa = ( (s0[1]/r) / Math.cos(result[1] ) );
      result[0] = Math.atan2 (sinaa,cosaa);
      if ( result[0] < 0.0 ) result[0] = result[0] + 2*Math.PI;
    
        result[0] = result[0]*180/Math.PI;
        result[1] = result[1]*180/Math.PI;
      return result;
    };
    
    // coo : array of lon, lat in degrees
    CooConversion.GalacticToJ2000 = function(coo) {
        return CooConversion.Transform(coo, CooConversion.GALACTIC_TO_J2000);
    };
    // coo : array of lon, lat in degrees
    CooConversion.J2000ToGalactic = function(coo) {
        return CooConversion.Transform(coo, CooConversion.J2000_TO_GALACTIC);
    };
    return CooConversion;
})();/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Sesame = (function() {
    Sesame = {};
    
    Sesame.cache = {};
    
    Sesame.resolve = function(objectName, callbackFunctionSuccess, callbackFunctionError) {
        //var sesameUrl = "http://cdsportal.u-strasbg.fr/services/sesame?format=json";
        var sesameUrl = "http://cds.u-strasbg.fr/cgi-bin/nph-sesame.jsonp?";
        $.ajax({
            url: sesameUrl ,
            data: {"object": objectName},
            method: 'GET',
            dataType: 'jsonp',
            success: callbackFunctionSuccess,
            error: callbackFunctionError
            });
    };
    
    return Sesame;
})();

/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HealpixCache
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

// class holding some HEALPix computations for better performances
//
// it is made of :
// - a static cache for HEALPix corners at nside=8 
// - a dynamic cache for 
HealpixCache = (function() {

    var HealpixCache = {};
    
    HealpixCache.staticCache = {corners: {nside8: []}};
    // TODO : utilisation du dynamicCache
    HealpixCache.dynamicCache = {};
    
    HealpixCache.lastNside = 8;
    
    HealpixCache.hpxIdxCache = null;
    
    // TODO : conserver en cache le dernier résultat ?
    
    HealpixCache.init = function() {
    	// pre-compute corners position for nside=8
    	var hpxIdx = new HealpixIndex(8);
    	hpxIdx.init();
    	var npix = HealpixIndex.nside2Npix(8);
    	for (var ipix=0; ipix<npix; ipix++) {
    		HealpixCache.staticCache.corners.nside8[ipix] = hpxIdx.corners_nest(ipix, 1);
    	}
    	
    	HealpixCache.hpxIdxCache = hpxIdx;
    };
    
    HealpixCache.corners_nest = function(ipix, nside) {
    	if (nside==8) {
    		return HealpixCache.staticCache.corners.nside8[ipix];
    	}
    	
    	if (nside != HealpixCache.lastNside) {
    		HealpixCache.hpxIdxCache = new HealpixIndex(nside);
    		HealpixCache.hpxIdxCache.init();
    		HealpixCache.lastNside = nside;
    	}
    	
    	return HealpixCache.hpxIdxCache.corners_nest(ipix, 1);
    	
    };
    
    return HealpixCache;
})();
	
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Utils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Utils = Utils || {};

// adding relMouseCoords to HTMLCanvasElement prototype (see http://stackoverflow.com/questions/55677/how-do-i-get-the-coordinates-of-a-mouse-click-on-a-canvas-element ) 
function relMouseCoords(event){
    var totalOffsetX = 0;
    var totalOffsetY = 0;
    var canvasX = 0;
    var canvasY = 0;
    var currentElement = this;

    do {
        totalOffsetX += currentElement.offsetLeft - currentElement.scrollLeft;
        totalOffsetY += currentElement.offsetTop - currentElement.scrollTop;
    }
    while(currentElement = currentElement.offsetParent)

    if (event.pageX) {
        canvasX = event.pageX - totalOffsetX - document.body.scrollLeft;
        canvasY = event.pageY - totalOffsetY - document.body.scrollTop;
    }
    // if touch events
    else {
        canvasX = event.originalEvent.targetTouches[0].screenX - totalOffsetX - document.body.scrollLeft;
        canvasY = event.originalEvent.targetTouches[0].screenY - totalOffsetY - document.body.scrollTop;    	
    }

    return {x:canvasX, y:canvasY};
}
HTMLCanvasElement.prototype.relMouseCoords = relMouseCoords;

$ = $ || jQuery;

/* source : http://stackoverflow.com/a/8764051 */
$.urlParam = function(name, queryString){
    if (queryString===undefined) {
        queryString = location.search;
    }
	return decodeURIComponent((new RegExp('[?|&]' + name + '=' + '([^&;]+?)(&|#|;|$)').exec(queryString)||[,""])[1].replace(/\+/g, '%20'))||null;
};/******************************************************************************
 * Aladin HTML5 project
 * 
 * File AladinUtils
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
AladinUtils = (function() {
    return {
    	/**
    	 * passage de xy projection à xy dans la vue écran 
    	 * @param x
    	 * @param y
    	 * @param width
    	 * @param height
    	 * @param largestDim largest dimension of the view
    	 * @returns position in the view
    	 */
    	xyToView: function(x, y, width, height, largestDim, zoomFactor) {
    		// we round the result for performance gains
    		return {vx: AladinUtils.myRound(largestDim/2*(1+zoomFactor*x)-(largestDim-width)/2), vy: AladinUtils.myRound(largestDim/2*(1+zoomFactor*y)-(largestDim-height)/2)};
    	},
    	
    	/**
    	 * passage de xy dans la vue écran à xy projection
    	 * @param vx
    	 * @param vy
    	 * @param width
    	 * @param height
    	 * @param largestDim
    	 * @param zoomFactor
    	 * @returns position in xy projection
    	 */
    	viewToXy: function(vx, vy, width, height, largestDim, zoomFactor) {
    		return {x: ((2*vx+(largestDim-width))/largestDim-1)/zoomFactor, y: ((2*vy+(largestDim-height))/largestDim-1)/zoomFactor};
    	},
    	
    	myRound: function(a) {
    		if (a<0) {
    			return -1*( (-a) | 0);
    		}
    		else {
    			return a | 0;
    		}
    	},
    	
    	/**
    	 * tests whether a healpix pixel is visible or not
    	 * @param pixCorners array of position (xy view) of the corners of the pixel
    	 * @param viewW
    	 */
    	isHpxPixVisible: function(pixCorners, viewWidth, viewHeight) {
    		for (var i = 0; i<pixCorners.length; i++) {
    			if ( pixCorners[i].vx>=-20 && pixCorners[i].vx<(viewWidth+20) &&
    				 pixCorners[i].vy>=-20 && pixCorners[i].vy<(viewHeight+20) ) {
    				return true;
    			}
    		}
    		return false;
    	},
    	
    	ipixToIpix: function(npixIn, norderIn, norderOut) {
    		var npixOut = [];
    		if (norderIn>=norderOut) {
    		}
    	},
        
        getZoomFactorForAngle: function(angleInDegrees, projectionMethod) {
            var p1 = {ra: 0, dec: 0};
            var p2 = {ra: angleInDegrees, dec: 0};
            var projection = new Projection(angleInDegrees/2, 0);
            projection.setProjection(projectionMethod);
            var p1Projected = projection.project(p1.ra, p1.dec);
            var p2Projected = projection.project(p2.ra, p2.dec);
           
            var zoomFactor = 1/(p1Projected.X - p2Projected.Y);
            return zoomFactor;
        }
    	
    };
})();

/******************************************************************************
 * Aladin HTML5 project
 * 
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
 ProjectionEnum = {
    SIN: Projection.PROJ_SIN,
    AITOFF:  Projection.PROJ_AITOFF
 };/******************************************************************************
 * Aladin HTML5 project
 * 
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
CooFrameEnum = (function() {

    return {
        J2000: "J2000",
        GAL:  "Galactic"
    };
 
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Downloader
 * Queue downloading for image elements
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Downloader = (function() {
	var NB_MAX_SIMULTANEOUS_DL = 4;
	// TODO : le fading ne marche pas bien actuellement
	var FADING_ENABLED = false;
	var FADING_DURATION = 700; // in milliseconds
	
	
	var Downloader = function(view) {
		this.view = view; // reference to the view to be able to request redraw
		this.nbDownloads = 0; // number of current downloads
		this.dlQueue = []; // queue of items being downloaded
        this.urlsInQueue = {};
	};
	
	Downloader.prototype.requestDownload = function(img, url) {
        // first check if url already in queue
        if (url in this.urlsInQueue)  {
            return;
        }
		// put in queue
		this.dlQueue.push({img: img, url: url});
        this.urlsInQueue[url] = 1;
		
		this.tryDownload();
	};
	
	// try to download next items in queue if possible
	Downloader.prototype.tryDownload = function() {
	    //if (this.dlQueue.length>0 && this.nbDownloads<NB_MAX_SIMULTANEOUS_DL) {
		while (this.dlQueue.length>0 && this.nbDownloads<NB_MAX_SIMULTANEOUS_DL) {
			this.startDownloadNext();
		}
	};
	
	Downloader.prototype.startDownloadNext = function() {
		// get next in queue
		var next = this.dlQueue.shift();
		if ( ! next) {
			return;
		}

		
		this.nbDownloads++;
		var downloaderRef = this;
		next.img.onload = function() {
			downloaderRef.completeDownload(this, true); // in this context, 'this' is the Image
		};
			
		next.img.onerror = function() {

			downloaderRef.completeDownload(this, false); // in this context, 'this' is the Image
		};
		
		next.img.src = next.url;
	};
	
	Downloader.prototype.completeDownload = function(img, success) {
        delete this.urlsInQueue[img.src];
		img.onerror = null;
		img.onload = null;
		this.nbDownloads--;
		if (success) {
			if (FADING_ENABLED) {
				var now = new Date().getTime();
				img.fadingStart = now;
				img.fadingEnd = now + FADING_DURATION;
			}
			this.view.requestRedraw();
		}
		
		this.tryDownload();
	};
	
	
	
	return Downloader;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Source
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Source = (function() {
    // constructor
    function Source(ra, dec, mesures) {
    	this.ra = ra;
    	this.dec = dec;
    	this.mesures = mesures;
    	this.catalog = null;
    	
    	this.isShowing = true;
    	this.isSelected = false;
    };
    
    Source.prototype.setCatalog = function(catalog) {
        this.catalog = catalog;
    };
    
    Source.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    Source.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    Source.prototype.select = function() {
        if (this.isSelected) {
            return;
        }
        this.isSelected = true;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    Source.prototype.deselect = function() {
        if (! this.isSelected) {
            return;
        }
        this.isSelected = false;
        if (this.catalog) {
            this.catalog.reportChange();
        }
    };
    
    return Source;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Catalog = (function() {
    var Catalog = function(name, url, color, sourceSize) {
    	this.name = name;
    	this.url = url;
    	
    	this.indexationNorder = 5; // à quel niveau indexe-t-on les sources
    	this.sources = [];
    	this.color = color ? color : "#ff0000";
    	this.hpxIdx = new HealpixIndex(this.indexationNorder);
    	this.hpxIdx.init();
    	this.selectionColor = '#00ff00';
    	
    	this.sourceSize = sourceSize ? sourceSize : 5;
    };
    
    Catalog.prototype.addSources = function(sourcesToAdd) {
    	this.sources = this.sources.concat(sourcesToAdd);
    	for (var k=0, len=sourcesToAdd.length; k<len; k++) {
    	    sourcesToAdd[k].setCatalog(this);
    	}
        this.view.requestRedraw();
    };
    
    // return a source by index
    Catalog.prototype.getSource = function(idx) {
        if (idx<this.sources.length) {
            return this.sources[idx];
        }
        else {
            return null;
        }
    };
    
    Catalog.prototype.setView = function(view) {
        this.view = view;
    };
    
    Catalog.prototype.removeAll = function() {
        // TODO : RAZ de l'index
        this.sources = [];
    };
    
    Catalog.prototype.draw = function(ctx, projection, frame, width, height, largestDim, zoomFactor) {
        // tracé simple
    	//ctx.fillStyle = this.color;
        ctx.strokeStyle= this.color;

        ctx.lineWidth = 1;
    	ctx.beginPath();
    	xyviews = [];
    	for (var k=0, len = this.sources.length; k<len; k++) {
    		xyviews.push(this.drawSource(this.sources[k], ctx, projection, frame, width, height, largestDim, zoomFactor));
    		
    	}
        ctx.stroke();

    	// tracé sélection
        ctx.strokeStyle= this.selectionColor;
        ctx.beginPath();
        for (var k=0, len = this.sources.length; k<len; k++) {
            if (! this.sources[k].isSelected) {
                continue;
            }
            this.drawSourceSelection(ctx, xyviews[k]);
            
        }
    	ctx.stroke();
    };
    
    
    
    Catalog.prototype.drawSource = function(s, ctx, projection, frame, width, height, largestDim, zoomFactor) {
        if (! s.isShowing) {
            return;
        }
        var sourceSize = this.sourceSize;
        var xy;
        var xyview = null;
        if (frame!=CooFrameEnum.J2000) {
            var lonlat = CooConversion.J2000ToGalactic([s.ra, s.dec]);
            xy = projection.project(lonlat[0], lonlat[1]);
        }
        else {
            xy = projection.project(s.ra, s.dec);
        }
        if (xy) {
            xyview = AladinUtils.xyToView(xy.X, xy.Y, width, height, largestDim, zoomFactor);
            if (xyview) {
                ctx.moveTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
                ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy-sourceSize/2);
                ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
                ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy+sourceSize/2);
                ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
                //ctx.fillRect(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2, sourceSize, sourceSize);
            }
        }
        
        return xyview;
    };
    
    Catalog.prototype.drawSourceSelection = function(ctx, xyview) {
        if (!xyview) {
            return;
        }
        var sourceSize = this.sourceSize+2;
        ctx.moveTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
        ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy+sourceSize/2);
        ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy+sourceSize/2);
        ctx.lineTo(xyview.vx+sourceSize/2, xyview.vy-sourceSize/2);
        ctx.lineTo(xyview.vx-sourceSize/2, xyview.vy-sourceSize/2);
    };

    
    // callback function to be called when the status of one of the sources has changed
    Catalog.prototype.reportChange = function() {
        this.view.requestRedraw();
    };

    return Catalog;
})();/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Tile
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Tile = (function() {
    // constructor
	function Tile(img, url) {
		this.img = img;
		this.url = url;
	};
	
	// check whether the image corresponding to the tile is loaded and ready to be displayed
	//
	// source : http://www.sajithmr.me/javascript-check-an-image-is-loaded-or-not
	Tile.isImageOk = function(img) {
		if (img.allSkyTexture) {
			return true;
		}
		
        if (!img.src) {
            return false;
        }

	    // During the onload event, IE correctly identifies any images that
	    // weren’t downloaded as not complete. Others should too. Gecko-based
	    // browsers act like NS4 in that they report this incorrectly.
	    if (!img.complete) {
	        return false;
	    }

	    // However, they do have two very useful properties: naturalWidth and
	    // naturalHeight. These give the true size of the image. If it failed
	    // to load, either of these should be zero.

	    if (typeof img.naturalWidth != "undefined" && img.naturalWidth == 0) {
	        return false;
	    }

	    // No other way of checking: assume it’s ok.
	    return true;
	};
	
	return Tile;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File TileBuffer
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

TileBuffer = (function() {
	var NB_MAX_TILES = 1000; // buffer size
	
	// constructor
	function TileBuffer() {
		this.pointer = 0;
		this.tilesMap = {};
		this.tilesArray = new Array(NB_MAX_TILES);

		for (var i=0; i<NB_MAX_TILES; i++) {
			this.tilesArray[i] = new Tile(new Image(), null, null);
		}
	};
	
	TileBuffer.prototype.addTile = function(url) {
	    // return null if already in buffer
        if (this.getTile(url)) {
            return null;
        }

        // delete existing tile
        var curTile = this.tilesArray[this.pointer];
        curTile.img.src = null;
        delete this.tilesMap[curTile.url];

        this.tilesArray[this.pointer].url = url;
        this.tilesMap[url] = this.tilesArray[this.pointer];

        this.pointer++;
        if (this.pointer>=NB_MAX_TILES) {
            this.pointer = 0;
        }

        return this.tilesMap[url];
	};
	
	TileBuffer.prototype.getTile = function(url) {
        return this.tilesMap[url];
		
	};
	
	return TileBuffer;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HpxImageSurvey
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HpxImageSurvey = (function() {


    var HpxImageSurvey = function(name, rootUrl, cooFrame, maxOrder, tileBuffer) {
    	this.name = name;
    	this.rootUrl = rootUrl;
        this.cooFrame = cooFrame;
        this.maxOrder = maxOrder;
    	
    	this.allskyTextures = [];
    	
    
    	this.allskyTextureSize = 0;
    
        this.lastUpdateDateNeededTiles = 0;
        
    };
    
    HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY = 1000; // in milliseconds
    
    HpxImageSurvey.prototype.init = function(view) {
    	this.view = view;
    	
    	//this.tileBuffer = new TileBuffer();
    	// tileBuffer is now shared across different image surveys
    	this.tileBuffer = view.tileBuffer;
    
    	this.retrieveAllskyTextures();
    };
    
    HpxImageSurvey.getAvailableSurveys = function() {
    	return [
    			{name: "DSS Red", url: "http://alasky.u-strasbg.fr/DSS/DSS2Merged", frame: CooFrameEnum.J2000, maxOrder: 9},
    			{name: "DSS Color", url: "http://alasky.u-strasbg.fr/DssColor", frame: CooFrameEnum.J2000, maxOrder: 9},
    			{name: "Mellinger RGB", url: "http://alasky.u-strasbg.fr/MellingerRGB", frame: CooFrameEnum.GAL, maxOrder: 4},
    			{name: "SDSS DR7 Color", url: "http://alasky.u-strasbg.fr/SDSS/Color", frame: CooFrameEnum.J2000, maxOrder: 10},
    			{name: "GALEX AIS Color", url: "http://alasky.u-strasbg.fr/GALEX/GR6-02-Color", frame: CooFrameEnum.J2000, maxOrder: 8},
    			{name: "IRAC Color", url: "http://alasky.u-strasbg.fr/SpitzerI1I2I4color", frame: CooFrameEnum.GAL, maxOrder: 9},
    			{name: "IRAS-IRIS Color", url: "http://alasky.u-strasbg.fr/IRISColor", frame: CooFrameEnum.GAL, maxOrder: 3},
    			{name: "WISE W1 low-res", url: "http://alasky.u-strasbg.fr/WISE/W1", frame: CooFrameEnum.GAL, maxOrder: 5},
    			{name: "2MASS Color", url: "http://alasky.u-strasbg.fr/2MASS/Color", frame: CooFrameEnum.J2000, maxOrder: 9},
    			{name: "Halpha composite map", url: "http://alasky.u-strasbg.fr/FinkbeinerHalpha", frame: CooFrameEnum.GAL, maxOrder: 3},
    			{name: "Planck HFI color", url: "http://alasky.u-strasbg.fr/PLANCK/HFIColor353-545-857", frame: CooFrameEnum.GAL, maxOrder: 3},
    			{name: "XMM-Newton", url: "http://saada.u-strasbg.fr/xmmallsky", frame: CooFrameEnum.GAL, maxOrder: 7},
    			{name: "GALEX NUV", url: "http://alasky.u-strasbg.fr/GALEX/GR6-01-ND", frame: CooFrameEnum.J2000, maxOrder: 8}
    			//{name: "SDSS DR9", url: "http://alaskybis.u-strasbg.fr/SDSS-DR9", frame: CooFrameEnum.J2000, maxOrder: 10}
    
    			
    	];
    };
    
    HpxImageSurvey.getSurveyInfoFromName = function(name) {
        var surveys = HpxImageSurvey.getAvailableSurveys();
        for (var i=0; i<surveys.length; i++) {
            if (surveys[i].name==name) {
                return surveys[i];
            }
        }
        return null;
    };
    
    HpxImageSurvey.getSurveyFromName = function(name) {
    	var info = HpxImageSurvey.getSurveyInfoFromName(name);
    	if (info) {
    		return new HpxImageSurvey(info.name, info.url, info.frame, info.maxOrder);
    	}
    	
    	return null;
    };
    
    HpxImageSurvey.prototype.getTileURL = function(norder, npix) {
    	var dirIdx = Math.floor(npix/10000)*10000;
    	return this.rootUrl + "/" + "Norder" + norder + "/Dir" + dirIdx + "/Npix" + npix + ".jpg";
    };
    
    HpxImageSurvey.prototype.retrieveAllskyTextures = function() {
    	// start loading of allsky
    	var img = new Image();
    	var survey = this;
    	img.onload = function() {
    		// sur ipad, le fichier qu'on récupère est 2 fois plus petit. Il faut donc déterminer la taille de la texture dynamiquement
    		survey.allskyTextureSize = img.width/27;
    
    		// récupération des 768 textures (NSIDE=4)
    		for (var j=0; j<29; j++) {
    			for (var i=0; i<27; i++) {
    				var c = document.createElement('canvas');
    				c.width = c.height = survey.allskyTextureSize;
    				c.allSkyTexture = true;
    				var context = c.getContext('2d');
    				context.drawImage(img, i*survey.allskyTextureSize, j*survey.allskyTextureSize, survey.allskyTextureSize, survey.allskyTextureSize, 0, 0, c.width, c.height);
    				survey.allskyTextures.push(c);
    			}
    		}
    		survey.view.requestRedraw();
    	};
    	img.src = this.rootUrl + '/Norder3/Allsky.jpg';
    
    };
    
    HpxImageSurvey.prototype.redrawAllsky = function(ctx, cornersXYViewMap, fov, norder) {
    	// for norder deeper than 6, we think it brings nothing to draw the all-sky
    	if (view.curNorder>6) {
    		return;
    	}
    	
    	if ( ! this.allskyTextures ) {
    		return;
    	}
    	
        var cornersXYView;
        var coeff = 0;
        var center;
        var ipix;
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;
    		
            if ( ! this.allskyTextures[ipix] || !Tile.isImageOk(this.allskyTextures[ipix]) ) {
                continue;
            }
    		
    
    		// TODO : plutot agrandir le clip ?
    	    // grow cornersXYView
    	    if (fov>40) {
    			coeff = 0.02;
    	        center = {x: (cornersXYView[0].vx+cornersXYView[2].vx)/2, y: (cornersXYView[0].vy+cornersXYView[2].vy)/2};
    	        for (var i=0; i<4; i++) {
    	            var diff = {x: cornersXYView[i].vx-center.x, y: cornersXYView[i].vy-center.y};
    	            cornersXYView[i].vx += coeff*diff.x;
    	            cornersXYView[i].vy += coeff*diff.y;
    	        }
    	    }
    			
    	    this.drawOneTile(ctx, this.allskyTextures[ipix], cornersXYView, this.allskyTextureSize);
    	}
    };
    
    // TODO: avoir un mode où on ne cherche pas à dessiner d'abord les tuiles parentes (pour génération vignettes côté serveur)
    HpxImageSurvey.prototype.redrawHighres = function(ctx, cornersXYViewMap, norder) {
        var now = new Date().getTime();
        var updateNeededTiles = (now-this.lastUpdateDateNeededTiles) > HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY;
        var tile, url, parentTile, parentUrl;
        var parentNorder = norder - 1;
        var cornersXYView, parentCornersXYView;
        var tilesToDraw = [];
        var parentTilesToDraw = [];
        var parentTilesToDrawIpix = {};
        var missingTiles = false;
        
        var tilesToDownload = [];
        var parentTilesToDownload = [];
        
        var parentIpix;
        var ipix;
        
        // tri des tuiles selon la distance
        if (updateNeededTiles) {
            var center = [(cornersXYViewMap[0][0].vx+cornersXYViewMap[0][1].vx)/2, (cornersXYViewMap[0][0].vy+cornersXYViewMap[0][1].vy)/2];
            var newCornersXYViewMap = cornersXYViewMap.sort(function(a, b) {
                var cA = [(a[0].vx+a[2].vx)/2, (a[0].vy+a[2].vy)/2];
                var cB = [(b[0].vx+b[2].vx)/2, (b[0].vy+b[2].vy)/2]; 

                var distA = (cA[0]-center[0])*(cA[0]-center[0]) + (cA[1]-center[1])*(cA[1]-center[1]);
                var distB = (cB[0]-center[0])*(cB[0]-center[0]) + (cB[1]-center[1])*(cB[1]-center[1]);
                
                return distA-distB;
                    
            });
            cornersXYViewMap = newCornersXYViewMap;
        }
        
    	for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
    		cornersXYView = cornersXYViewMap[k];
    		ipix = cornersXYView.ipix;
            
            // on demande à charger le parent (cas d'un zoomOut)
            // TODO : mettre priorité plus basse
            parentIpix = ~~(ipix/4);
        	parentUrl = this.getTileURL(parentNorder, parentIpix);
            if (updateNeededTiles && parentNorder>=3) {
            	parentTile = this.tileBuffer.addTile(parentUrl);
                if (parentTile) {
                    parentTilesToDownload.push({img: parentTile.img, url: parentUrl});
                    //this.view.downloader.requestDownload(parentTile.img, parentUrl);
                }
            }
            
            url = this.getTileURL(norder, ipix);
            tile = this.tileBuffer.getTile(url);
            
            if ( ! tile ) {
                missingTiles = true;
                
                if (updateNeededTiles) {
                    var tile = this.tileBuffer.addTile(url);
                    if (tile) {
                        tilesToDownload.push({img: tile.img, url: url});
                        //this.view.downloader.requestDownload(tile.img, url);
                    }
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
    
                continue;
            }
            else if ( ! Tile.isImageOk(tile.img)) {
                missingTiles = true;
                if (updateNeededTiles) {
                    tilesToDownload.push({img: tile.img, url: url});
                    //this.view.downloader.requestDownload(tile.img, url);
                }
                
                // is the parent tile available ?
                if (parentNorder>=3 && ! parentTilesToDrawIpix[parentIpix]) {
                	parentTile = this.tileBuffer.getTile(parentUrl);
                	if (parentTile && Tile.isImageOk(parentTile.img)) {
                		parentCornersXYView = view.getPositionsInView(parentIpix, parentNorder);
                		if (parentCornersXYView) {
                			parentTilesToDraw.push({img: parentTile.img, corners: parentCornersXYView, ipix: parentIpix});
                		}
                	}
                	parentTilesToDrawIpix[parentIpix] = 1;
                }
                
                continue;
            }
            tilesToDraw.push({img: tile.img, corners: cornersXYView});
        }
    	
    
    
        // draw parent tiles
        for (var k=0, len = parentTilesToDraw.length; k<len; k++) {
        	this.drawOneTile(ctx, parentTilesToDraw[k].img, parentTilesToDraw[k].corners, parentTilesToDraw[k].img.width);
        }
        // draw tiles
        for (var k=0, len = tilesToDraw.length; k<len; k++) {
        	var alpha = null;
        	var img = tilesToDraw[k].img;
        	if (img.fadingStart) {
        		if (img.fadingEnd && now<img.fadingEnd) {
        			alpha = 0.2 + (now - img.fadingStart)/(img.fadingEnd - img.fadingStart)*0.8;
        		}
        	}
        	this.drawOneTile(ctx, tilesToDraw[k].img, tilesToDraw[k].corners, tilesToDraw[k].img.width, alpha);
        }
    
    
        // demande de chargement des tuiles manquantes et mise à jour lastUpdateDateNeededTiles
        if (updateNeededTiles) {
            // demande de chargement des tuiles
            for (var k=0, len = tilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(tilesToDownload[k].img, tilesToDownload[k].url);
            }
            //demande de chargement des tuiles parentes
            for (var k=0, len = parentTilesToDownload.length; k<len; k++) {
                this.view.downloader.requestDownload(parentTilesToDownload[k].img, parentTilesToDownload[k].url);
            }
            this.lastUpdateDateNeededTiles = now;
        }
        if (missingTiles) {
            // callback pour redemander un display dans 1000ms
            this.view.requestRedrawAtDate(now+HpxImageSurvey.UPDATE_NEEDED_TILES_DELAY+10);
        }
    };
    
    function dist2(x1,y1,x2,y2) {
    	return Math.pow(x2-x1, 2) + Math.pow(y2-y1, 2);
    }
    
    HpxImageSurvey.prototype.drawOneTile = function(ctx, img, cornersXYView, textureSize, alpha) {
    	// is the tile a diamond ?
    //	var round = AladinUtils.myRound;
    //	var b = cornersXYView;
    //	var flagDiamond =  round(b[0].vx - b[2].vx) == round(b[1].vx - b[3].vx)
    //    				&& round(b[0].vy - b[2].vy) == round(b[1].vy - b[3].vy); 
    	
    	                  
    	
    	
    	drawTexturedTriangle(ctx, img,
                cornersXYView[0].vx, cornersXYView[0].vy,
                cornersXYView[1].vx, cornersXYView[1].vy,
    	        cornersXYView[3].vx, cornersXYView[3].vy,
    	        textureSize-1, textureSize-1,
    	        textureSize-1, 0,
    	        0, textureSize-1,
    	        alpha);
        drawTexturedTriangle(ctx, img,
        		cornersXYView[1].vx, cornersXYView[1].vy,
        		cornersXYView[3].vx, cornersXYView[3].vy,
        		cornersXYView[2].vx, cornersXYView[2].vy,
        		textureSize-1, 0,
        		0, textureSize-1,
        		0, 0,
        		alpha);
    };
    
    
    	        
    
    // uses affine texture mapping to draw a textured triangle
    // at screen coordinates [x0, y0], [x1, y1], [x2, y2] from
    // img *pixel* coordinates [u0, v0], [u1, v1], [u2, v2]
    // code from http://www.dhteumeuleu.com/lab/image3D.html
    function drawTexturedTriangle(ctx, img, x0, y0, x1, y1, x2, y2,
                                        u0, v0, u1, v1, u2, v2, alpha) {
    	
        // ---- centroid ----
        var xc = (x0 + x1 + x2) / 3;
        var yc = (y0 + y1 + y2) / 3;
        ctx.save();
        if (alpha) {
        	ctx.globalAlpha = alpha;
        }
    
        ctx.beginPath();
        var coeff = 0.05;
        // ---- scale triangle by 1.05 to remove anti-aliasing and draw ----
        ctx.moveTo(((1+coeff) * x0 - xc * coeff), ((1+coeff) * y0 - yc * coeff));
        ctx.lineTo(((1+coeff) * x1 - xc * coeff), ((1+coeff) * y1 - yc * coeff));
        ctx.lineTo(((1+coeff) * x2 - xc * coeff), ((1+coeff) * y2 - yc * coeff));
        ctx.closePath();
        ctx.clip();
        // ---- transform texture ----
        var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
        ctx.transform(
            -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
             (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
             (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
            -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
             (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
             (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
        );
        //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); // faster ??
        ctx.drawImage(img, 0, 0); // slower ??
        
    //    ctx.globalAlpha = 1.0;
    
        ctx.restore();
    }
    
    /*
    function drawTexturedTriangle4Points(ctx, img, x0, y0, x1, y1, x2, y2,
            u0, v0, u1, v1, u2, v2) {
    
    	var x3 = x1+x2-x0;
    	var y3 = y1+y2-y0;
    // ---- centroid ----
    var xc = (x0 + x1 + x2 + x3) / 4;
    var yc = (y0 + y1 + y2 + y3) / 4;
    ctx.save();
    ctx.beginPath();
    // ---- scale triagle by 1.05 to remove anti-aliasing and draw ----
    ctx.moveTo((1.05 * x0 - xc * 0.05), (1.05 * y0 - yc * 0.05));
    ctx.lineTo((1.05 * x1 - xc * 0.05), (1.05 * y1 - yc * 0.05));
    ctx.lineTo((1.05 * x3 - xc * 0.05), (1.05 * y3 - yc * 0.05));
    ctx.lineTo((1.05 * x2 - xc * 0.05), (1.05 * y2 - yc * 0.05));
    ctx.closePath();
    ctx.clip();
    // ---- transform texture ----
    var d_inv = 1/ (u0 * (v2 - v1) - u1 * v2 + u2 * v1 + (u1 - u2) * v0);
    ctx.transform(
    -(v0 * (x2 - x1) -  v1 * x2  + v2 *  x1 + (v1 - v2) * x0) * d_inv, // m11
    (v1 *  y2 + v0  * (y1 - y2) - v2 *  y1 + (v2 - v1) * y0) * d_inv, // m12
    (u0 * (x2 - x1) -  u1 * x2  + u2 *  x1 + (u1 - u2) * x0) * d_inv, // m21
    -(u1 *  y2 + u0  * (y1 - y2) - u2 *  y1 + (u2 - u1) * y0) * d_inv, // m22
    (u0 * (v2 * x1  -  v1 * x2) + v0 * (u1 *  x2 - u2  * x1) + (u2 * v1 - u1 * v2) * x0) * d_inv, // dx
    (u0 * (v2 * y1  -  v1 * y2) + v0 * (u1 *  y2 - u2  * y1) + (u2 * v1 - u1 * v2) * y0) * d_inv  // dy
    );
    //ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, img.width, img.height); // faster ??
    ctx.drawImage(img, 0, 0); // slower ??
    
    ctx.restore();
    }
    */
    
    function grow(b, val)  {
    
    	  var b1 = new Array(b.length);
    	  for( var i=0; i<4; i++ ) {
    		  b1[i] = {vx: b[i].vx, vy: b[i].vy};
    	  }
    
    	  for( var i=0; i<2; i++ ) {
    	     var a= i==1 ? 1 : 0;
    	     var c= i==1 ? 2 : 3;
    
    	     var angle = Math.atan2(b1[c].vy-b1[a].vy, b1[c].vx-b1[a].vx);
    	     var chouilla = val*Math.cos(angle);
    	     b1[a].vx -= chouilla;
    	     b1[c].vx += chouilla;
    	     chouilla = val*Math.sin(angle);
    	     b1[a].vy -= chouilla;
    	     b1[c].vy += chouilla;
    	  }
      return b1;
    }

    return HpxImageSurvey;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File HealpixGrid
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

HealpixGrid = (function() {
	var HealpixGrid = function() {
	};
	
	HealpixGrid.prototype.redraw = function(ctx, cornersXYViewMap, fov, norder) {
		// on dessine les lignes
		ctx.lineWidth = 1;
		ctx.strokeStyle = "rgb(100,100,200)";
		ctx.beginPath();
		var cornersXYView;
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;
			
			// draw pixel
			ctx.moveTo(cornersXYView[0].vx, cornersXYView[0].vy);
			ctx.lineTo(cornersXYView[1].vx, cornersXYView[1].vy);
			ctx.lineTo(cornersXYView[2].vx, cornersXYView[2].vy);
			//ctx.lineTo(cornersXYView[3].vx, cornersXYView[3].vy);
			

            //ctx.strokeText(ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
		
		// on dessine les numéros de pixel HEALpix
        ctx.strokeStyle="#FFDDDD";
		ctx.beginPath();
		for (var k=0, len=cornersXYViewMap.length; k<len; k++) {
			cornersXYView = cornersXYViewMap[k];
			ipix = cornersXYView.ipix;

            ctx.strokeText(norder + '/' + ipix, (cornersXYView[0].vx + cornersXYView[2].vx)/2, (cornersXYView[0].vy + cornersXYView[2].vy)/2);
		}
		ctx.stroke();
	};

	
	
	return HealpixGrid;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Location = (function() {
    // constructor
    Location = function(locationDiv) {
    		this.div = $(locationDiv);
    	};
	
	Location.prototype.update = function(lon, lat, cooFrame, italic) {
		var coo = new Coo(lon, lat, 7);
		if (cooFrame==CooFrameEnum.J2000) {
            this.div.html('&alpha;, &delta;: ' + (italic ? '<em>' : '') + coo.format('s/') + (italic ? '</em>' : ''));
        }
        else {
            this.div.html( 'l, b: ' + (italic ? '<em>' : '') + coo.format('d/') + (italic ? '</em>' : ''));
        }
	};
	
	return Location;
})();
	
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File View.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

View = (function() {

    /** Constructor */
    function View (aladinDiv, imageCanvas, catalogCanvas, reticleCanvas, location, cooFrame, lon, lat) {
    		this.aladinDiv = aladinDiv; 
    		this.imageCanvas = imageCanvas;
    		this.catalogCanvas = catalogCanvas;
    		this.reticleCanvas = reticleCanvas;
    		this.location = location;
    		this.mustClearCatalog = true;
    		this.mustRedrawReticle = true;
    		
    		this.healpixGrid = new HealpixGrid(this.imageCanvas);
    		if (cooFrame) {
                this.cooFrame = cooFrame;
            }
            else {
                this.cooFrame = CooFrameEnum.GAL;
            }
    		
            if (! lon) {
                lon = 0;
            }
            if (!lat) {
                lat = 0;
            }
    		
    		this.projectionMethod = ProjectionEnum.SIN;
    		this.projection = new Projection(lon, lat);
    		this.projection.setProjection(this.projectionMethod);
            this.zoomLevel = 0;
            this.zoomFactor = this.computeZoomFactor();
            
    
    		this.viewCenter = {lon: lon, lat: lat}; // position of center of view
    		
    		// current image survey displayed
    		this.imageSurvey = null;
    		// current catalog displayed
    		this.catalogs = [];
    		
    		/*
    		var c = new Catalog();
    		c.addSource(new Source(201.36508, -43.01911));
    		c.addSource(new Source(0.0, 0.0));
    		this.addCatalog(c);
    		*/
    //		this.catalog.addSource(new Source(10, 5));
    //		this.catalog.addSource(new Source(0, 15));
    //		this.catalog.addSource(new Source(23.0, -20));
    		
    		// catalogue for source to highlight
    		
    
    		
    		this.tileBuffer = new TileBuffer(); // tile buffer is shared across different image surveys
    		this.fixLayoutDimensions();
            
    
    //		this.width = this.imageCanvas.width;
    //		this.height = this.imageCanvas.height;
    //		
    //		this.cx = this.width/2;
    //		this.cy = this.height/2;
    //		
    //		this.largestDim = Math.max(this.width, this.height);
    //		this.smallestDim = Math.min(this.width, this.height);
    //		this.ratio = this.largestDim/this.smallestDim;
    //
    //		
    //		this.mouseMoveIncrement = 160/this.largestDim;
    		
    		this.curNorder = 1;
    		
    		// some variables for mouse handling
    		this.dragging = false;
    		this.dragx = null;
    		this.dragy = null;
    		this.needRedraw = true;
    
            this.downloader = new Downloader(this); // the downloader object is shared across all HpxImageSurveys
            this.flagForceRedraw = false;
    
            this.fadingLatestUpdate = null;
    		
            this.dateRequestRedraw = null;
            
    		init(this);
    		
    		
    //		this.resizeTimer = null;
    //		var self = this;
    //		$(window).resize(function() {
    //		    clearTimeout(self.resizeTimer);
    //		    self.resizeTimer = setTimeout(self.fixLayoutDimensions(self), 100);
    //		});
    	};
	
	View.DRAW_SOURCES_WHILE_DRAGGING = true;
	
	// called at startup and when window is resized
	View.prototype.fixLayoutDimensions = function() {
		this.width = $(this.aladinDiv).width();
		this.height = $(this.aladinDiv).height();
		
		
		this.cx = this.width/2;
		this.cy = this.height/2;
		
		this.largestDim = Math.max(this.width, this.height);
		this.smallestDim = Math.min(this.width, this.height);
		this.ratio = this.largestDim/this.smallestDim;

		
		this.mouseMoveIncrement = 160/this.largestDim;
		

		
		// reinitialize 2D context
		this.imageCtx = this.imageCanvas.getContext("2d");
		this.catalogCtx = this.catalogCanvas.getContext("2d");
		this.reticleCtx = this.reticleCanvas.getContext("2d");
		
		this.imageCtx.canvas.width = this.width;
		this.catalogCtx.canvas.width = this.width;
        this.reticleCtx.canvas.width = this.width;

		
		this.imageCtx.canvas.height = this.height;
		this.catalogCtx.canvas.height = this.height;
        this.reticleCtx.canvas.height = this.height;
	};
    





	/**
	 * Compute the FoV in degrees of the view and update mouseMoveIncrement
	 * 
	 * @param view
	 * @returns FoV (array of 2 elements : width and height) in degrees
	 */
	computeFov = function(view) {
		var fov;
		// if zoom factor < 1, we view 180°
		if (view.zoomFactor<1) {
			fov = 180;
		}
		else {
			// TODO : fov sur les 2 dimensions !!
			// to compute FoV, we first retrieve 2 points at coordinates (0, view.cy) and (width-1, view.cy)
			var xy1 = AladinUtils.viewToXy(0, view.cy, view.width, view.height, view.largestDim, view.zoomFactor);
			var lonlat1 = view.projection.unproject(xy1.x, xy1.y);
			
			var xy2 = AladinUtils.viewToXy(view.imageCanvas.width-1, view.cy, view.width, view.height, view.largestDim, view.zoomFactor);
			var lonlat2 = view.projection.unproject(xy2.x, xy2.y);
			
			
			fov = new Coo(lonlat1.ra, lonlat1.dec).distance(new Coo(lonlat2.ra, lonlat2.dec));
		}
		
		view.mouseMoveIncrement = fov/view.imageCanvas.width;
			
		return fov;
	};
	
	/**
	 * compute the norder corresponding to the current view resolution
	 */
	computeNOrder = function(view) {
		var resolution = view.fov / view.largestDim; // in degree/pixel
		var tileSize = 512;
		var nside = HealpixIndex.calculateNSide(3600*tileSize*resolution); // 512 = taille d'une image "tuile"
		var norder = Math.log(nside)/Math.log(2);
		//norder += 1;
		norder = Math.max(norder, 1);

		// forcer le passage à norder 3?
//		if (view.fov<50 && norder==2) {
//			norder = 3;
//		}
		
        if (view.imageSurvey && norder>view.imageSurvey.maxOrder) {
            norder = view.imageSurvey.maxOrder;
        }
        // should never happen, as calculateNSide will return something <=HealpixIndex.ORDER_MAX
        if (norder>HealpixIndex.ORDER_MAX) {
        	norder = HealpixIndex.ORDER_MAX;
        }
		return norder;
	};
	
	init = function(view) {

		
        var stats = new Stats();
        stats.domElement.style.top = '50px';
        if ($('#statsDiv').length>0) {
        	$('#statsDiv')[0].appendChild( stats.domElement );
        }
        
        view.stats = stats;
		
        var hasTouchEvents = false;
        if ('ontouchstart' in window) {
        	hasTouchEvents = true;
        }

        
		// various listeners
        onDblClick = function(e) {
        	var xymouse = view.imageCanvas.relMouseCoords(e);
			var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
			try {
				var lonlat = view.projection.unproject(xy.x, xy.y);
			}
			catch(err) {
				return;
			}
			radec = [];
			// convert to J2000 if needed
			if (view.cooFrame==CooFrameEnum.GAL) {
				radec = CooConversion.GalacticToJ2000([lonlat.ra, lonlat.dec]);
			}
			else {
				radec = [lonlat.ra, lonlat.dec];
			}

			view.pointTo(radec[0], radec[1]);
        };
        if (! hasTouchEvents) {
            $(view.reticleCanvas).dblclick(onDblClick);
        }
        
        
		$(view.reticleCanvas).bind("mousedown touchstart", function(e) {
		    var xymouse = view.imageCanvas.relMouseCoords(e);
			if (e.originalEvent && e.originalEvent.targetTouches) {
				view.dragx = e.originalEvent.targetTouches[0].clientX;
				view.dragy = e.originalEvent.targetTouches[0].clientY;
			}
			else {
			    /*
				view.dragx = e.clientX;
				view.dragy = e.clientY;
				*/
			    view.dragx = xymouse.x;
                view.dragy = xymouse.y;
			}
			view.dragging = true;
			view.reticleCanvas.style.cursor = 'move';
            return false; // to disable text selection
		});
		$(view.reticleCanvas).bind("mouseup mouseout touchend", function(e) {
			view.dragx = view.dragy = null;
			view.dragging = false;
            view.reticleCanvas.style.cursor = 'default';
            view.mustClearCatalog = true;
			view.requestRedraw();
		});
		$(view.reticleCanvas).bind("mousemove touchmove", function(e) {
            e.preventDefault();

            var xymouse = view.imageCanvas.relMouseCoords(e);
			if (!view.dragging || hasTouchEvents) {
				    updateLocation(view, xymouse.x, xymouse.y, true);
				    /*
					var xy = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
					var lonlat;
					try {
						lonlat = view.projection.unproject(xy.x, xy.y);
					}
					catch(err) {
					}
					if (lonlat) {
						view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, true);
					}
					*/
				if (!hasTouchEvents) return;
			}

			var xoffset, yoffset;
			var pos1, pos2;
            
			if (e.originalEvent && e.originalEvent.targetTouches) {
			    // ???
				xoffset = e.originalEvent.targetTouches[0].clientX-view.dragx;
				yoffset = e.originalEvent.targetTouches[0].clientY-view.dragy;
                var xy1 = AladinUtils.viewToXy(e.originalEvent.targetTouches[0].clientX, e.originalEvent.targetTouches[0].clientY, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

				pos1 = view.projection.unproject(xy1.x, xy1.y);
				pos2 = view.projection.unproject(xy2.x, xy2.y);
			}
			else {
			    /*
				xoffset = e.clientX-view.dragx;
				yoffset = e.clientY-view.dragy;
				*/
			    xoffset = xymouse.x-view.dragx;
                yoffset = xymouse.y-view.dragy;
                
                var xy1 = AladinUtils.viewToXy(xymouse.x, xymouse.y, view.width, view.height, view.largestDim, view.zoomFactor);
                var xy2 = AladinUtils.viewToXy(view.dragx, view.dragy, view.width, view.height, view.largestDim, view.zoomFactor);

                
                pos1 = view.projection.unproject(xy1.x, xy1.y);
                pos2 = view.projection.unproject(xy2.x, xy2.y);
                
			}
			var distSquared = xoffset*xoffset+yoffset*yoffset;
			
			// TODO : faut il faire ce test ??
			if (distSquared<3) {
				return;
			}
			if (e.originalEvent && e.originalEvent.targetTouches) {
				view.dragx = e.originalEvent.targetTouches[0].clientX;
				view.dragy = e.originalEvent.targetTouches[0].clientY;
			}
			else {
			    view.dragx = xymouse.x;
			    view.dragy = xymouse.y;
				/*
			    view.dragx = e.clientX;
				view.dragy = e.clientY;
				*/
			}

			//view.viewCenter.lon += xoffset*view.mouseMoveIncrement/Math.cos(view.viewCenter.lat*Math.PI/180.0);
			/*
			view.viewCenter.lon += xoffset*view.mouseMoveIncrement;
			view.viewCenter.lat += yoffset*view.mouseMoveIncrement;
			*/
			view.viewCenter.lon += pos2.ra -  pos1.ra;
            view.viewCenter.lat += pos2.dec - pos1.dec;
            

			
			// can not go beyond poles
			if (view.viewCenter.lat>90) {
				view.viewCenter.lat = 90;
			}
			else if (view.viewCenter.lat < -90) {
				view.viewCenter.lat = -90;
			}
			
			// limit lon to [0, 360]
			if (view.viewCenter.lon < 0) {
				view.viewCenter.lon = 360 + view.viewCenter.lon;
			}
			else if (view.viewCenter.lon > 360) {
				view.viewCenter.lon = view.viewCenter.lon % 360;
			}
			view.requestRedraw();
		}); //// endof mousemove ////
		
        // disable text selection on IE
        $(view.aladinDiv).onselectstart = function () { return false; }

		$(view.reticleCanvas).bind('mousewheel', function(event, delta) {
			event.preventDefault();
			event.stopPropagation();
			var level = view.zoomLevel;
			if (delta>0) {
				level += 1;
			}
			else {
				level -= 1;
			}
			view.setZoomLevel(level);
			
			return false;
		});
	
        view.displayHpxGrid = false;
        view.displaySurvey = true;
        view.displayCatalog = false;
        view.displayReticle = true;
        
		// initial draw
		view.fov = computeFov(view);
		// TODO : voir comment séparer cette dépendance de la vue
		window.view = view;
		redraw(view);
	};

	function updateLocation(view, x, y, italic) {
	    if (!view.projection) {
	        return;
	    }
	    var xy = AladinUtils.viewToXy(x, y, view.width, view.height, view.largestDim, view.zoomFactor);
        var lonlat;
        try {
            lonlat = view.projection.unproject(xy.x, xy.y);
        }
        catch(err) {
        }
        if (lonlat) {
            view.location.update(lonlat.ra, lonlat.dec, view.cooFrame, italic);
        }
	}
	
	View.prototype.requestRedrawAtDate = function(date) {
	    this.dateRequestDraw = date;
	};
	
	/**
	 * redraw the whole view
	 */
	redraw = function() {
		var saveNeedRedraw = view.needRedraw;
		requestAnimFrame(redraw);

		var now = new Date().getTime();
		if (view.dateRequestDraw && now>view.dateRequestDraw) {
		    view.dateRequestDraw = null;
		} 
		else if (! view.needRedraw) {
            if ( ! view.flagForceRedraw) {
			    return;
            }
            else {
                view.flagForceRedraw = false;
            }
		}
		view.stats.update();

		var imageCtx = view.imageCtx;
		//////// 1. Draw images ////////
		
		//// clear canvas ////
		imageCtx.clearRect(0, 0, view.imageCanvas.width, view.imageCanvas.height);
		////////////////////////
		
		// black background
        if (view.fov>80 && view.projectionMethod==ProjectionEnum.SIN) {
        	imageCtx.fillStyle = "rgb(0,0,0)";
        	imageCtx.beginPath();
        	imageCtx.arc(view.cx, view.cy, view.cx*view.zoomFactor, 0, 2*Math.PI, true);
        	imageCtx.fill();
        }

		if (!view.projection) {
			view.projection = new Projection(view.viewCenter.lon, view.viewCenter.lat);
		}
		else {
			view.projection.setCenter(view.viewCenter.lon, view.viewCenter.lat);
		}
		view.projection.setProjection(view.projectionMethod);
	

		// ************* Tracé au niveau allsky (faible résolution) *****************
		var cornersXYViewMapAllsky = view.getVisibleCells(3);
		var cornersXYViewMapHighres = null;
		if (view.curNorder>=3) {
			if (view.curNorder==3) {
				cornersXYViewMapHighres = cornersXYViewMapAllsky;
			}
			else {
				cornersXYViewMapHighres = view.getVisibleCells(view.curNorder);
			}
		}

		// redraw image survey
		if (view.imageSurvey && view.displaySurvey) {
			view.imageSurvey.redrawAllsky(imageCtx, cornersXYViewMapAllsky, view.fov, view.curNorder);
            if (view.curNorder>=3) {
                view.imageSurvey.redrawHighres(imageCtx, cornersXYViewMapHighres, view.curNorder);
            }
		}
		
		
		
		
		
		// redraw grid
        if( view.displayHpxGrid) {
        	if (cornersXYViewMapHighres && view.curNorder>3) {
        		view.healpixGrid.redraw(imageCtx, cornersXYViewMapHighres, view.fov, view.curNorder);
        	}
            else {
        	    view.healpixGrid.redraw(imageCtx, cornersXYViewMapAllsky, view.fov, 3);
            }
        }
 		
        // TODO : dessiner cette valeur ailleurs que sur le imageCtx ?
 		// draw FoV value
        imageCtx.beginPath();
        imageCtx.font = "16pt";
        imageCtx.fillStyle = "rgb(230,120,250)";
        imageCtx.textWidth = 2.5;
        var fovStr;
        if (view.fov>1) {
            fovStr = Math.round(view.fov*100)/100 + "°";
        }
        else if (view.fov*60>1) {
            fovStr = Math.round(view.fov*60*100)/100 + "'";
        }
        else {
            fovStr = Math.round(view.fov*3600*100)/100 + '"';
        }
        imageCtx.fillText("FoV: " + fovStr, 10, view.height-10);
        imageCtx.stroke();

        
		////// 2. Draw catalogues////////
		var catalogCtx = view.catalogCtx;

		var catalogCanvasCleared = false;
        if (view.mustClearCatalog) {
            catalogCtx.clearRect(0, 0, view.width, view.height);
            catalogCanvasCleared = true;
            view.mustClearCatalog = false;
        }
		if (view.catalogs && view.catalogs.length>0 && view.displayCatalog && (! view.dragging  || View.DRAW_SOURCES_WHILE_DRAGGING)) {
		      // TODO : ne pas effacer systématiquement
	        //// clear canvas ////
		    if (! catalogCanvasCleared) {
		        catalogCtx.clearRect(0, 0, view.width, view.height);
		    }
		    for (var i=0; i<view.catalogs.length; i++) {
		        view.catalogs[i].draw(catalogCtx, view.projection, view.cooFrame, view.width, view.height, view.largestDim, view.zoomFactor, view.cooFrame);
		    }
        }
		
		////// 3. Draw reticle ///////
		// TODO : canvas supplémentaire avec réticule uniquement ? --> mustRedrawReticle
		var reticleCtx = view.reticleCtx;
		if (view.mustRedrawReticle) {
            reticleCtx.clearRect(0, 0, view.width, view.height);
		}
		if (view.displayReticle) {
    		reticleCtx.lineWidth = 1;
    		reticleCtx.strokeStyle = "rgb(178, 0, 178)";
    		reticleCtx.beginPath();
    		reticleCtx.moveTo(view.width/2, view.height/2+10);
    		reticleCtx.lineTo(view.width/2, view.height/2+2);
    		reticleCtx.moveTo(view.width/2, view.height/2-10);
    		reticleCtx.lineTo(view.width/2, view.height/2-2);
    		
    		reticleCtx.moveTo(view.width/2+10, view.height/2);
    		reticleCtx.lineTo(view.width/2+2,  view.height/2);
    		reticleCtx.moveTo(view.width/2-10, view.height/2);
    		reticleCtx.lineTo(view.width/2-2,  view.height/2);
            
    		reticleCtx.stroke();
    		view.mustRedrawReticle = false;
		}
        
        
 		// TODO : est ce la bonne façon de faire ?
 		if (saveNeedRedraw==view.needRedraw) {
 			view.needRedraw = false;
 		}
	};

    View.prototype.forceRedraw = function() {
        this.flagForceRedraw = true;
    };
	
	View.prototype.getVisibleCells = function(norder) {
		var cells = []; // will be returned
		var cornersXY = [];
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		var npix = HealpixIndex.nside2Npix(nside);
		var ipixCenter = null;
		
		// build list of pixels
		var pixList;
		if (this.fov>80) {
			pixList = [];
			for (var ipix=0; ipix<npix; ipix++) {
				pixList.push(ipix);
			}
		}
		else {
			var hpxIdx = new HealpixIndex(nside);
			hpxIdx.init();
			var spatialVector = new SpatialVector();
			// si frame != frame survey image, il faut faire la conversion dans le système du survey
			var xy = AladinUtils.viewToXy(this.cx, this.cy, this.width, this.height, this.largestDim, this.zoomFactor);
			var radec = this.projection.unproject(xy.x, xy.y);
			var lonlat = [];
			if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
				if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
                    lonlat = CooConversion.GalacticToJ2000([radec.ra, radec.dec]); 
                }
                else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
                    lonlat = CooConversion.J2000ToGalactic([radec.ra, radec.dec]);
                }
			}
			else {
				lonlat = [radec.ra, radec.dec];
			}
			spatialVector.set(lonlat[0], lonlat[1]);
			var radius = this.fov*0.5*this.ratio;
			// we need to extend the radius
			if (this.fov>60) {
				radius *= 1.6;
			}
			else if (this.fov>12) {
				radius *=1.45;
			}
            else {
                radius *= 1.1;
            }
			
			
				
			pixList = hpxIdx.queryDisc(spatialVector, radius*Math.PI/180.0, true, true);
			// add central pixel at index 0
			var polar = Utils.radecToPolar(lonlat[0], lonlat[1]);
			ipixCenter = hpxIdx.ang2pix_nest(polar.theta, polar.phi);
			pixList.unshift(ipixCenter);
		}
		
		
		var ipix;
		var lon, lat;
		for (var ipixIdx=0, len=pixList.length; ipixIdx<len; ipixIdx++) {
			ipix = pixList[ipixIdx];
			if (ipix==ipixCenter && ipixIdx>0) { 
				continue;
			}
			var cornersXYView = [];
			corners = HealpixCache.corners_nest(ipix, nside);

			for (var k=0; k<4; k++) {
				spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	            // need for frame transformation ?
				if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
	                if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
	                    var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	                else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
	                    var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                    lon = radec[0];
	                    lat = radec[1];
	                }
	            }
	            else {
	                lon = spVec.ra();
	                lat = spVec.dec();
	            }
	            
				cornersXY[k] = this.projection.project(lon, lat);
			}


			if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
	            continue;
	        }


			for (var k=0; k<4; k++) {
				cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
			}


			// check if pixel is visible
//			if (this.fov<160) { // don't bother checking if fov is large enough
//				if ( ! AladinUtils.isHpxPixVisible(cornersXYView, this.width, this.height) ) {
//					continue;
//				}
//			}
			// check if we have a pixel at the edge of the view in AITOFF --> TO BE MODIFIED
			if (this.projection.PROJECTION==ProjectionEnum.AITOFF) {
				var xdiff = cornersXYView[0].vx-cornersXYView[2].vx;
				var ydiff = cornersXYView[0].vy-cornersXYView[2].vy;
				var distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
				xdiff = cornersXYView[1].vx-cornersXYView[3].vx;
				ydiff = cornersXYView[1].vy-cornersXYView[3].vy;
				distDiag = Math.sqrt(xdiff*xdiff + ydiff*ydiff);
				if (distDiag>this.largestDim/5) {
					continue;
				}
			}
			
			cornersXYView.ipix = ipix;
			cells.push(cornersXYView);
		}
		
		return cells;
	};
	
	
	
	// get position in view for a given HEALPix cell
	View.prototype.getPositionsInView = function(ipix, norder) {
		var cornersXY = [];
		var lon, lat;
		var spVec = new SpatialVector();
		var nside = Math.pow(2, norder); // TODO : à changer
		
		
		var cornersXYView = [];  // will be returned
		var corners = HealpixCache.corners_nest(ipix, nside);

		for (var k=0; k<4; k++) {
			spVec.setXYZ(corners[k].x, corners[k].y, corners[k].z);
				
	        // need for frame transformation ?
			if (this.imageSurvey && this.imageSurvey.cooFrame != this.cooFrame) {
	            if (this.imageSurvey.cooFrame==CooFrameEnum.J2000) {
	                var radec = CooConversion.J2000ToGalactic([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	            else if (this.imageSurvey.cooFrame==CooFrameEnum.GAL) {
	                var radec = CooConversion.GalacticToJ2000([spVec.ra(), spVec.dec()]); 
	                lon = radec[0];
	                lat = radec[1];
	            }
	        }
	        else {
	            lon = spVec.ra();
	            lat = spVec.dec();
	        }
	            
			cornersXY[k] = this.projection.project(lon, lat);
		}
		
		if (cornersXY[0] == null ||  cornersXY[1] == null  ||  cornersXY[2] == null ||  cornersXY[3] == null ) {
            return null;
        }


		for (var k=0; k<4; k++) {
			cornersXYView[k] = AladinUtils.xyToView(cornersXY[k].X, cornersXY[k].Y, this.width, this.height, this.largestDim, this.zoomFactor);
		}

		return cornersXYView;
	};
	
	
	View.prototype.computeZoomFactor = function() {
    	if (this.zoomLevel>0) {
    	    return AladinUtils.getZoomFactorForAngle(180/Math.pow(1.15, this.zoomLevel), this.projectionMethod);
    		//return 1/Math.pow(0.9, this.zoomLevel);
		}
		else {
		    return 1 + 0.1*this.zoomLevel;
		}
		/*
		if (this.zoomLevel==0) {
			return 1;
		}
		if (this.zoomLevel>0) {
			return Math.sqrt(0.3+this.zoomLevel);
		}
		if (this.zoomLevel<0) {
			return 1/Math.log(10-this.zoomLevel);
		}
		*/
	};
	
	View.prototype.setZoom = function(fovDegrees) {
	    if (fovDegrees<0 || fovDegrees>180) {
	        return;
	    }
	    var zoomLevel = Math.log(180/fovDegrees)/Math.log(1.15);
	    this.setZoomLevel(zoomLevel);
	};

    View.prototype.setZoomLevel = function(level) {
        this.zoomLevel = Math.max(-3, level);
        this.zoomFactor = this.computeZoomFactor();
        
        this.fov = computeFov(this);
        this.curNorder = computeNOrder(this);

        this.forceRedraw();
		this.requestRedraw();
    };
	
	View.prototype.setImageSurvey = function(imageSurvey) {
		this.imageSurvey = null;
		this.imageSurvey = imageSurvey;
        this.curNorder = computeNOrder(this);
        this.imageSurvey.init(this);
        this.requestRedraw();
	};
	
	View.prototype.requestRedraw = function() {
		this.needRedraw = true;
		//redraw(this);
	};
	
	View.prototype.changeProjection = function(projectionMethod) {
		this.projectionMethod = projectionMethod;
		this.requestRedraw();
	};

	View.prototype.changeFrame = function(cooFrame) {
		this.cooFrame = cooFrame;
        // recompute viewCenter
        if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = lb[0];
            this.viewCenter.lat = lb[1]; 
        }
        else if (this.cooFrame==CooFrameEnum.J2000) {
            var radec = CooConversion.GalacticToJ2000([this.viewCenter.lon, this.viewCenter.lat]);
            this.viewCenter.lon = radec[0];
            this.viewCenter.lat = radec[1]; 
        }
		this.requestRedraw();
	};

    View.prototype.showHealpixGrid = function(show) {
        this.displayHpxGrid = show;
        this.requestRedraw();
    };
    
    View.prototype.showSurvey = function(show) {
        this.displaySurvey = show;

        this.requestRedraw();
    };
    
    View.prototype.showCatalog = function(show) {
        this.displayCatalog = show;

        if (!this.displayCatalog) {
            this.mustClearCatalog = true;
        }
        this.requestRedraw();
    };
    
    View.prototype.showReticle = function(show) {
        this.displayReticle = show;

        this.mustRedrawReticle = true;
        this.requestRedraw();
    };

    View.prototype.pointTo = function(ra, dec) {
        ra = parseFloat(ra);
        dec = parseFloat(dec);
        if (isNaN(ra) || isNaN(dec)) {
            return;
        }
        if (this.cooFrame==CooFrameEnum.J2000) {
		    this.viewCenter.lon = ra;
		    this.viewCenter.lat = dec;
        }
        else if (this.cooFrame==CooFrameEnum.GAL) {
            var lb = CooConversion.J2000ToGalactic([ra, dec]);
		    this.viewCenter.lon = lb[0];
		    this.viewCenter.lat = lb[1];
        }

        this.forceRedraw();
        this.requestRedraw();
    };
    
    View.prototype.addCatalog = function(catalog) {
        this.catalogs.push(catalog);
        catalog.setView(this);
    }
    
    return View;
})();
/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Aladin.js (main class)
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

/** @license CDS - Centre de Données astronomiques de Strasbourg , 2013*/
Aladin = (function() {
    
    // Constructor
	var Aladin = function(aladinDiv, requestedOptions) {
	    HealpixCache.init();
        
	    
	    // if not options was set, try to retrieve them from the query string
	    if (requestedOptions===undefined) {
	        requestedOptions = this.getOptionsFromQueryString();
	    }
	    requestedOptions = requestedOptions || {};
	    
	    //console.log(requestedOptions);
	    
	    // merge with default options
	    var options = {};
	    for (var key in Aladin.DEFAULT_OPTIONS) {
	        if (requestedOptions[key] !== undefined) {
	            options[key] = requestedOptions[key]; 
	        }
	        else {
	            options[key] = Aladin.DEFAULT_OPTIONS[key];
	        }
	    }
	    //console.log(options);
	    
//		canvas.imageSmoothingEnabled = false;
//		canvas.webkitImageSmoothingEnabled = false;
//		canvas.mozImageSmoothingEnabled = false;

		this.aladinDiv = aladinDiv;

		// parent div
		$(aladinDiv).css("position", "relative");
		// div where we write the position
		var locationDiv = $('<div id="locationDiv" style="z-index: 1000;position:absolute; padding: 2px 4px 2px 4px;  font-size: 13px; background-color: rgba(255, 255, 255, 0.5)"></div>').appendTo(aladinDiv);

		// canvas to draw the images
		var imageCanvas = $("<canvas style='position: absolute; left: 0; top: 0; z-index: 1;'></canvas>").appendTo(aladinDiv)[0];
		// canvas to draw the catalogs
		var catalogCanvas = $("<canvas style='position: absolute; left: 0; top: 0; z-index: 2;'></canvas>").appendTo(aladinDiv)[0];
		// canvas to draw the reticle
	    var reticleCanvas = $("<canvas style='position: absolute; left: 0; top: 0; z-index: 3;'></canvas>").appendTo(aladinDiv)[0];

		// Aladin logo
		$("<img src='" + Aladin.LOGO + "' width='50' height='29' style='position: absolute; bottom: 5px; right: 10px; z-index: 3;' />").appendTo(aladinDiv);
		
		// control panel
		// TODO : valeur des checkbox en fonction des options
		if (options.showControl) {
			$('<button id="showControlBox" style="z-index: 20; position: absolute; right: 0px; top: 0px;">Controls</button>').appendTo(aladinDiv);
			$('<div id="controlBox" style="display: none;background: white;position: absolute; right: 0px; top: 0px; border: 2px solid; padding: 4px 10px 10px 10px; z-index: 30; ">' +
            '<button id="closeControlBox" style="float: right;">Close</button>' +
            '<div style="clear: both;">' +
	        '<form id="targetForm" style="clear; both;">Target: <input type="text" id="target" /></form>' +
	        'Frame: <select id="frameChoice"><option>J2000</option><option selected="selected">GAL</option></select><br/>' +
	        'Projection: <select id="projectionChoice"><option>SINUS</option><option>AITOFF</option></select><br/>' +
	        '<input type="checkbox" id="displayHpxGrid"/><label for="displayHpxGrid">Show HEALPix grid</label><br/>' +
	        '<input type="checkbox" id="displaySurvey" checked="checked" /><label for="displaySurvey">Show survey</label><br/>' +
	        '<input type="checkbox" id="displayCatalog" /><label for="displayCatalog">Show catalogs</label><br/>' +
            '<input type="checkbox" id="displayReticle" checked="checked" /><label for="displayReticle">Show reticle</label><br/>' +
	        '<select id="surveySelection"></select><br/>' +
	        'Zoom:<br/>' +
	        '<button id="zoomPlus" style="width: 30%"><b> + </b></button> <button id="zoomMinus"  style="width: 30%"><b> - </b></button>' +
	        '</div></div>').appendTo(aladinDiv);
			
			$('#showControlBox').click(function() {$('#controlBox').show();});
			$('#closeControlBox').click(function() {$('#controlBox').hide();});
		}
		
		var surveys = HpxImageSurvey.getAvailableSurveys();
        for (var i=0; i<surveys.length; i++) {
        	$('#surveySelection').append($("<option />").val(surveys[i].name).text(surveys[i].name));
        };
        
        

		
		
		
		
		var location = new Location(locationDiv);
        
		// set different options
		var cooFrame = options.cooFrame;
		this.view = new View(this.aladinDiv, imageCanvas, catalogCanvas, reticleCanvas, location, cooFrame);
        this.gotoObject(options.target);

        if (options.log) {
            Logger.log("startup");
        }

		if (options.zoom) {
            this.setZoom(options.zoom);
        }
		
		this.showReticle(options.showReticle);
		
		var surveyInfo = HpxImageSurvey.getSurveyInfoFromName(options.survey);
		this.setImageSurvey(new HpxImageSurvey(surveyInfo.name, surveyInfo.url, surveyInfo.frame, surveyInfo.maxOrder));
		this.view.showCatalog(options.showCatalog);
		
	    
    	var aladin = this;
    	$('#frameChoice').change(function() {
    		aladin.setFrame($(this).val());
    	});
    	$('#projectionChoice').change(function() {
    		aladin.setProjection($(this).val());
    	});
        $('#displayHpxGrid').change(function() {
            aladin.showHealpixGrid($(this).is(':checked'));
        });
        $('#displaySurvey').change(function() {
            aladin.showSurvey($(this).is(':checked'));
        });
        $('#displayCatalog').change(function() {
            aladin.showCatalog($(this).is(':checked'));
        });
        $('#displayReticle').change(function() {
            aladin.showReticle($(this).is(':checked'));
        });

        $('#targetForm').submit(function() {
            aladin.gotoObject($('#target').val());
            return false;
        });
        
        $('#zoomPlus').click(function() {
        	aladin.increaseZoom();
        });
        
        $('#zoomMinus').click(function() {
            aladin.decreaseZoom();
        });
        
        $('#surveySelection').change(function() {
            var survey = surveys[$(this)[0].selectedIndex];
        	aladin.setImageSurvey(new HpxImageSurvey(survey.name, survey.url, survey.frame, survey.maxOrder));
        });
        
        

		
	};
	
	Aladin.LOG_URL = "";
	
    Aladin.VERSION = "0.1";
    
    Aladin.LOGO = 'data:image/gif;base64,R0lGODlhRgApAOf/AAACAA4HBQgMDBcLCxITEBkYFyEaHikaHSMkIjEkKrQAMYIOOmEYQlweSWoaSLcFOGceSFImUjM0MksrXmwkTUgyO1EwOLoRQEE3N0c1QlotWngmTz02Zkc5P7QZRJsiSWEzYHIvVrceSzpFUIIvV3E2SLEjSURGQ7IlUHk2W3E6Y2c9Y01IUl5EX11GWGlBbE5MT2lGVm1DaY46YW9EWk5QTVtMU3xBYntBZ5Y6X2tHa7Y0VnRJYaU8VoRFYoBGZ25LccA1W45DZq07YHpKZ5dDWaFAW3hLb7Q8XWhSZptFZpZHZVJYiJNJX2xSeb49Ym5Ub4pMb4FQbIRPc35Sa4dPb3hTeIFSdadIanZYb15hYX9Xd7pHaoVWeJ9Qa39Zf5BVdqdPb6VRZhxzrJJWcstIbG9ihn5fgodedndiiGxqasZPba1WfI5fgcNRcmppkENzo51eb7dWdYljg15ul3poh6tcdlB1lZ1ihm5ymF53nqVlgkJ/sc5be2R3o5lqiJFtilh/q4Z2hMJmgoF4mnx+e5J2k6F0f6F0j2qCq5F9g3aCp6x4hdBviZp/lmCOrZl/nY6CoXaJraR+mISHqWGQub15jqODjNR1i3CRuYSQmI+PjZmLpKKJn3+St7GInXiWu5uRl8aEnWedxJeSsKqOo5WWlKOSmJ6UmtOGm4SbuMSLoqeVqqWXpN2MormWqp+gnqWfqq2ds9yRpX6qy7ycrrOhttGarbClqbSkq5arxqKqsqOpvp6qxKiqp82gsIWz0+GdrsSlt7mpveSmtc+su8WvvZS81qS40Niru8Gyv5y71bO4vuGru7i4tby6zsi5x6bD2uqyv6zD1cq9vsDCvum6xdDBz67K4bbJ1tXBysjE1u28x7TL3cjKx8/J08fL2+zCy9fI1uLGy7vS5N/L1L/X6fHN1Mba5t7U2+fS2t3X49ja5szg7PPX3OXb4dXh7+Lf4Pba3/Tf4uzi6N7n7/Ti6ujm6unp5vrl5+3r7/Tp8Pfq6+nu8ebv9////yH5BAEKAP8ALAAAAABGACkAAAj+AP8JHEiwoMGDCBMqXMiwocOHECNKnEixYsNvAtv945fvn7x5/8D9E6fsmi1WtmzJsqVs3b11+v6dE0hOIDaLENEJBPlPnS1DZ7506SIkhxAZESKcMWQIkiFeywjaw7lw3kxi+/4t+0nIUJcbFFKEEIYoShQvM1Lc6HIlaVI9Z2Z6pGpQm0BiAqX9+2YowpU/KvJwMoZMoLJ//qIJ1JaCAQMpGpLKiAQNmECMdAXyG9jxnTBHV6oIqfJuKkF3/9gJ9Ofv375PECBcARFhBYVhAt/h1P2vWV6Bv0rNiaAijBA7wgSGEyhynrzU/f5h07jcDYkpOIiDuZZ7orXL//T+6Xa3fJatyEA42UG0L9m/d+n+EZN2Lhw3d4he6Tem7zA5XRBIkUIEN3Tyz3IVtfaPTv+UM5IyVkQAwhzbqCbXNfaEE4wHCijgARZCUOCYY6/8w014DExRxRGEIFPPQww+Rw89/2hUo0CTrODWEYiUU09rxDQixxAKPGDCBX2UgYRjDozIQAoyvQfBCiuYMQZrEDn4zzECdYNYTfT84dYKk9g1yz/z9NHhGrOE05E0Hc2jDhcPIEECAxAMo5g7e1QBQh2V3LQgQjwNpGBr9USnjkD3QJIUB7bI9WIjDygwBGrGCHTigf+4U6kCGzAQxl3GXKEDJKB4mc+mB6H2nkD+L/6jxQlaCEQPNRIg0MIETNwz1z4dEICBNPykYwoByBJQQCH4zKQIAAAs4QK00AZAAAuU8NGQXv7oVE5MXBYC7T23/OMMtC5Q8o85ra1xQRHQxqNNOHFQSy0C8cwiBrRooGIvtbvUBBGCJ9KTC7Sw/MMLFdAK8k9y/2CCggjjQHuKmjvgckkRRVgArQRuDEHDKeYwA20SUIwA7SESuUoQAgDUQIsTP0DLjEDuBDFECPHBAEAJHS63hgIXnFMNtFIQgUs1/3gDrRmZaAKtKfZkNZEaBEigBQACmMHADdDGck8xInRYAQCFnALAAJXaAUAAmAgUTAAAhGIKtMm4DcD+I//AMvU+Vke0DAEAqOE0ADykoAy01ZDdoRt0e/MPtEgoYMTbBw49AADV3A2ADwwD4Mw//gKgSUXnArCLKAYAYINA0F6CQocKMALtP/nUAIAi/HwDrRuW6t3KJtDqcQe0u9TjNwAJU7R1AJ2QcXYBsAPQxA7YP6EGAAj8Yw3x3R8MgBdkxED3AW40AW02xAPAjD/LNw8Rl//Q/S8ArOjzbwJr3E85vP8KQBjAIAhoTUNqovuH3uTnkBfVYxf+A4AvJmevBPQggjFAw78yQIRFjCEUt/Pc6HyBrAk+xB8vIgczBJEFHhDBB/HABz4EYgns7QAJQzABEpDghRQIYQ/+rxCGr/Chj284ogtngMMoMmMQ+vnBCwy4ABfI8SJa+MMStPMQFpQgBDCswh7a0M0x0EGPFLRhDpB4Rrf+8ZLUvIg3ErFRTNARjTRgYQMKCIJAcPOPPXxgAQswQRh+8IdPCEQx/4AGNCZBgRC8QRX3wEZrEIkXnMQKVpJgkgLKUBB6vCI/NOrOQIoBCAo0gA58uImCCtIRnCASGuBIA5Me8IBnCMQVexSIOLrEj1s0ohFkoAAI/DCKdgiKQauBxz+kUaiKyEMvpZhCAxhAgQ3kYBD/6MU/4PGidMBHGY7AwxOGsAEIvCAQS1wjExfSjmvgYJpXgAADgsAFUYgiFauUAESEIhMBMMiBDXjwBP3MYat1KqQmkWiAqWSwBQaEAAxR6EIIlCALXvCCFJXIhC6ioUwG2chXBl0IKMxABzOYIQ1OcMIb8pCIStCCHIiMjz8EFtKI+AobtAAGLXYKDI20A1EFRUxNI6IYf9zES//I1D+CQRMTgYR+Q40IQf+BVJrK5KlRzapWt8rVrnr1q2ANK0UCAgA7';
	
    Aladin.DEFAULT_OPTIONS = {
        target:    "0 +0",
        cooFrame: "Galactic",
        survey: "DSS Red",
        zoom:         60,
        showReticle: true,
        showControl: true,
        showCatalog: true,
        log:         true
    };
    
    Aladin.prototype.getOptionsFromQueryString = function() {
        var options = {};
        var requestedTarget = $.urlParam('target');
        if (requestedTarget) {
            options.target = requestedTarget;
        }
        var requestedFrame = $.urlParam('frame');
        if (requestedFrame && CooFrameEnum[requestedFrame] ) {
            options.frame = requestedFrame;
        }
        var requestedSurveyName = $.urlParam('survey');
        if (requestedSurveyName && HpxImageSurvey.getSurveyInfoFromName(requestedSurveyName)) {
            options.survey = requestedSurveyName;
        }
        var requestedZoom = $.urlParam('zoom');
        if (requestedZoom && requestedZoom>0 && requestedZoom<180) {
            options.zoom = requestedZoom;
        }
        
        var requestedShowreticle = $.urlParam('showReticle');
        if (requestedShowreticle) {
            options.showReticle = requestedShowreticle.toLowerCase()=='true';
        }
        
        return options;
    };
	
	Aladin.prototype.setZoom = function(fovDegrees) {
		this.view.setZoom(fovDegrees);
	};
	
    Aladin.prototype.setFrame = function(frameName) {
        if (! frameName) {
            return;
        }
        frameName = frameName.toLowerCase();
        if (frameName.indexOf('j2000')==0) {
            this.view.changeFrame(CooFrameEnum.J2000);
        }
        else if (frameName.indexOf('gal')==0) {
            this.view.changeFrame(CooFrameEnum.GAL);
        }
    }

	Aladin.prototype.setProjection = function(projectionName) {
		if (! projectionName) {
			return;
		}
		projectionName = projectionName.toLowerCase();
		switch(projectionName) {
			case "aitoff":
				this.view.changeProjection(ProjectionEnum.AITOFF);
				break;
			case "sinus":
			default:
				this.view.changeProjection(ProjectionEnum.SIN);
		}
	}
    
    // point view to a given object (resolved by Sesame) or position
    Aladin.prototype.gotoObject = function(targetName) {
    	var isObjectName = /[a-zA-Z]/.test(targetName);
    	
    	// try to parse as a position
    	if ( ! isObjectName) {
    		var coo = new Coo();

			coo.parse(targetName);
			var lonlat = [coo.lon, coo.lat];
			if (this.view.cooFrame == CooFrameEnum.GAL) {
				lonlat = CooConversion.GalacticToJ2000(lonlat);
			}
    		this.view.pointTo(lonlat[0], lonlat[1]);
    	}
    	// ask resolution by Sesame
    	else {
	        var self = this;
	        Sesame.resolve(targetName,
	                       function(data) {
	        				   if (data.Target && data.Target.Resolver && data.Target.Resolver.jpos) {
	        					   var ra = data.Target.Resolver.jradeg;
	        					   var dec = data.Target.Resolver.jdedeg;
	        					   self.view.pointTo(ra, dec);
	        				   }
	        				   else {
	                                if (console) console.log(data);
	        				   }
	        				   /*
	                           if (data.sesame.error) {
	                                if (console) console.log(data.sesame.error);
	                           }
	                           else {
	                               var radec = data.sesame.decimalPosition.split(" ");
	                               self.view.pointTo(parseFloat(radec[0]), parseFloat(radec[1]));
	                           }
	                           */
	                       },
	                       function(data) {
	                            if (console) {
	                                console.log("Could not resolve object name " + targetName);
	                                console.log(data);
	                            }
	                       });
    	}
    };
    
    Aladin.prototype.gotoPosition = function(ra, dec) {
    	this.view.pointTo(ra, dec);
    };

    Aladin.prototype.showHealpixGrid = function(show) {
        this.view.showHealpixGrid(show);
    };
    
    Aladin.prototype.showSurvey = function(show) {
        this.view.showSurvey(show);
    };
    Aladin.prototype.showCatalog = function(show) {
        this.view.showCatalog(show);
    };
    Aladin.prototype.showReticle = function(show) {
        this.view.showReticle(show);
    };
    Aladin.prototype.addCatalog = function(catalog) {
        this.view.addCatalog(catalog);
    };
    
    
    Aladin.prototype.setImageSurvey = function(imageSurvey) {
    	this.view.setImageSurvey(imageSurvey);
    };
    
    Aladin.prototype.increaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel+step);
    };
    
    Aladin.prototype.decreaseZoom = function(step) {
        if (!step) {
            step = 5;
        }
    	this.view.setZoomLevel(this.view.zoomLevel-step);
    };
    
	
	return Aladin;
})();

$.aladin = function(divSelector, options) {
    return new Aladin($(divSelector)[0], options);
};