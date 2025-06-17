vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}