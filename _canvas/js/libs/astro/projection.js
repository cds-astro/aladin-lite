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
}