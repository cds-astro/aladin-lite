const PI2 = 2 * Math.PI;
const PI = Math.PI;
const PI_2 = Math.PI / 2;
const PI_4 = Math.PI / 4;
const PI_8 = Math.PI / 8;

HealpixIndex = (function() {

    function HealpixIndex(t) {
    "use strict";
    this.Nside = t;
    this.NS_MAX = 8388608;
        this.ORDER_MAX = 23;
        console.log('order '+this.ORDER_MAX);
    }
    
        HealpixIndex.prototype.init = function() {
            "use strict";
            
        };
        
    HealpixIndex.prototype.order2nside = function(order) {
        return 1 << order;
    };

    HealpixIndex.prototype.nside2order = function(nside) {
        return ilog2(nside);
    };

    HealpixIndex.prototype.nside2npix = function(nside) {
        return 12 * nside * nside;
    };

    HealpixIndex.prototype.vec2pix_nest = function(nside, v) {
        const { z, a } = vec2za(v[0], v[1], v[2]);
        return za2pix_nest(nside, z, a);
    };

    HealpixIndex.prototype.vec2pix_ring = function(nside, v) {
        const { z, a } = vec2za(v[0], v[1], v[2]);
        return nest2ring(nside, za2pix_nest(nside, z, a));
    };

    HealpixIndex.prototype.ang2pix_nest = function(nside, theta, phi) {
        const z = Math.cos(theta);
        return za2pix_nest(nside, z, phi);
    };

    HealpixIndex.prototype.ang2pix_ring = function(nside, theta, phi) {
        const z = Math.cos(theta);
        return nest2ring(nside, za2pix_nest(nside, z, phi));
    };

    HealpixIndex.prototype.nest2ring = function(nside, ipix) {
        const { f, x, y } = nest2fxy(nside, ipix);
        return fxy2ring(nside, f, x, y);
    };

    HealpixIndex.prototype.ring2nest = function(nside, ipix) {
        if (nside == 1) {
            return ipix;
        }
        const { f, x, y } = ring2fxy(nside, ipix);
        return fxy2nest(nside, f, x, y);
    };

    HealpixIndex.prototype.ring2fxy = function(nside, ipix) {
        const polar_lim = 2 * nside * (nside - 1);
        if (ipix < polar_lim) { // north polar cap
            var i = Math.floor((Math.sqrt(1 + 2 * ipix) + 1) / 2);
            var j = ipix - 2 * i * (i - 1);
            var f = Math.floor(j / i);
            var k = j % i;
            var x = nside - i + k;
            const y = nside - 1n - k;
            return { f, x, y };
        }
        if (ipix < polar_lim + 8 * nside * nside) { // equatorial belt
            const k = ipix - polar_lim;
            const ring = 4n * nside;
            const i = nside - k / ring;
            const s = i % 2n == 0 ? 1n : 0n;
            const j = 2n * (k % ring) + s;
            const jj = j - 4n * nside;
            const ii = i + 5n * nside - 1n;
            const pp = (ii + jj) / 2n;
            const qq = (ii - jj) / 2n;
            const PP = pp / nside;
            const QQ = qq / nside;
            const V = 5n - (PP + QQ);
            const H = PP - QQ + 4n;
            const f = 4n * V + (H >> 1n) % 4n;
            const x = pp % nside;
            const y = qq % nside;
            return { f, x, y };
        }
        else { // south polar cap
            const p = 12n * nside * nside - ipix - 1n;
            const i = (Math.sqrt(1n + 2n * p) + 1n) / 2n;
            const j = p - 2n * i * (i - 1n);
            const f = 11n - j / i;
            const k = j % i;
            const x = i - k - 1n;
            const y = k;
            return { f, x, y };
        }
    };
    
    HealpixIndex.prototype.pix2vec_nest = function(nside, ipix) {
        const { f, x, y } = nest2fxy(nside, ipix);
        const { t, u } = fxy2tu(nside, f, x, y);
        const { z, a } = tu2za(t, u);
        return za2vec(z, a);
    };

    HealpixIndex.prototype.pix2ang_nest = function(nside, ipix) {
        const { f, x, y } = nest2fxy(nside, ipix);
        const { t, u } = fxy2tu(nside, f, x, y);
        const { z, a } = tu2za(t, u);
        return { theta: Math.acos(z), phi: a };
    };

    HealpixIndex.prototype.pix2vec_ring = function(nside, ipix) {
        return pix2vec_nest(nside, ring2nest(nside, ipix));
    };

    HealpixIndex.prototype.pix2ang_ring = function(nside, ipix) {
        return pix2ang_nest(nside, ring2nest(nside, ipix));
    };

    // TODO: cleanup
    HealpixIndex.prototype.query_disc_inclusive_nest = function(nside, v, radius, cb) {
        if (radius > PI_2) {
            throw new Error(`query_disc: radius must < PI/2`);
        }
        const pixrad = max_pixrad(nside);
        const d = PI_4 / nside;
        const { z: z0, a: a0 } = vec2za(v[0], v[1], v[2]); // z0 = cos(theta)
        const sin_t = Math.sqrt(1n - z0 * z0);
        const cos_r = Math.cos(radius); // r := radius 
        const sin_r = Math.sin(radius);
        const z1 = z0 * cos_r + sin_t * sin_r; // cos(theta - r)
        const z2 = z0 * cos_r - sin_t * sin_r; // cos(theta + r)
        const u1 = za2tu(z1, 0).u;
        const u2 = za2tu(z2, 0).u;
        const cover_north_pole = sin_t * cos_r - z0 * sin_r < 0; // sin(theta - r) < 0
        const cover_south_pole = sin_t * cos_r + z0 * sin_r < 0; // sin(theta - r) < 0
        let i1 = Math.floor((PI_2 - u1) / d);
        let i2 = Math.floor((PI_2 - u2) / d + 1);
        if (cover_north_pole) {
            ++i1;
            for (let i = 1; i <= i1; ++i)
                walk_ring(nside, i, cb);
            ++i1;
        }
        if (i1 == 0) {
            walk_ring(nside, 1, cb);
            i1 = 2;
        }
        if (cover_south_pole) {
            --i2;
            for (let i = i2; i <= 4 * nside - 1; ++i)
                walk_ring(nside, i, cb);
            --i2;
        }
        if (i2 == 4n * nside) {
            walk_ring(nside, 4n * nside - 1, cb);
            i2 = 4n * nside - 2n;
        }
        const theta = Math.acos(z0);
        for (let i = i1; i <= i2; ++i)
            walk_ring_around(nside, i, a0, theta, radius + pixrad, function(ipix) {
                if (angle(pix2vec_nest(nside, ipix), v) <= radius + pixrad)
                    cb(ipix);
            });
    };

    HealpixIndex.prototype.query_disc_inclusive_ring = function(nside, v, radius, cb_ring) {
        return query_disc_inclusive_nest(nside, v, radius, function(ipix) {
            cb_ring(nest2ring(nside, ipix));
        });
    };

    HealpixIndex.prototype.max_pixrad = function(nside) {
        const unit = PI_4 / nside;
        return angle(
            tu2vec(unit, nside * unit),
            tu2vec(unit, (nside + 1n) * unit),
        );
    };

    function angle(a, b) {
        return 2 * Math.asin(Math.sqrt(distance2(a, b)) / 2);
    };

    function tu2vec(t, u) {
        const { z, a } = tu2za(t, u);
        return za2vec(z, a);
    };

    function distance2(a, b) {
        const dx = a[0] - b[0];
        const dy = a[1] - b[1];
        const dz = a[2] - b[2];
        return dx * dx + dy * dy  + dz * dz;
    };

    function walk_ring_around(nside, i, a0, theta, r, cb) {
        if (theta < r || theta + r > PI)
            return walk_ring(nside, i, cb);
        const u = PI_4 * (2n - i / nside);
        const z = tu2za(PI_4, u).z;
        const st = Math.sin(theta);
        const ct = Math.cos(theta);
        const sr = Math.sin(r);
        const cr = Math.cos(r);
        const w = Math.atan2(
            Math.sqrt(-square(z - ct * cr) / (square(st) * sr * sr) + 1) * sr,
            (-z * ct + cr) / st
        );
        if (w >= PI)
            return walk_ring(nside, i, cb);
        const t1 = center_t(nside, i, za2tu(z, wrap(a0 - w, PI2)).t);
        const t2 = center_t(nside, i, za2tu(z, wrap(a0 + w, PI2)).t);
        const begin = tu2fxy(nside, t1, u);
        const end = right_next_pixel(nside, tu2fxy(nside, t2, u));
        for (let s = begin; !fxy_compare(s, end); s = right_next_pixel(nside, s)) {
            cb(fxy2nest(nside, s.f, s.x, s.y));
        }
    };

    function center_t(nside, i, t) {
        var d = PI_4 / nside;
        t /= d;
        t = (((t + i % 2) >> 1) << 1) + 1 - i % 2;
        t *= d;
        return t;
    };

    function walk_ring(nside, i, cb) {
        const u = PI_4 * (2 - i / nside);
        const t = PI_4 * (1 + (1 - i % 2) / nside);
        const begin = tu2fxy(nside, t, u);
        let s = begin;
        do {
            cb(fxy2nest(nside, s.f, s.x, s.y));
            s = right_next_pixel(nside, s);
        } while (!fxy_compare(s, begin))
    };

    function fxy_compare(a, b) {
        return a.x == b.x && a.y == b.y && a.f == b.f;
    };

    function right_next_pixel(nside, { f, x, y}) {
        ++x;
        if (x == nside) {
            switch (Math.floor(f / 4)) {
                case 0:
                    f = (f + 1) % 4;
                    x = y;
                    y = nside;
                    break;
                case 1:
                    f = f - 4;
                    x = 0;
                    break;
                case 2:
                    f = 4 + (f + 1) % 4;
                    x = 0;
                    break;
            }
        }
        --y;
        if (y == -1) {
            switch (f / 4) {
                case 0:
                    f = 4 + (f + 1) % 4;
                    y = nside - 1;
                    break
                case 1:
                    f = f + 4;
                    y = nside - 1;
                    break;
                case 2: {
                    f = 8 + (f + 1) % 4;
                    y = x - 1;
                    x = 0;
                    break;
                }
            }
        }
        return { f, x, y };
    };

    HealpixIndex.prototype.corners_nest = function(nside, ipix) {
        const { f, x, y } = nest2fxy(nside, ipix);
        const { t, u } = fxy2tu(nside, f, x, y);
        const d = PI_4 / nside;
        var xyzs = [];
        for (const [tt, uu] of [
            [0, d],
            [-d, 0],
            [0, -d],
            [d, 0],
        ]) {
            const { z, a } = tu2za(t + tt, u + uu);
            xyzs.push(za2vec(z, a));
        }
        return xyzs;
    };
    
    HealpixIndex.prototype.corners_ring = function(nside, ipix) {
        return corners_nest(nside, ring2nest(nside, ipix));
    };

    // pixel area
    HealpixIndex.prototype.nside2pixarea = function(nside) {
        return PI / (3 * nside * nside); //$$
    };

    // average pixel size
    HealpixIndex.prototype.nside2resol = function(nside) {
        return Math.sqrt(PI / 3) / nside;
    };

    HealpixIndex.prototype.pixcoord2vec_nest = function(nside, ipix, ne, nw) {
        const { f, x, y } = nest2fxy(nside, ipix);
        const { t, u } = fxy2tu(nside, f, x, y);
        const d = PI_4 / nside;
        const { z, a } = tu2za(t + d * (ne - nw), u + d * (ne + nw - 1));
        return za2vec(z, a);
    };

    HealpixIndex.prototype.pixcoord2vec_ring = function(nside, ipix, ne, nw) {
        return pixcoord2vec_nest(nside, ring2nest(nside, ipix), ne, nw);
    }

    function za2pix_nest(nside, z, a) {
        const { t, u } = za2tu(z, a);
        const { f, x, y } = tu2fxy(nside, t, u);
        return fxy2nest(nside, f, x, y);
    };

    HealpixIndex.prototype.tu2fxy = function(nside, t, u) {
        const { f, p, q } = tu2fpq(t, u);
        const x = clip(nside * p, 0, nside - 1);
        const y = clip(nside * q, 0, nside - 1);
        return { f, x, y };
    };

    function wrap(A, B) {
        return A < 0 ? B - (-A % B) : A % B;
    };


    function sigma(z) {
        if (z < 0)
            return -sigma(-z);
        else
            return 2 - Math.sqrt(3 * (1 - z));
    };

    /**
     * HEALPix spherical projection.
     */
    HealpixIndex.prototype.za2tu = function(z, a) {
        if (Math.abs(z) <= 2 / 3) { // equatorial belt
            const t = a;
            const u = 3 * PI_8 * z;
            return { t, u };
        }
        else { // polar caps
            const p_t = a % (PI_2);
            const sigma_z = sigma(z);
            const t = a - (Math.abs(sigma_z) - 1) * (p_t - PI_4);
            const u = PI_4 * sigma_z;
            return { t, u };
        }
    };

    /**
     * Inverse HEALPix spherical projection.
     */
    HealpixIndex.prototype.tu2za = function(t, u) {
        const abs_u = Math.abs(u);
        if (abs_u >= PI_2) { // error
            return { z: sign(u), a: 0 };
        }
        if (abs_u <= Math.PI / 4) { // equatorial belt
            const z = 8 / (3 * PI) * u;
            const a = t;
            return { z, a };
        }
        else { // polar caps
            const t_t = t % (Math.PI / 2);
            const a = t - (abs_u - PI_4) / (abs_u - PI_2) * (t_t - PI_4);
            const z = sign(u) * (1 - 1 / 3 * square(2 - 4 * abs_u / PI));
            return { z, a };
        }
    };

    // (x, y, z) -> (z = cos(theta), phi)
    function vec2za(X, Y, z) {
        const r2 = X * X + Y * Y;
        if (r2 == 0)
            return { z: z < 0 ? -1 : 1, a: 0 };
        else {
            const a = (Math.atan2(Y, X) + PI2) % PI2;
            z /= Math.sqrt(z * z + r2);
            return { z, a };
        }
    };

    // (z = cos(theta), phi) -> (x, y, z)
    function za2vec(z, a) {
        const sin_theta = Math.sqrt(1 - z * z);
        const X = sin_theta * Math.cos(a);
        const Y = sin_theta * Math.sin(a);
        return [X, Y, z];
    };


    HealpixIndex.prototype.ang2vec = function(theta, phi) {
        const z = Math.cos(theta);
        return za2vec(z, phi);
    };

    HealpixIndex.prototype.vec2ang = function(v) {
        const { z, a } = vec2za(v[0], v[1], v[2]);
        return { theta: Math.acos(z), phi: a };
    };

    // spherical projection -> f, p, q
    // f: base pixel index
    // p: coord in north east axis of base pixel
    // q: coord in north west axis of base pixel
    function tu2fpq(t, u) {
        t /= PI_4;
        u /= PI_4;
        t = wrap(t, 8);
        t += -4;
        u += 5;
        const pp = clip((u + t) / 2, 0, 5);
        const PP = Math.floor(pp);
        const qq = clip((u - t) / 2, 3 - PP, 6 - PP);
        const QQ = Math.floor(qq);
        const V = 5 - (PP + QQ);
        if (V < 0) { // clip
            return { f: 0, p: 1, q: 1 };
        }
        const H = PP - QQ + 4;
        const f = 4 * V + (H >> 1) % 4;
        const p = pp % 1;
        const q = qq % 1;
        return { f, p, q };
    };

    // f, p, q -> nest index
    HealpixIndex.prototype.fxy2nest = function(nside, f, x, y) {
        return f * nside * nside + bit_combine(x, y);
    };

    // x = (...x2 x1 x0)_2 <- in binary
    // y = (...y2 y1 y0)_2
    // p = (...y2 x2 y1 x1 y0 x0)_2
    // returns p
/* Python for bit manipulation
n = 25
s = ' | '.join(['x & 1'] + [f'(x & BigInt(0x{2 ** (i+1):x}) | y & BigInt(0x{2 ** i:x})) << {i + 1}n' for i in range(n)] + [f'y & BigInt(0x{2**n:x}) << {n+1}n'])
*/
    HealpixIndex.prototype.bit_combine = function(x, y) {
        assert(x < (1 << 26n));
        assert(y < (1 << 25n));
        return (
            x & 1 | (x & BigInt(0x2) | y & BigInt(0x1)) << 1n | (x & BigInt(0x4) | y & BigInt(0x2)) << 2n | (x & BigInt(0x8) | y & BigInt(0x4)) << 3n | (x & BigInt(0x10) | y & BigInt(0x8)) << 4n | (x & BigInt(0x20) | y & BigInt(0x10)) << 5n | (x & BigInt(0x40) | y & BigInt(0x20)) << 6n | (x & BigInt(0x80) | y & BigInt(0x40)) << 7n | (x & BigInt(0x100) | y & BigInt(0x80)) << 8n | (x & BigInt(0x200) | y & BigInt(0x100)) << 9n | (x & BigInt(0x400) | y & BigInt(0x200)) << 10n | (x & BigInt(0x800) | y & BigInt(0x400)) << 11n | (x & BigInt(0x1000) | y & BigInt(0x800)) << 12n | (x & BigInt(0x2000) | y & BigInt(0x1000)) << 13n | (x & BigInt(0x4000) | y & BigInt(0x2000)) << 14n | (x & BigInt(0x8000) | y & BigInt(0x4000)) << 15n | (x & BigInt(0x10000) | y & BigInt(0x8000)) << 16n | (x & BigInt(0x20000) | y & BigInt(0x10000)) << 17n | (x & BigInt(0x40000) | y & BigInt(0x20000)) << 18n | (x & BigInt(0x80000) | y & BigInt(0x40000)) << 19n | (x & BigInt(0x100000) | y & BigInt(0x80000)) << 20n | (x & BigInt(0x200000) | y & BigInt(0x100000)) << 21n | (x & BigInt(0x400000) | y & BigInt(0x200000)) << 22n | (x & BigInt(0x800000) | y & BigInt(0x400000)) << 23n | (x & BigInt(0x1000000) | y & BigInt(0x800000)) << 24n | (x & BigInt(0x2000000) | y & BigInt(0x1000000)) << 25n | y & BigInt(0x2000000) << 26n
        );
    };

    // x = (...x2 x1 x0)_2 <- in binary
    // y = (...y2 y1 y0)_2
    // p = (...y2 x2 y1 x1 y0 x0)_2
    // returns x, y
    HealpixIndex.prototype.bit_decombine = function(p) {
        assert(p <= 0x1fffffffffffff);
        // (python)
        // ' | '.join(f'(p & BigInt(0x{2**(2*i):x})) >> {i}n' for i in range(26))
const x = ((p & BigInt(0x1)) >> 0n | (p & BigInt(0x4)) >> 1n | (p & BigInt(0x10)) >> 2n | (p & BigInt(0x40)) >> 3n | (p & BigInt(0x100)) >> 4n | (p & BigInt(0x400)) >> 5n | (p & BigInt(0x1000)) >> 6n | (p & BigInt(0x4000)) >> 7n | (p & BigInt(0x10000)) >> 8n | (p & BigInt(0x40000)) >> 9n | (p & BigInt(0x100000)) >> 10n | (p & BigInt(0x400000)) >> 11n | (p & BigInt(0x1000000)) >> 12n | (p & BigInt(0x4000000)) >> 13n | (p & BigInt(0x10000000)) >> 14n | (p & BigInt(0x40000000)) >> 15n | (p & BigInt(0x100000000)) >> 16n | (p & BigInt(0x400000000)) >> 17n | (p & BigInt(0x1000000000)) >> 18n | (p & BigInt(0x4000000000)) >> 19n | (p & BigInt(0x10000000000)) >> 20n | (p & BigInt(0x40000000000)) >> 21n | (p & BigInt(0x100000000000)) >> 22n | (p & BigInt(0x400000000000)) >> 23n | (p & BigInt(0x1000000000000)) >> 24n | (p & BigInt(0x4000000000000)) >> 25n);
                // (python)
        // ' | '.join(f'(p & BigInt(0x{2**(2*i + 1):x})) >> {i+1}n' for i in range(25))
const y = ((p & BigInt(0x2)) >> 1n | (p & BigInt(0x8)) >> 2n | (p & BigInt(0x20)) >> 3n | (p & BigInt(0x80)) >> 4n | (p & BigInt(0x200)) >> 5n | (p & BigInt(0x800)) >> 6n | (p & BigInt(0x2000)) >> 7n | (p & BigInt(0x8000)) >> 8n | (p & BigInt(0x20000)) >> 9n | (p & BigInt(0x80000)) >> 10n | (p & BigInt(0x200000)) >> 11n | (p & BigInt(0x800000)) >> 12n | (p & BigInt(0x2000000)) >> 13n | (p & BigInt(0x8000000)) >> 14n | (p & BigInt(0x20000000)) >> 15n | (p & BigInt(0x80000000)) >> 16n | (p & BigInt(0x200000000)) >> 17n | (p & BigInt(0x800000000)) >> 18n | (p & BigInt(0x2000000000)) >> 19n | (p & BigInt(0x8000000000)) >> 20n | (p & BigInt(0x20000000000)) >> 21n | (p & BigInt(0x80000000000)) >> 22n | (p & BigInt(0x200000000000)) >> 23n | (p & BigInt(0x800000000000)) >> 24n | (p & BigInt(0x2000000000000)) >> 25n);
        return { x, y };
    };

    // f: base pixel index
    // x: north east index in base pixel
    // y: north west index in base pixel
    function nest2fxy(nside, ipix) {
        const nside2 = nside * nside;
        const f = ipix / nside2; // base pixel index
        const k = ipix % nside2;             // nested pixel index in base pixel
        const { x, y } = bit_decombine(k);
        return { f, x, y };
    };

    function fxy2ring(nside, f, x, y) {
        const f_row = f / 4n; // {0 .. 2}
        const f1 = f_row + 2n;            // {2 .. 4}
        const v = x + y;
        const i = f1 * nside - v - 1n;

        if (i < nside) { // north polar cap
            const f_col = f % 4n;
            const ipix = 2n * i * (i - 1n) + (i * f_col) + nside - y - 1n;
            return ipix
        }
        if (i < 3n * nside) { // equatorial belt
            const h = x - y;
            const f2 = 2n * (f % 4n) - (f_row % 2n) + 1n;  // {0 .. 7}
            const k = (f2 * nside + h + (8n * nside)) % (8n * nside);
            const offset = 2n * nside * (nside - 1n);
            const ipix = offset + (i - nside) * 4n * nside + (k >> 1n);
            return ipix;
        }
        else { // south polar cap
            const i_i = 4n * nside - i
            const i_f_col = 3n - (f % 4n)
            const j = 4n * i_i - (i_i * i_f_col) - y
            const i_j = 4n * i_i - j + 1n
            const ipix = 12n * nside * nside - 2n * i_i * (i_i - 1n) - i_j;
            return ipix;
        }
    };

    // f, x, y -> spherical projection
    HealpixIndex.prototype.fxy2tu = function(nside, f, x, y) {
        const f_row = f / 4n;
        const f1 = f_row + 2n;
        const f2 = 2n * (f % 4n) - (f_row % 2n) + 1n;
        const v = x + y;
        const h = x - y;
        const i = f1 * nside - v - 1n;
        const k = (f2 * nside + h + (8n * nside));
        const t = k / nside * PI_4;
        const u = PI_2 - i / nside * PI_4;
        return { t, u };
    };

    HealpixIndex.prototype.orderpix2uniq = function(order, ipix) {
        /**
         * Pack `(order, ipix)` into a `uniq` integer.
         * 
         * This HEALPix "unique identifier scheme" is starting to be used widely:
         * - see section 3.2 in http://healpix.sourceforge.net/pdf/intro.pdf
         * - see section 2.3.1 in http://ivoa.net/documents/MOC/
         */
        return 4n * ((1n << (2n * order)) - 1n) + ipix;
    };

    HealpixIndex.prototype.uniq2orderpix = function(uniq) {
        /**
         * Unpack `uniq` integer into `(order, ipix)`.
         * 
         * Inverse of `orderpix2uniq`.
         */
        assert(uniq <= 0x1fffffffffffff);
        let order = 0n;
        let l = (uniq >> 2n) + 1n;
        while (l >= 4n) {
            l >>= 2n;
            ++order;
        }
        const ipix = uniq - (((1n << (2n * order)) - 1n) << 2n);
        return { order, ipix };
    };

    function ilog2(x) {
    return x.toString(2).length - 1;
    };

    var sign = Math.sign || function (A) {
        return A > 0 ? 1 : (A < 0 ? -1 : 0);
    };

    function square(A) {
        return A * A;
    };

    function clip(Z, A, B) {
        return Z < A ? A : (Z > B ? B : Z);
    };

    function assert(condition) {
        console.assert(condition)
        if (!condition) {
            debugger;
        }
    };

    return HealpixIndex;
});
