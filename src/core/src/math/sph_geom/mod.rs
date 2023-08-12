pub mod bbox;
pub mod region;
pub mod great_circle_arc;

use super::{PI, TWICE_PI};

#[inline]
pub fn is_in_lon_range(lon0: f64, lon1: f64, lon2: f64) -> bool {
    // First version of the code: 
    //   ((v2.lon() - v1.lon()).abs() > PI) != ((v2.lon() > coo.lon()) != (v1.lon() > coo.lon()))
    // 
    // Lets note 
    //   - lonA = v1.lon()
    //   - lonB = v2.lon()
    //   - lon0 = coo.lon()
    // When (lonB - lonA).abs() <= PI 
    //   => lonB > lon0 != lonA > lon0  like in PNPOLY
    //   A    B    lonA <= lon0 && lon0 < lonB
    // --[++++[--
    //   B    A    lonB <= lon0 && lon0 < lonA
    //
    // But when (lonB - lonA).abs() > PI, then the test should be 
    //  =>   lonA >= lon0 == lonB >= lon0 
    // <=> !(lonA >= lon0 != lonB >= lon0)
    //    A  |  B    (lon0 < lonB) || (lonA <= lon0)
    //  --[++|++[--
    //    B  |  A    (lon0 < lonA) || (lonB <= lon0)
    //
    // Instead of lonA > lon0 == lonB > lon0,
    //     i.e. !(lonA > lon0 != lonB > lon0).
    //    A  |  B    (lon0 <= lonB) || (lonA < lon0)
    //  --]++|++]--
    //    B  |  A    (lon0 <= lonA) || (lonB < lon0)
    //
    // So the previous code was bugged in this very specific case: 
    // - `lon0` has the same value as a vertex being part of:
    //   - one segment that do not cross RA=0
    //   - plus one segment crossing RA=0.
    //   - the point have an odd number of intersections with the polygon 
    //     (since it will be counted 0 or 2 times instead of 1).
    let dlon = lon2 - lon1;
    if dlon < 0.0 {
        (dlon >= -PI) == (lon2 <= lon0 && lon0 < lon1)
    } else {
        (dlon <=  PI) == (lon1 <= lon0 && lon0 < lon2)
    }
}

// Returns the longitude length between two longitudes
// lon1 lies in [0; 2\pi[
// lon2 lies in [0; 2\pi[
#[inline]
pub fn distance_from_two_lon(lon1: f64, lon2: f64) -> f64 {
    // Cross the primary meridian
    if lon1 > lon2 {
        lon2 + TWICE_PI - lon1
    } else {
        lon2 - lon1
    }
}

