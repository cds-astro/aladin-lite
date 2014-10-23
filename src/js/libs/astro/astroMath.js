//=================================
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
