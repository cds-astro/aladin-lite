import sys
import argparse
import json
import random
import math
import healpy


random.seed(0)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--out', '-o', type=argparse.FileType('w'), default=sys.stdout)
    args = parser.parse_args()

    testcase = {}
    vec2pix_nest(testcase)
    vec2pix_ring(testcase)
    ang2pix_nest(testcase)
    ang2pix_ring(testcase)
    nest2ring(testcase)
    ring2nest(testcase)
    pix2vec_nest(testcase)
    pix2vec_ring(testcase)
    nside2pixarea(testcase)
    nside2resol(testcase)
    max_pixrad(testcase)
    corners_nest(testcase)
    corners_ring(testcase)
    orderpix2uniq(testcase)
    uniq2orderpix(testcase)
    nside2order(testcase)
    json.dump(testcase, args.out, indent=2)


def vec2pix_nest(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            v = random_vec()
            cs.append(dict(
                args=(nside, v),
                expected=healpy.vec2pix(nside, *v, nest=True).tolist()
            ))
    testcase['vec2pix_nest'] = cs


def vec2pix_ring(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            v = random_vec()
            cs.append(dict(
                args=(nside, v),
                expected=healpy.vec2pix(nside, *v).tolist()
            ))
    testcase['vec2pix_ring'] = cs


def ang2pix_nest(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            theta = random.uniform(0, math.pi)
            phi = random.uniform(0, 2 * math.pi)
            args = (nside, theta, phi)
            cs.append(dict(
                args=args,
                expected=healpy.ang2pix(*args, nest=True).tolist()
            ))
    testcase['ang2pix_nest'] = cs


def ang2pix_ring(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            theta = random.uniform(0, math.pi)
            phi = random.uniform(0, 2 * math.pi)
            args = (nside, theta, phi)
            cs.append(dict(
                args=args,
                expected=healpy.ang2pix(*args).tolist()
            ))
    testcase['ang2pix_ring'] = cs


def nest2ring(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.nest2ring(*args).tolist()
            ))
    testcase['nest2ring'] = cs


def pix2vec_nest(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.pix2vec(*args, nest=True)
            ))
    testcase['pix2vec_nest'] = cs


def pix2vec_ring(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.pix2vec(*args, nest=False)
            ))
    testcase['pix2vec_ring'] = cs


def ring2nest(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.ring2nest(*args).tolist()
            ))
    testcase['ring2nest'] = cs


def nside2pixarea(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        args = (nside,)
        cs.append(dict(
            args=args,
            expected=healpy.nside2pixarea(*args)
        ))
    testcase['nside2pixarea'] = cs


def nside2resol(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        args = (nside,)
        cs.append(dict(
            args=args,
            expected=healpy.nside2resol(*args)
        ))
    testcase['nside2resol'] = cs


def max_pixrad(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        args = (nside,)
        cs.append(dict(
            args=args,
            expected=healpy.max_pixrad(*args)
        ))
    testcase['max_pixrad'] = cs


def corners_nest(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.boundaries(*args, nest=True).T.tolist()
            ))
    testcase['corners_nest'] = cs


def corners_ring(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (nside, ipix)
            cs.append(dict(
                args=args,
                expected=healpy.boundaries(*args).T.tolist()
            ))
    testcase['corners_ring'] = cs


def orderpix2uniq_python(order, index):
    return 4 * ((1 << (2 * order)) - 1) + index


def orderpix2uniq(testcase):
    cs = []
    for norder in range(24):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            args = (norder, ipix)
            cs.append(dict(
                args=args,
                expected=orderpix2uniq_python(*args)
            ))
    testcase['orderpix2uniq'] = cs


def uniq2orderpix(testcase):
    cs = []
    for norder in range(14):
        nside = 1 << norder
        for i in range(1000):
            ipix = random.randrange(12 * nside * nside)
            uid = orderpix2uniq_python(norder, ipix)
            cs.append(dict(
                args=(uid,),
                expected={'order': norder, 'ipix': ipix}
            ))
    testcase['uniq2orderpix'] = cs


def nside2order(testcase):
    cs = []
    for order in range(24):
        nside = 1 << order
        cs.append(dict(
            args=(nside,),
            expected=order,
        ))
    testcase['nside2order'] = cs


def random_vec():
    return [
        random.uniform(-1, 1),
        random.uniform(-1, 1),
        random.uniform(-1, 1),
    ]


if __name__ == '__main__':
    main()
