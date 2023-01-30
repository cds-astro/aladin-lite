	/**
 	* HEALPix Javascript code derived from the jhealpix Java library
 	* 
 	* Class Constants
 	* 
 	* @author: Thomas Boch [CDS]
 	*/

	 export let Constants = {};

	 /** The Constant PI. */
	 Constants.PI = Math.PI;//3.141592653589793238462643383279502884197;
 
	 /** The Constant C_PR. */
	 Constants.C_PR = Math.PI / 180;
 
	 /** The Constant VLEV. */
	 Constants.VLEV = 2;
 
	 /** The Constant EPS. */
	 Constants.EPS = 0.0000001;
 
	 /** The Constant C. */
	 Constants.c = 0.105;
 
	 /** The Constant LN10. */
	 Constants.LN10 = Math.log(10);
 
	 /** The Constant PIOVER2. */
	 Constants.PIOVER2 = Math.PI / 2.;
 
	 /** The Constant TWOPI. */
	 Constants.TWOPI = 2 * Math.PI;//6.283185307179586476925286766559005768394;// 2 * PI
 
	 /** The Constant TWOTHIRD. */
	 Constants.TWOTHIRD = 2. / 3.;
 
	 /** The Constant 1 arcsecond in units of radians. */
	 Constants.ARCSECOND_RADIAN = 4.84813681109536e-6;

/**
 * HEALPix Javascript code derived from the jhealpix Java library
 * 
 * Class HealpixIndex
 * 
 * Main methods :
 * - ang2pix_nest
 * - pix2ang_nest
 * - nest2ring
 * - corners_nest
 * - queryDisc
 * - calculateNSide
 * 
 * @author: Thomas Boch [CDS]
 */


export let HealpixIndex = (function () {
	/**
	 * Some utility functions
	 *
	 * @author Thomas Boch [CDS]
	 *
	 */

	let Utils = function () { }

	Utils.radecToPolar = function (ra, dec) {
		return {
			theta: Math.PI / 2. - dec / 180. * Math.PI,
			phi: ra / 180. * Math.PI
		};
	}

	Utils.polarToRadec = function (theta, phi) {
		return {
			ra: phi * 180. / Math.PI,
			dec: (Math.PI / 2. - theta) * 180. / Math.PI
		};
	}


	Utils.castToInt = function (nb) {
		if (nb > 0) {
			return Math.floor(nb);
		}
		else {
			return Math.ceil(nb);
		}
	}

	/**
 * HEALPix Javascript code derived from the jhealpix Java library
	* 
 * Class SpatialVector
	* 
 * @author: Thomas Boch[CDS]
	* /



	/**
 * HEALPix Javascript code derived from the jhealpix Java library
 * 
 * Class AngularPosition
 * 
 * @author: Thomas Boch [CDS]
 */


	let AngularPosition = (function () {

		/** Constructor
		 * 
		 *  @theta theta in radians [0, 2*Pi] 
		 *  @phi phi in radians [0, Pi]
		 */
		function AngularPosition(theta, phi) {
			"use strict";
			this.theta = theta;
			this.phi = phi;
		};

		AngularPosition.prototype.toString = function () {
			"use strict";
			return "theta: " + this.theta + ", phi: " + this.phi;
		};

		return AngularPosition;
	})();

	/**
 * HEALPix Javascript code derived from the jhealpix Java library
 * 
 * Class LongRangeSetBuilder
 * 
 * @author: Thomas Boch [CDS]
 */

	let LongRangeSetBuilder = (function () {
		/* Constructor */
		function LongRangeSetBuilder() {
			this.items = [];
		}

		LongRangeSetBuilder.prototype.appendRange = function (lo, hi) {
			for (var i = lo; i <= hi; i++) {
				if (i in this.items) {
					continue;
				}

				this.items.push(i);
			}
		};
		return LongRangeSetBuilder;
	})();



	/** Constructor */
	function HealpixIndex(nside) {
		"use strict";
		this.nside = nside;
	};

	/** Constants * */
	HealpixIndex.NS_MAX = 16384/*536870912*/;

	HealpixIndex.ORDER_MAX = 14/*29*/;


	/** Available nsides ..always power of 2 ..* */
	HealpixIndex.NSIDELIST = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048,
		4096, 8192, 16384/*, 32768, 65536, 131072, 262144, 524288,
                               1048576, 2097152, 4194304, 8388608, 16777216, 33554432,
                               67108864, 134217728,  268435456, 536870912*/ ];

	// coordinate of the lowest corner of each face
	HealpixIndex.JRLL = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
	HealpixIndex.JPLL = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];

	HealpixIndex.XOFFSET = [-1, -1, 0, 1, 1, 1, 0, -1];
	HealpixIndex.YOFFSET = [0, 1, 1, 1, 0, -1, -1, -1];
	HealpixIndex.FACEARRAY =
		[[8, 9, 10, 11, -1, -1, -1, -1, 10, 11, 8, 9],   // S
		[5, 6, 7, 4, 8, 9, 10, 11, 9, 10, 11, 8],   // SE
		[-1, -1, -1, -1, 5, 6, 7, 4, -1, -1, -1, -1],   // E
		[4, 5, 6, 7, 11, 8, 9, 10, 11, 8, 9, 10],   // SW
		[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],   // center
		[1, 2, 3, 0, 0, 1, 2, 3, 5, 6, 7, 4],   // NE
		[-1, -1, -1, -1, 7, 4, 5, 6, -1, -1, -1, -1],   // W
		[3, 0, 1, 2, 3, 0, 1, 2, 4, 5, 6, 7],   // NW
		[2, 3, 0, 1, -1, -1, -1, -1, 0, 1, 2, 3]]; // N
	HealpixIndex.SWAPARRAY =
		[[0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3],   // S
		[0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6],   // SE
		[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],   // E
		[0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5],   // SW
		[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],   // center
		[5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0],   // NE
		[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],   // W
		[6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0],   // NW
		[3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0]]; // N
	/** The Constant z0. */
	HealpixIndex.Z0 = Constants.TWOTHIRD; // 2/3



	HealpixIndex.prototype.init = function () {
		"use strict";
		var tabmax = 0x100;
		this.ctab = new Array(tabmax);
		this.utab = new Array(tabmax);
		for (var m = 0; m < 0x100; ++m) {
			this.ctab[m] = ((m & 0x1) | ((m & 0x2) << 7) | ((m & 0x4) >> 1) | ((m & 0x8) << 6) |
				((m & 0x10) >> 2) | ((m & 0x20) << 5) | ((m & 0x40) >> 3) | ((m & 0x80) << 4));
			this.utab[m] = ((m & 0x1) | ((m & 0x2) << 1) | ((m & 0x4) << 2) | ((m & 0x8) << 3) |
				((m & 0x10) << 4) | ((m & 0x20) << 5) | ((m & 0x40) << 6) | ((m & 0x80) << 7));
		}

		// end tablefiller
		this.nl2 = 2 * this.nside;
		this.nl3 = 3 * this.nside;
		this.nl4 = 4 * this.nside;
		this.npface = this.nside * this.nside;
		this.ncap = 2 * this.nside * (this.nside - 1);// points in each polar cap,
		// =0 for

		this.npix = 12 * this.npface;
		this.fact2 = 4.0 / this.npix;
		this.fact1 = (this.nside << 1) * this.fact2;

		this.order = HealpixIndex.nside2order(this.nside);
	}

    /**
     * calculate required nside given pixel size in arcsec
     *
     * @param pixsize in arcsec
     * @return long nside parameter
     */
	HealpixIndex.calculateNSide = function (pixsize) {
		var res = 0;
		var pixelArea = pixsize * pixsize;
		var degrad = 180. / Constants.PI;
		var skyArea = 4. * Constants.PI * degrad * degrad * 3600. * 3600.;
		var npixels = Utils.castToInt(skyArea / pixelArea);
		var nsidesq = npixels / 12;
		var nside_req = Math.sqrt(nsidesq);
		var mindiff = HealpixIndex.NS_MAX;
		var indmin = 0;
		for (var i = 0; i < HealpixIndex.NSIDELIST.length; i++) {
			if (Math.abs(nside_req - HealpixIndex.NSIDELIST[i]) <= mindiff) {
				mindiff = Math.abs(nside_req - HealpixIndex.NSIDELIST[i]);
				res = HealpixIndex.NSIDELIST[i];
				indmin = i;
			}
			if ((nside_req > res) && (nside_req < HealpixIndex.NS_MAX))
				res = HealpixIndex.NSIDELIST[indmin + 1];
			if (nside_req > HealpixIndex.NS_MAX) {
				console.log("nside cannot be bigger than " + HealpixIndex.NS_MAX);
				return HealpixIndex.NS_MAX;
			}

		}
		return res;
	}
    /**
     * static method to find order from nside
     * 
     * @param nside
     * @return order
     */
	HealpixIndex.nside2order = function (nside) {
		"use strict";
		if ((nside & (nside - 1)) > 0) {
			return -1;
		}
		return Utils.castToInt(HealpixIndex.log2(nside));
	}

    /**
     * Log base two
     * 
     * @param num
     * @return log2
     */
	HealpixIndex.log2 = function (num) {
		"use strict";
		return (Math.log(num) / Math.log(2));
	}


    /**
     * TESTED. Works OK for nside<=8192
     *
     * renders the pixel number ipix ( scheme as defined for object) for a pixel
     * which contains a point on a sphere at coordinates theta and phi, given the
     * map resolution parameter nside
     * 
     * @param theta
     *            angle (along meridian), in [0,Pi], theta=0 : north pole
     * @param phi
     *            angle (along parallel), in [0,2*Pi]
     * @return pixel index number
     * @throws Exception
     */
	HealpixIndex.prototype.ang2pix_nest = function (theta, phi) {
		"use strict";

		var ipix;
		var z, za, tt, tp;
		var ifp, ifm;
		var jp, jm;
		var ntt, face_num, ix, iy;

		if (phi >= Constants.TWOPI)
			phi = phi - Constants.TWOPI;
		if (phi < 0.)
			phi = phi + Constants.TWOPI;
		if (theta > Constants.PI || theta < 0) {
			throw { name: "Illegal argument", message: "theta must be between 0 and " + Constants.PI };
		}
		if (phi > Constants.TWOPI || phi < 0) {
			throw { name: "Illegal argument", message: "phi must be between 0 and " + Constants.TWOPI };
		}

		z = Math.cos(theta);
		za = Math.abs(z);
		tt = phi / Constants.PIOVER2;// in [0,4]


		if (za <= HealpixIndex.Z0) { // Equatorial region
			// (the index of edge lines increase when the longitude=phi goes up)
			var temp1 = this.nside * (0.5 + tt);
			var temp2 = this.nside * (z * 0.75);

			var jp = (temp1 - temp2);
			// ascending edge line index
			var jm = (temp1 + temp2);
			// descending edge line index

			// finds the face
			ifp = jp >> this.order; // in {0,4}
			ifm = jm >> this.order;
			if (ifp == ifm) { // faces 4 to 7
				face_num = (ifp == 4 ? 4 : ifp + 4);
			} else {
				if (ifp < ifm) { // (half-)faces 0 to 3
					face_num = ifp;
				} else { // (half-)faces 8 to 11
					face_num = ifm + 8;
				}
			}

			ix = Utils.castToInt(jm & (this.nside - 1));
			iy = Utils.castToInt(this.nside - (jp & (this.nside - 1)) - 1);
		} else { // polar region, za > 2/3
			ntt = Utils.castToInt(tt);
			if (ntt >= 4)
				ntt = 3;
			tp = tt - ntt;
			var tmp = this.nside * Math.sqrt(3.0 * (1.0 - za));


			// (the index of edge lines increase when distance from the closest
			// pole goes up)
			jp = Utils.castToInt(tp * tmp);// line going toward the
			// pole as phi increases
			jm = Utils.castToInt((1.0 - tp) * tmp); // that one goes
			// away of the closest pole
			jp = Math.min(HealpixIndex.NS_MAX - 1, jp);
			// for points too close to the boundary
			jm = Math.min(HealpixIndex.NS_MAX - 1, jm);



			// finds the face and pixel's (x,y)
			if (z >= 0) { // North Pole
				// System.out.println("Polar z>=0 ntt:"+ntt+" tt:"+tt);
				face_num = ntt; // in {0,3}
				ix = Utils.castToInt(this.nside - jm - 1);
				iy = Utils.castToInt(this.nside - jp - 1);

			} else {
				// System.out.println("Polar z<0 ntt:"+ntt+" tt:"+tt);
				face_num = ntt + 8;// in {8,11}
				ix = jp;
				iy = jm;
			}
		}

		ipix = this.xyf2nest(ix, iy, face_num);

		return ipix;
	}

	HealpixIndex.prototype.xyf2nest = function (ix, iy, face_num) {
		"use strict";
		return ((face_num) << (2 * this.order)) +
			(((this.utab[ix & 0xff]))
				| ((this.utab[(ix >> 8) & 0xff]) << 16)
				| ((this.utab[(ix >> 16) & 0xff]) << 32)
				| ((this.utab[(ix >> 24) & 0xff]) << 48)
				| ((this.utab[iy & 0xff]) << 1)
				| ((this.utab[(iy >> 8) & 0xff]) << 17)
				| ((this.utab[(iy >> 16) & 0xff]) << 33)
				| ((this.utab[(iy >> 24) & 0xff]) << 49));
	}

	HealpixIndex.prototype.nest2xyf = function (ipix) {
		"use strict";
		var ret = {};
		ret.face_num = ipix >> (2 * this.order);
		var pix = ipix & (this.npface - 1);
		// need o check the & here - they were unsigned in cpp ...
		var raw = (((pix & 0x555500000000) >> 16)
			| ((pix & 0x5555000000000000) >> 31)
			| (pix & 0x5555)
			| ((pix & 0x55550000) >> 15));
		ret.ix = this.ctab[raw & 0xff]
			| (this.ctab[(raw >> 8) & 0xff] << 4)
			| (this.ctab[(raw >> 16) & 0xff] << 16)
			| (this.ctab[(raw >> 24) & 0xff] << 20);
		pix >>= 1;
		raw = (((pix & 0x555500000000) >> 16)
			| ((pix & 0x5555000000000000) >> 31)
			| (pix & 0x5555)
			| ((pix & 0x55550000) >> 15));
		ret.iy = this.ctab[raw & 0xff]
			| (this.ctab[(raw >> 8) & 0xff] << 4)
			| (this.ctab[(raw >> 16) & 0xff] << 16)
			| (this.ctab[(raw >> 24) & 0xff] << 20);

		return ret;
	}

    /**
     * TESTED. Works OK for nside<=8192
     * 
     * Convert from pix number to angle renders theta and phi coordinates of the
     * nominal pixel center for the pixel number ipix (NESTED scheme) given the
     * map resolution parameter nside
     *
     * @param ipix
     *            pixel index number
     * @return double array of [theta, phi] angles in radians [0,Pi], [0,2*Pi]
     * @throws Exception if ipix not in expected range for norder
     */
	HealpixIndex.prototype.pix2ang_nest = function (ipix) {
		"use strict";

		if (ipix < 0 || ipix > this.npix - 1) {
			throw { name: "Illegal argument", message: "ipix out of range" };
		}

		var x = this.nest2xyf(ipix);

		var ix = x.ix;
		var iy = x.iy;
		var face_num = x.face_num;

		// TODO this c++ bit shift givesa differnt jr to the Healpix Code - why ?
		var jr = ((HealpixIndex.JRLL[face_num] << this.order)) - ix - iy - 1;
		var nr, z, kshift;

		// ring number in {1,4*nside-1}

		if (jr < this.nside) { // north pole region
			nr = jr;
			z = 1.0 - nr * nr * this.fact2;
			kshift = 0;
		} else if (jr > this.nl3) { // south pole region
			nr = this.nl4 - jr;
			z = nr * nr * this.fact2 - 1.0;
			kshift = 0;
		} else {
			nr = this.nside;
			z = (this.nl2 - jr) * this.fact1;
			kshift = (jr - this.nside) & 1;
		}
		var theta = Math.acos(z);

		// computes the phi coordinate on the sphere, in [0,2Pi]
		var jp = (HealpixIndex.JPLL[face_num] * nr + ix - iy + 1 + kshift) / 2;
		// 'phi' number in the ring in {1,4*nr}
		if (jp > this.nl4) {
			jp = jp - this.nl4;
		}
		if (jp < 1) {
			jp = jp + this.nl4;
		}

		var phi = (jp - (kshift + 1) * 0.50) * (Constants.PIOVER2 / nr);

		// if (phi < 0)
		// phi += 2.0 * Math.PI; // phi in [0, 2pi]

		return { theta: theta, phi: phi };
	}

	HealpixIndex.nside2Npix = function (nside) {
		"use strict";

		// check if power of 2 and if nside<NS_MAX
		if (nside < 0 || (nside & -nside) != nside || nside > HealpixIndex.NS_MAX) {
			throw { name: "Illegal argument", message: "nside should be >0, power of 2, <" + HealpixIndex.NS_MAX };
		}
		var res = 12 * nside * nside;
		return res;
	}

	HealpixIndex.prototype.xyf2ring = function (ix, iy, face_num) {
		"use strict";

		var jr = HealpixIndex.JRLL[face_num] * this.nside - ix - iy - 1;

		var nr, kshift, n_before;
		if (jr < this.nside) {
			nr = jr;
			n_before = 2 * nr * (nr - 1);
			kshift = 0;
		}
		else if (jr > 3 * this.nside) {
			nr = this.nl4 - jr;
			n_before = this.npix - 2 * (nr + 1) * nr;
			kshift = 0;
		}
		else {
			nr = this.nside;
			n_before = this.ncap + (jr - this.nside) * this.nl4;
			kshift = (jr - this.nside) & 1;
		}

		var jp = (HealpixIndex.JPLL[face_num] * nr + ix - iy + 1 + kshift) / 2;
		if (jp > this.nl4) {
			jp -= this.nl4;
		}
		else {
			if (jp < 1) {
				jp += this.nl4;
			}
		}

		return n_before + jp - 1;
	}

    /**
     * 
     * TESTED. Works OK for nside<=8192
     * 
     * performs conversion from NESTED to RING pixel number
     *
     * @param ipnest
     *            pixel NEST index number
     * @return RING pixel index number
     * @throws Exception
     */
	HealpixIndex.prototype.nest2ring = function (ipnest) {
		"use strict";
		var xyf = this.nest2xyf(ipnest);
		var ipring = this.xyf2ring(xyf.ix, xyf.iy, xyf.face_num);
		return ipring;
	}

    /**
     * 
     * TESTED. Works OK for nside<=8192
     * 
     * Returns set of points along the boundary of the given pixel in NEST
     * scheme. Step 1 gives 4 points on the corners.
     *
     * @param pix
     *            pixel index number in nest scheme
     * @param step
     * @return {@link SpatialVector} for each points
     * @throws Exception
     */
	HealpixIndex.prototype.corners_nest = function (pix, step) {
		"use strict";

		var pixr = this.nest2ring(pix);
		return this.corners_ring(pixr, step);
	}


    /**
     * Convert from pix number to angle renders theta and phi coordinates of the
     * nominal pixel center for the pixel number ipix (RING scheme) given the
     * map resolution parameter nside
     *
     * @param ipix
     *            pixel index number
     * @return double array of [theta, phi] angles in radians [0,Pi], [0,2*Pi]
     * @throws Exception
     */
	HealpixIndex.prototype.pix2ang_ring = function (ipix) {
		"use strict";

		var theta, phi;
		var iring, iphi, ip, ipix1;
		var fodd, hip, fihip;
		// -----------------------------------------------------------------------
		if (ipix < 0 || ipix > this.npix - 1) {
			throw { name: "Illegal argument", message: "ipix out of range" };
		}

		ipix1 = ipix + 1;// in {1, npix}
		if (ipix1 <= this.ncap) { // North Polar cap -------------

			hip = ipix1 / 2.0;
			fihip = Utils.castToInt(hip);
			iring = Utils.castToInt(Math.sqrt(hip - Math.sqrt(fihip))) + 1;
			// counted from North pole
			iphi = ipix1 - 2 * iring * (iring - 1);

			theta = Math.acos(1.0 - (iring * iring * this.fact2));
			phi = ((iphi) - 0.50) * Constants.PI / (2.0 * iring);

		} else {
			if (ipix < (this.npix - this.ncap)) { // Equatorial region
				ip = ipix - this.ncap;
				iring = (ip / this.nl4) + this.nside;// counted from North pole
				iphi = ip % this.nl4 + 1;

				fodd = (((iring + this.nside) & 1) > 0) ? 1 : 0.5;
				// 1 if iring+nside is odd, 1/2 otherwise
				theta = Math.acos((this.nl2 - iring) * this.fact1);
				phi = ((iphi) - fodd) * Constants.PI
					/ this.nl2;
			} else { // South Polar cap -----------------------------------
				ip = this.npix - ipix;
				iring = Utils.castToInt(0.5 * (1 + Math.sqrt(2 * ip - 1)));
				// counted from South pole
				iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));

				theta = Math.acos(-1.0 + Math.pow(iring, 2) * this.fact2);
				phi = ((iphi) - 0.50) * Constants.PI
					/ (2.0 * iring);

			}
		};

		return [theta, phi];
	}

    /**
     * return ring number for given pix in ring scheme
     *
     * @param ipix
     *            pixel index number in ring scheme
     * @return ring number
     * @throws Exception
     */
	HealpixIndex.prototype.ring = function (ipix) {
		"use strict";
		var iring = 0;
		var ipix1 = ipix + 1;// in {1, npix}
		var ip;
		var hip, fihip = 0;
		if (ipix1 <= this.ncap) { // North Polar cap -------------
			hip = (ipix1 / 2.0);
			fihip = Utils.castToInt(hip);
			iring = Utils.castToInt(Math.sqrt(hip - Math.sqrt(fihip))) + 1;// counted
			// from
			// North
			// pole
		} else {
			if (ipix1 <= this.nl2 * (5 * this.nside + 1)) { // Equatorial region
				// ------
				ip = Utils.castToInt(ipix1 - this.ncap - 1);
				iring = Utils.castToInt((ip / this.nl4) + this.nside);// counted from North pole
			} else { // South Polar cap -----------------------------------
				ip = (this.npix - ipix1 + 1);
				hip = (ip / 2.0);
				fihip = Utils.castToInt(hip);
				iring = Utils.castToInt(Math.sqrt(hip - Math.sqrt(fihip))) + 1;// counted
				// from
				// South
				// pole
				iring = (this.nl4 - iring);
			}
		}
		return iring;
	}

    /**
     * integration limits in cos(theta) for a given ring i_th, i_th > 0
     *
     * @param i_th
     *            ith ring
     * @return limits
     */
	HealpixIndex.prototype.integration_limits_in_costh = function (i_th) {
		"use strict";
		var a, ab, b, r_n_side;
		// integration limits in cos(theta) for a given ring i_th
		// i > 0 !!!
		r_n_side = 1.0 * this.nside;
		if (i_th <= this.nside) {
			ab = 1.0 - (Math.pow(i_th, 2.0) / 3.0) / this.npface;
			b = 1.0 - (Math.pow((i_th - 1), 2.0) / 3.0) / this.npface;
			if (i_th == this.nside) {
				a = 2.0 * (this.nside - 1.0) / 3.0 / r_n_side;
			} else {
				a = 1.0 - Math.pow((i_th + 1), 2) / 3.0 / this.npface;
			}
		} else {
			if (i_th < this.nl3) {
				ab = 2.0 * (2 * this.nside - i_th) / 3.0 / r_n_side;
				b = 2.0 * (2 * this.nside - i_th + 1) / 3.0 / r_n_side;
				a = 2.0 * (2 * this.nside - i_th - 1) / 3.0 / r_n_side;
			} else {
				if (i_th == this.nl3) {
					b = 2.0 * (-this.nside + 1) / 3.0 / r_n_side;
				} else {
					b = -1.0 + Math.pow((4 * this.nside - i_th + 1), 2) / 3.0
						/ this.npface;
				}

				a = -1.0 + Math.pow((this.nl4 - i_th - 1), 2) / 3.0
					/ this.npface;
				ab = -1.0 + Math.pow((this.nl4 - i_th), 2) / 3.0 / this.npface;
			}

		}
		// END integration limits in cos(theta)
		return [b, ab, a];
	}

    /**
     * calculate the points of crossing for a given theata on the boundaries of
     * the pixel - returns the left and right phi crossings
     *
     * @param i_th
     *            ith pixel
     * @param i_phi
     *            phi angle
     * @param i_zone
     *            ith zone (0,...,3), a quarter of sphere
     * @param cos_theta
     *            theta cosinus
     * @return the left and right phi crossings
     */
	HealpixIndex.prototype.pixel_boundaries = function (i_th, i_phi, i_zone, cos_theta) {
		var sq3th, factor, jd, ju, ku, kd, phi_l, phi_r;
		var r_n_side = 1.0 * this.nside;

		// HALF a pixel away from both poles
		if (Math.abs(cos_theta) >= 1.0 - 1.0 / 3.0 / this.npface) {
			phi_l = i_zone * Constants.PIOVER2;
			phi_r = (i_zone + 1) * Constants.PIOVER2;
			return [phi_l, phi_r];
		}
		// -------
		// NORTH POLAR CAP
		if (1.50 * cos_theta >= 1.0) {
			sq3th = Math.sqrt(3.0 * (1.0 - cos_theta));
			factor = 1.0 / r_n_side / sq3th;
			jd = (i_phi);
			ju = jd - 1;
			ku = (i_th - i_phi);
			kd = ku + 1;

			phi_l = Constants.PIOVER2
				* (Math.max((ju * factor), (1.0 - (kd * factor))) + i_zone);
			phi_r = Constants.PIOVER2
				* (Math.min((1.0 - (ku * factor)), (jd * factor)) + i_zone);

		} else {
			if (-1.0 < 1.50 * cos_theta) {
				// -------
				// -------
				// EQUATORIAL ZONE
				var cth34 = 0.50 * (1.0 - 1.50 * cos_theta);
				var cth34_1 = cth34 + 1.0;
				var modfactor = (this.nside + (i_th % 2));

				jd = i_phi - (modfactor - i_th) / 2.0;
				ju = jd - 1;
				ku = (modfactor + i_th) / 2.0 - i_phi;
				kd = ku + 1;

				phi_l = Constants.PIOVER2
					* (Math.max((cth34_1 - (kd / r_n_side)),
						(-cth34 + (ju / r_n_side))) + i_zone);

				phi_r = Constants.PIOVER2
					* (Math.min((cth34_1 - (ku / r_n_side)),
						(-cth34 + (jd / r_n_side))) + i_zone);
				// -------
				// -------
				// SOUTH POLAR CAP

			} else {
				sq3th = Math.sqrt(3.0 * (1.0 + cos_theta));
				factor = 1.0 / r_n_side / sq3th;
				var ns2 = 2 * this.nside;

				jd = i_th - ns2 + i_phi;
				ju = jd - 1;
				ku = ns2 - i_phi;
				kd = ku + 1;

				phi_l = Constants.PIOVER2
					* (Math.max((1.0 - (ns2 - ju) * factor),
						((ns2 - kd) * factor)) + i_zone);

				phi_r = Constants.PIOVER2
					* (Math.min((1.0 - (ns2 - jd) * factor),
						((ns2 - ku) * factor)) + i_zone);
			}// of SOUTH POLAR CAP
		}
		// and that's it
		// System.out.println(" nside:"+nside+" i_th:"+i_th+" i_phi:"+i_phi+"
		// izone:"+i_zone+" cos_theta:"+cos_theta+" phi_l:"+phi_l+"
		// phi_r:"+phi_r);

		return [phi_l, phi_r];
	}

    /**
     * Construct a {@link SpatialVector} from the angle (theta,phi)
     *
     * @param theta
     *            angle (along meridian), in [0,Pi], theta=0 : north pole
     * @param phi
     *            angle (along parallel), in [0,2*Pi]
     * @return vector {@link SpatialVector}
     */
	HealpixIndex.vector = function (theta, phi) {
		"use strict";
		var x = 1 * Math.sin(theta) * Math.cos(phi);
		var y = 1 * Math.sin(theta) * Math.sin(phi);
		var z = 1 * Math.cos(theta);
		return new SpatialVector(x, y, z);
	}

    /**
     * Returns set of points along the boundary of the given pixel in RING
     * scheme. Step 1 gives 4 points on the corners.
     * Mainly for graphics = you may not want to use LARGE NSIDEs..
     *
     * @param pix
     *            pixel index number in ring scheme
     * @param step
     * @return {@link SpatialVector} for each points
     * @throws Exception
     */
	HealpixIndex.prototype.corners_ring = function (pix, step) {
		"use strict";

		var nPoints = step * 2 + 2;
		var points = new Array(nPoints);
		var p0 = this.pix2ang_ring(pix);
		var cos_theta = Math.cos(p0[0]);
		var theta = p0[0];
		var phi = p0[1];

		var i_zone = Utils.castToInt(phi / Constants.PIOVER2);
		var ringno = this.ring(pix);
		var i_phi_count = Math.min(ringno, Math.min(this.nside, (this.nl4) - ringno));
		var i_phi = 0;
		var phifac = Constants.PIOVER2 / i_phi_count;
		if (ringno >= this.nside && ringno <= this.nl3) {
			// adjust by 0.5 for odd numbered rings in equatorial since
			// they start out of phase by half phifac.
			i_phi = Utils.castToInt(phi / phifac + ((ringno % 2) / 2.0)) + 1;
		} else {
			i_phi = Utils.castToInt(phi / phifac) + 1;
		}
		// adjust for zone offset
		i_phi = i_phi - (i_zone * i_phi_count);
		var spoint = (nPoints / 2);

		// get north south middle - middle should match theta !
		var nms = this.integration_limits_in_costh(ringno);
		var ntheta = Math.acos(nms[0]);
		var stheta = Math.acos(nms[2]);
		var philr = this.pixel_boundaries(ringno, i_phi, i_zone, nms[0]);

		if (i_phi > (i_phi_count / 2)) {
			points[0] = HealpixIndex.vector(ntheta, philr[1]);
		} else {
			points[0] = HealpixIndex.vector(ntheta, philr[0]);
		}
		philr = this.pixel_boundaries(ringno, i_phi, i_zone, nms[2]);
		if (i_phi > (i_phi_count / 2)) {
			points[spoint] = HealpixIndex.vector(stheta, philr[1]);
		} else {
			points[spoint] = HealpixIndex.vector(stheta, philr[0]);
		}
		if (step == 1) {
			var mtheta = Math.acos(nms[1]);
			philr = this.pixel_boundaries(ringno, i_phi, i_zone, nms[1]);
			points[1] = HealpixIndex.vector(mtheta, philr[0]);
			points[3] = HealpixIndex.vector(mtheta, philr[1]);
		} else {
			var cosThetaLen = nms[2] - nms[0];
			var cosThetaStep = (cosThetaLen / (step + 1)); // skip
			// North
			// and south
			for (var p = 1; p <= step; p++) {
				/* Integrate points along the sides */
				cos_theta = nms[0] + (cosThetaStep * p);
				theta = Math.acos(cos_theta);
				philr = this.pixel_boundaries(ringno, i_phi, i_zone, cos_theta);
				points[p] = HealpixIndex.vector(theta, philr[0]);
				points[nPoints - p] = HealpixIndex.vector(theta, philr[1]);
			}
		}
		return points;
	}

    /**
     * converts a SpatialVector in a tuple of angles tup[0] = theta co-latitude
     * measured from North pole, in [0,PI] radians, tup[1] = phi longitude
     * measured eastward, in [0,2PI] radians
     *
     * @param v
     *            SpatialVector
     * @return double[] out_tup out_tup[0] = theta out_tup[1] = phi
     */
	HealpixIndex.vec2Ang = function (v) {
		"use strict";

		var z = v.z / v.length();
		var theta = Math.acos(z);
		var phi = 0.;
		if ((v.x != 0.) || (v.y != 0)) {
			phi = Math.atan2(v.y, v.x); // phi in [-pi,pi]
		}
		if (phi < 0)
			phi += 2.0 * Math.PI; // phi in [0, 2pi]
		return [theta, phi];
	}

    /**
     * generates in the RING or NESTED scheme all pixels that lies within an
     * angular distance Radius of the center.
     *
     * TESTED. Works OK for nside<=8192
     *
     * @param nside
     *            long map resolution
     * @param vector
     *            Vector3d pointing to the disc center
     * @param radius
     *            double angular radius of the disk (in RADIAN )
     * @param do_nest
     *            if true, output in NESTED scheme
     *            if false, output in RING scheme
     * @param do_inclusive
     *            if set to false: only pixels whose center lie in the triangle
     *            are listed, if set to true, all pixels overlapping the triangle
     *            are listed
     * @return ArrayList of pixel numbers calls: RingNum(nside, ir)
     *         InRing(nside, iz, phi0, dphi,nest)
     */
	HealpixIndex.prototype.queryDisc = function (vector, radius, do_nest, do_inclusive) {
		"use strict";

		if (radius < 0.0 || radius > Constants.PI) {
			throw { "name": "Illegal argument", "message": "angular radius is in RADIAN and should be in [0,pi]" };
		}

		var res = new LongRangeSetBuilder();
		var irmin, irmax, iz;
		var ang = null;
		var z0, radius_eff, theta, phi, cosang, x, ysq;
		var dth1, dth2, dphi;
		var rlat1, rlat2, zmin, zmax, z, xa;

		var radius_eff = radius;
		if (do_inclusive) {
			radius_eff += Constants.PI / (this.nl4); // increase radius by
			// half pixel: different in C++ version where a 'magic' number is used.
		}

		// this pix back abnf fourth is ok until you put in  precise vector like a pole .
		// then it shifts the whole elipse...
		ang = HealpixIndex.vec2Ang(vector);

		theta = ang[0];
		phi = ang[1];
		dth1 = this.fact2;
		dth2 = this.fact1;
		z0 = Math.cos(theta);
		xa = 1. / Math.sqrt((1.0 - z0) * (1.0 + z0));

		/* coordinate z of highest and lowest points in the disc */

		rlat1 = theta - radius_eff;
		rlat2 = theta + radius_eff;


		cosang = Math.cos(radius_eff);
		zmax = Math.cos(rlat1);
		irmin = this.ringAbove(zmax) + 1;
		zmin = Math.cos(rlat2);
		irmax = this.ringAbove(zmin);

		if (irmax < irmin) {// in this case no pixels are returned - need irmax=irmin to loop
			if (irmax == 0) {
				irmax = irmin;
			}
		}

		if (rlat1 <= 0) {// north pole in the disc
			for (var m = 1; m < irmin; ++m) {// rings completely in the disc
				this.inRing(m, 0, Math.PI, res);
			}
		}

		/* loop on ring number */
		for (iz = irmin; iz <= irmax; ++iz) {
			if (iz < this.nside) { // north polar cap
				z = 1.0 - iz * iz * dth1;
			} else if (iz <= (this.nl3)) { // tropical band + equator
				z = (this.nl2 - iz) * dth2;
			} else {
				z = -1.0 + (this.nl4 - iz) * (this.nl4 - iz) * dth1;
			}
			/* find phi range in the disc for each z */
			x = (cosang - z * z0) * xa;
			ysq = 1.0 - z * z - x * x;
			// up north (and south ?) this atan does not work
			// dphi becomes NaN.
			dphi = Math.atan2(Math.sqrt(ysq), x);
			if (isNaN(dphi)) {
				dphi = radius_eff;
			}
			this.inRing(iz, phi, dphi, res);

		}
		if (rlat2 >= Math.PI) {// south pole in the disc
			for (var m = irmax + 1; m < (this.nl4); ++m) {
				// rings completely in the disc
				this.inRing(m, 0, Math.PI, res, false);
			}
		}

		var ret;
		if (do_nest) {
			var items = res.items;
			var items_nest = [];
			for (var i = 0; i < items.length; i++) {
				var nestIdx = this.ring2nest(items[i]);
				if (items_nest.indexOf(nestIdx) >= 0) {
					continue;
				}
				items_nest.push(nestIdx);
			}
			ret = items_nest;
		}
		else {
			ret = res.items;
		}

		return ret;

	}

    /**
     * returns the list of pixels in RING scheme with latitude in [phi0 -
     * dpi, phi0 + dphi] on the ring iz in [1, 4*nside -1 ] The pixel id numbers
     * are in [0, 12*nside^2 - 1] the indexing is in RING, unless nest is set to
     * 1
     * NOTE: this is the f90 code 'in_ring' method ported to java with 'conservative' flag to false
     *
     * @param nside
     *            long the map resolution
     * @param iz
     *            long ring number
     * @param phi0
     *            double
     * @param dphi
     *            double
     * @param res result
     */
	HealpixIndex.prototype.inRing = function (iz, phi0, dphi, res, conservative) {
		"use strict";

		var take_all = false;
		var to_top = false;

		//	String SID = "InRing:";
		var epsilon = 1e-12;//Double.MIN_VALUE; // the constant to eliminate
		// java calculation jitter
		var shift = 0.;
		var ir = 0;
		var kshift, nr, ipix1, ipix2;//nir1, nir2,
		var ip_low = 0, ip_hi = 0; //,in, nir;
		//	long inext;

		var phi_low = ((phi0 - dphi) % Constants.TWOPI) - epsilon; // phi min,															  // excluding
		// 2pi period
		//	double phi_low = phi0 - dphi - epsilon; // phi min,
		// excluding
		var phi_hi = phi0 + dphi + epsilon;

		// this was being moduloed but why ?? around the 2pi that casues a problem
		var phi_hi_mod = ((phi0 + dphi) % Constants.TWOPI) + epsilon;

		//
		if (Math.abs(dphi - Constants.PI) < epsilon) {
			take_all = true;
		}
		// what happens when phi_hi wraps round ??

		/* identifies ring number */
		if ((iz >= this.nside) && (iz <= this.nl3)) { // equatorial region
			ir = iz - this.nside + 1; // in [1, 2*nside + 1]
			ipix1 = this.ncap + this.nl4 * (ir - 1); // lowest pixel number in the
			// ring
			ipix2 = ipix1 + this.nl4 - 1; // highest pixel number in the ring
			kshift = ir % 2;

			nr = this.nl4;
		}
		else {
			if (iz < this.nside) { // north pole
				ir = iz;
				ipix1 = 2 * ir * (ir - 1); // lowest pixel number
				ipix2 = ipix1 + (4 * ir) - 1; // highest pixel number
			} else { // south pole
				ir = 4 * this.nside - iz;

				ipix1 = this.npix - 2 * ir * (ir + 1); // lowest pixel number
				ipix2 = ipix1 + 4 * ir - 1;       // highest pixel number
			}
			nr = ir * 4;
			kshift = 1;
		}

		// Construct the pixel list
		if (take_all) {
			res.appendRange(ipix1, ipix2);
			return;
		}

		shift = kshift / 2.0;

		// conservative : include every intersected pixel, even if the
		// pixel center is out of the [phi_low, phi_hi] region
		if (conservative) {
			ip_low = Math.round((nr * phi_low) / Constants.TWOPI - shift);
			ip_hi = Math.round((nr * phi_hi) / Constants.TWOPI - shift);

			ip_low = (ip_low % nr); // in [0, nr - 1]
			if (ip_hi > nr) { // ifit is =nr then this sets it to zero - not good
				ip_hi = (ip_hi % nr); // in [0, nr - 1]
			}
			//		System.out.println("ip_low="+ip_low+" ip_hi="+ip_hi);
		}
		else { // strict: includes only pixels whose center is in
			//                                                    [phi_low,phi_hi]

			ip_low = Math.ceil((nr * phi_low) / Constants.TWOPI - shift);
			ip_hi = Utils.castToInt((nr * phi_hi_mod) / Constants.TWOPI - shift);
			if (ip_hi < ip_low && iz == 1) {//this is not good - problem on pole with direction.
				ip_hi = Utils.castToInt((nr * phi_hi) / Constants.TWOPI - shift);
			}
			if (ip_low == ip_hi + 1) {
				ip_low = ip_hi;
			}

			if ((ip_low - ip_hi == 1) && (dphi * nr < Constants.PI)) {
				// the interval is too small ( and away from pixel center)
				// so no pixels is included in the list

				console.log("the interval is too small and avay from center");

				return; // return empty list
			}

			ip_low = Math.min(ip_low, nr - 1);
			ip_hi = Math.max(ip_hi, 0);
		}

		//
		if (ip_low > ip_hi) {
			to_top = true;
		}

		if (to_top) {
			ip_low += ipix1;
			ip_hi += ipix1;

			res.appendRange(ipix1, ip_hi);
			res.appendRange(ip_low, ipix2);
		} else {
			if (ip_low < 0) {
				ip_low = Math.abs(ip_low);

				res.appendRange(ipix1, ipix1 + ip_hi);
				res.appendRange(ipix2 - ip_low + 1, ipix2);
				return;

			}
			ip_low += ipix1;
			ip_hi += ipix1;

			res.appendRange(ip_low, ip_hi);
		}
	}

	HealpixIndex.prototype.ringAbove = function (z) {
		"use strict";

		var az = Math.abs(z);
		if (az > Constants.TWOTHIRD) { // polar caps
			var iring = Utils.castToInt(this.nside * Math.sqrt(3 * (1 - az)));
			return (z > 0) ? iring : 4 * this.nside - iring - 1;
		}
		else { // ----- equatorial region ---------
			return Utils.castToInt(this.nside * (2.0 - 1.5 * z));
		}
	}

	HealpixIndex.prototype.ring2nest = function (ipring) {
		"use strict";

		var xyf = this.ring2xyf(ipring);
		return this.xyf2nest(xyf.ix, xyf.iy, xyf.face_num);
	}

	HealpixIndex.prototype.ring2xyf = function (pix) {
		"use strict";

		var ret = {};
		var iring, iphi, kshift, nr;

		if (pix < this.ncap) { // North Polar cap
			iring = Utils.castToInt(0.5 * (1 + Math.sqrt(1 + 2 * pix))); //counted from North pole
			iphi = (pix + 1) - 2 * iring * (iring - 1);
			kshift = 0;
			nr = iring;
			ret.face_num = 0;
			var tmp = iphi - 1;
			if (tmp >= (2 * iring)) {
				ret.face_num = 2;
				tmp -= 2 * iring;
			}
			if (tmp >= iring) {
				++ret.face_num;
			}
		}
		else if (pix < (this.npix - this.ncap)) { // Equatorial region
			var ip = pix - this.ncap;
			if (this.order >= 0) {
				iring = (ip >> (this.order + 2)) + this.nside; // counted from North pole
				iphi = (ip & (this.nl4 - 1)) + 1;
			}
			else {
				iring = (ip / (this.nl4)) + this.nside; // counted from North pole
				iphi = (ip % (this.nl4)) + 1;
			}
			kshift = (iring + this.nside) & 1;
			nr = this.nside;
			var ire = iring - this.nside + 1;
			var irm = this.nl2 + 2 - ire;
			var ifm, ifp;
			if (this.order >= 0) {
				ifm = (iphi - Utils.castToInt(ire / 2) + this.nside - 1) >> this.order;
				ifp = (iphi - Utils.castToInt(irm / 2) + this.nside - 1) >> this.order;
			}
			else {
				ifm = (iphi - Utils.castToInt(ire / 2) + this.nside - 1) / this.nside;
				ifp = (iphi - Utils.castToInt(irm / 2) + this.nside - 1) / this.nside;
			}
			if (ifp == ifm) { // faces 4 to 7
				ret.face_num = (ifp == 4) ? 4 : Utils.castToInt(ifp) + 4;
			}
			else if (ifp < ifm) { // (half-)faces 0 to 3
				ret.face_num = Utils.castToInt(ifp);
			}
			else { // (half-)faces 8 to 11
				ret.face_num = Utils.castToInt(ifm) + 8;
			}
		}
		else { // South Polar cap
			var ip = this.npix - pix;
			iring = Utils.castToInt(0.5 * (1 + Math.sqrt(2 * ip - 1))); //counted from South pole
			iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));
			kshift = 0;
			nr = iring;
			iring = 2 * this.nl2 - iring;
			ret.face_num = 8;
			var tmp = iphi - 1;
			if (tmp >= (2 * nr)) {
				ret.face_num = 10;
				tmp -= 2 * nr;
			}
			if (tmp >= nr) {
				++ret.face_num;
			}
		}

		var irt = iring - (HealpixIndex.JRLL[ret.face_num] * this.nside) + 1;
		var ipt = 2 * iphi - HealpixIndex.JPLL[ret.face_num] * nr - kshift - 1;
		if (ipt >= this.nl2) {
			ipt -= 8 * this.nside;
		}


		ret.ix = ((ipt - irt) >> 1);
		ret.iy = ((-(ipt + irt)) >> 1);

		return ret;
	};

	HealpixIndex.utils = Utils;

	return HealpixIndex;
})();


/**
 * The SpatialVector contains standard 3D vector with the addition that each
 * coordinate (x,y,z) is also kept in ra,dec since we expect the vector to live
 * on the surface of the unit sphere, i.e.
 * 
 * <pre>
 *  2   2   2
 *  x + y + z  = 1
 * </pre>
 * 
 * This is not enforced, so you can specify a vector that has not unit length.
 * If you request the ra/dec of such a vector, it will be automatically
 * normalized to length 1 and you get the ra/dec of that vector (the
 * intersection of the vector's direction with the unit sphere.
 * 
 * This code comes originally from the HTM library of Peter Kunst during his
 * time at JHU.
 */


export let SpatialVector = (function () {

	/**
	 * Constructor from three coordinates
	 * 
	 * @param x
	 * @param y
	 * @param z
	 */
	function SpatialVector(x, y, z) {
		"use strict";
		this.x = x;
		this.y = y;
		this.z = z;
		this.ra_ = 0;
		this.dec_ = 0;
		this.okRaDec_ = false;
	}
	;
	SpatialVector.prototype.setXYZ = function (x, y, z) {
		this.x = x;
		this.y = y;
		this.z = z;
		this.okRaDec_ = false;
	};

	/**
	 * Returns the length of this vector.
	 * 
	 * @return the length of this vector
	 */
	SpatialVector.prototype.length = function () {
		"use strict";
		return Math.sqrt(this.lengthSquared());
	};

	/**
	 * Returns the squared length of this vector.
	 * 
	 * @return the squared length of this vector
	 */
	SpatialVector.prototype.lengthSquared = function () {
		"use strict";
		return this.x * this.x + this.y * this.y + this.z * this.z;
	};

	/**
	 * Normalized this vector
	 */
	SpatialVector.prototype.normalized = function () {
		"use strict";
		var d = this.length();
		// zero-div may occur.
		this.x /= d;
		this.y /= d;
		this.z /= d;
	};

	/**
	 * Sets the ra and dec angles in degrees
	 * 
	 * @param ra
	 *            right ascension angle in degrees
	 * @param dec
	 *            declination angle in degrees
	 * 
	 */
	SpatialVector.prototype.set = function (ra, dec) {
		"use strict";
		this.ra_ = ra;
		this.dec_ = dec;
		this.okRaDec_ = true;
		this.updateXYZ();
	};

	/**
	 * Returns the angle in radians between this vector and the vector
	 * parameter; the return value is constrained to the range [0,PI].
	 * 
	 * @param v1
	 *            the other vector
	 * @return the angle in radians in the range [0,PI]
	 */
	SpatialVector.prototype.angle = function (v1) {
		"use strict";
		// return (double)Math.acos(dot(v1)/v1.length()/v.length());
		// Numerically, near 0 and PI are very bad condition for acos.
		// In 3-space, |atan2(sin,cos)| is much stable.
		var xx = this.y * v1.z - this.z * v1.y;
		var yy = this.z * v1.x - this.x * v1.z;
		var zz = this.x * v1.y - this.y * v1.x;
		var cross = Math.sqrt(xx * xx + yy * yy + zz * zz);
		return Math.abs(Math.atan2(cross, dot(v1)));
	};

	/**
	 * Get the coordinates in a 3 elements 1D array
	 * 
	 * @return coordinates [x,y,z]
	 */
	SpatialVector.prototype.get = function () {
		"use strict";
		return [x, y, z];
	};

	SpatialVector.prototype.toString = function () {
		"use strict";
		return "SpatialVector[" + this.x + ", " + this.y + ", " + this.z + "]";
	};

	/**
	 * vector cross product
	 * 
	 * @param v
	 *            the vector to cross
	 * @return the vector cross product
	 */
	SpatialVector.prototype.cross = function (v) {
		"use strict";
		return new SpatialVector(this.y * v.z - v.y * this.z, this.z * v.x - v.z * this.x, this.x * v.y - v.x() * this.y);
	};

	/**
	 * Compare vectors if coordinates are equals
	 * 
	 * @param v
	 *            the vector to be compared with
	 * @return true if both coordinates of vectors are equal
	 */
	SpatialVector.prototype.equal = function (v) {
		"use strict";
		return ((this.x == v.x && this.y == v.y && this.z == v.z()) ? true : false);
	};


	/**
	 * multiply with a number
	 * 
	 * @param n
	 *            the scale number to be multiply to the coordinates x,y,z
	 * @return the vector with coordinates multiplied by n
	 */
	SpatialVector.prototype.mult = function (n) {
		"use strict";
		return new SpatialVector((n * this.x), (n * this.y), (n * this.z));
	};

	/**
	 * Computes the dot product of the this vector and vector v1.
	 * 
	 * @param v1
	 *            the other vector
	 * @return dot product
	 */
	SpatialVector.prototype.dot = function (v1) {
		"use strict";
		return this.x * v1.x + this.y * v1.y + this.z * v1.z;
	};

	/**
	 * vector addition
	 * 
	 * @param v
	 *            the vector to be added
	 * @return vector result by addition
	 */
	SpatialVector.prototype.add = function (v) {
		"use strict";
		return new SpatialVector(this.x + v.x, this.y + v.y, this.z + v.z);
	};

	/**
	 * vector subtraction
	 * 
	 * @param v
	 *            the vector to be substracted
	 * @return vector result by substraction
	 */
	SpatialVector.prototype.sub = function (v) {
		"use strict";
		return new SpatialVector(this.x - v.x, this.y - v.y, this.z - v.z);
	};

	/**
	 * Get the dec angle in degrees
	 * 
	 * @return declination angle
	 */
	SpatialVector.prototype.dec = function () {
		"use strict";
		if (!this.okRaDec_) {
			this.normalized();
			this.updateRaDec();
		}
		return this.dec_;
	};

	/**
	 * Get the ra angle in degrees
	 * 
	 * @return right ascension
	 */
	SpatialVector.prototype.ra = function () {
		"use strict";
		if (!this.okRaDec_) {
			this.normalized();
			this.updateRaDec();
		}
		return this.ra_;
	};

	/**
	 * Update x_ y_ z_ from ra_ and dec_ variables
	 */
	SpatialVector.prototype.updateXYZ = function () {
		"use strict";
		var cd = Math.cos(this.dec_ * Constants.C_PR);
		this.x = Math.cos(this.ra_ * Constants.C_PR) * cd;
		this.y = Math.sin(this.ra_ * Constants.C_PR) * cd;
		this.z = Math.sin(this.dec_ * Constants.C_PR);
	};

	/**
	 * Update ra_ and dec_ from x_ y_ z_ variables
	 */
	SpatialVector.prototype.updateRaDec = function () {
		"use strict";
		this.dec_ = Math.asin(this.z) / Constants.C_PR; // easy.
		var cd = Math.cos(this.dec_ * Constants.C_PR);
		if (cd > Constants.EPS || cd < -Constants.EPS) {
			if (this.y > Constants.EPS || this.y < -Constants.EPS) {
				if (this.y < 0.0) {
					this.ra_ = 360 - Math.acos(this.x / cd) / Constants.C_PR;
				}
				else {
					this.ra_ = Math.acos(this.x / cd) / Constants.C_PR;
				}
			} else {
				this.ra_ = (this.x < 0.0 ? 180 : 0.0);
			}
		}
		else {
			this.ra_ = 0.0;
		}
		this.okRaDec_ = true;
	};

	/**
	 * @return Right Ascension of this vector in radians
	 */
	SpatialVector.prototype.toRaRadians = function () {
		"use strict";
		var phi = 0.;
		if ((this.x != 0.) || (this.y != 0)) {
			phi = Math.atan2(this.y, this.x); // phi in [-pi,pi]
		}

		if (phi < 0) {
			phi += 2.0 * Math.PI; // phi in [0, 2pi]
		}

		return phi;
	};

	/**
	 * @return Declination of this vector in radians
	 */
	SpatialVector.prototype.toDeRadians = function () {
		var z2 = z / this.length();
		var theta = Math.acos(z2);
		return Math.PI / 2 - theta;
	};

	return SpatialVector;
})();