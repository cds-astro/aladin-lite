"use strict";
module.exports = class SpatialVector {
    constructor(t, s, i) {
        "use strict";
        if (t) {
        (this.x = t), (this.y = s), (this.z = i), (this.ra_ = 0), (this.dec_ = 0), (this.okRaDec_ = !1);
    }
    }
    
        setXYZ(t, s, i) {
            (this.x = t), (this.y = s), (this.z = i), (this.okRaDec_ = !1);
        }
        
        length() {
            "use strict";
            return Math.sqrt(this.lengthSquared());
        }
        
lengthSquared() {
            "use strict";
            return this.x * this.x + this.y * this.y + this.z * this.z;
        }
        
normalized() {
            "use strict";
            var t = this.length();
            (this.x /= t), (this.y /= t), (this.z /= t);
        }
        
set(t, s) {
            "use strict";
            (this.ra_ = t), (this.dec_ = s), (this.okRaDec_ = !0), this.updateXYZ();
        }
        
angle(t) {
            "use strict";
            var s = this.y * t.z - this.z * t.y,
                i = this.z * t.x - this.x * t.z,
                n = this.x * t.y - this.y * t.x,
                a = Math.sqrt(s * s + i * i + n * n);
            return Math.abs(Math.atan2(a, dot(t)));
        }
        
get() {
            "use strict";
            return [x, y, z];
        }
        
toString() {
            "use strict";
            return "SpatialVector[" + this.x + ", " + this.y + ", " + this.z + "]";
        }
        
cross(s) {
            "use strict";
            return new SpatialVector(this.y * s.z - s.y * this.z, this.z * s.x - s.z * this.x, this.x * s.y - s.x() * this.y);
        }
        
equal(t) {
            "use strict";
            return this.x == t.x && this.y == t.y && this.z == t.z() ? !0 : !1;
        }
        
mult(s) {
            "use strict";
            return new SpatialVector(s * this.x, s * this.y, s * this.z);
        }
        
dot(t) {
            "use strict";
            return this.x * t.x + this.y * t.y + this.z * t.z;
        }
        
add(s) {
            "use strict";
            return new SpatialVector(this.x + s.x, this.y + s.y, this.z + s.z);
        }
        
sub(s) {
            "use strict";
            return new SpatialVector(this.x - s.x, this.y - s.y, this.z - s.z);
        }
        
dec() {
            "use strict";
            return this.okRaDec_ || (this.normalized(), this.updateRaDec()), this.dec_;
        }
        
ra() {
            "use strict";
            return this.okRaDec_ || (this.normalized(), this.updateRaDec()), this.ra_;
        }
        
updateXYZ() {
            "use strict";
            var t = Math.cos(this.dec_ * Constants.C_PR);
            (this.x = Math.cos(this.ra_ * Constants.C_PR) * t), (this.y = Math.sin(this.ra_ * Constants.C_PR) * t), (this.z = Math.sin(this.dec_ * Constants.C_PR));
        }
        
updateRaDec() {
            "use strict";
            this.dec_ = Math.asin(this.z) / Constants.C_PR;
            var t = Math.cos(this.dec_ * Constants.C_PR);
            (this.ra_ =
                t > Constants.EPS || -Constants.EPS > t
                    ? this.y > Constants.EPS || this.y < -Constants.EPS
                        ? 0 > this.y
                            ? 360 - Math.acos(this.x / t) / Constants.C_PR
                            : Math.acos(this.x / t) / Constants.C_PR
                        : 0 > this.x
                        ? 180
                        : 0
                    : 0),
                (this.okRaDec_ = !0);
        }
        
toRaRadians() {
            "use strict";
            var t = 0;
            return (0 != this.x || 0 != this.y) && (t = Math.atan2(this.y, this.x)), 0 > t && (t += 2 * Math.PI), t;
        }
        
toDeRadians() {
            var t = z / this.length(),
                s = Math.acos(t);
            return Math.PI / 2 - s;
        }
        
}