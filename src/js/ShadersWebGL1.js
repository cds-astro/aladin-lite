/* Import all the shaders here*/ 
// Catalog shaders
import CatalogAitoffVS from '../glsl/webgl1/catalogs/aitoff.vert';
import CatalogMercatVS from '../glsl/webgl1/catalogs/mercator.vert';
import CatalogArcVS from '../glsl/webgl1/catalogs/arc.vert';
import CatalogTanVS from '../glsl/webgl1/catalogs/tan.vert';
import CatalogMollVS from '../glsl/webgl1/catalogs/mollweide.vert';
import CatalogOrthoVS from '../glsl/webgl1/catalogs/ortho.vert';
import CatalogOrthoFS from '../glsl/webgl1/catalogs/ortho.frag';
import CatalogFS from '../glsl/webgl1/catalogs/catalog.frag';

// Colormap shaders
import ColormapCatalogVS from '../glsl/webgl1/colormaps/colormap.vert'
import ColormapCatalogFS from '../glsl/webgl1/colormaps/colormap.frag'

// Grid shader
import GridVS_CPU from '../glsl/webgl1/grid/grid_cpu.vert'
import GridFS_CPU from '../glsl/webgl1/grid/grid_cpu.frag'

// HiPS shaders
// Raytracer
import RayTracerVS from '../glsl/webgl1/hips/raytracer/raytracer.vert'
import RayTracerColorFS from '../glsl/webgl1/hips/raytracer/color.frag'
import RayTracerGrayscale2ColormapFS from '../glsl/webgl1/hips/raytracer/grayscale_to_colormap.frag'
// Rasterizer
import RasterizerVS from '../glsl/webgl1/hips/rasterizer/raster.vert'
import RasterizerColorFS from '../glsl/webgl1/hips/rasterizer/color.frag'
import RasterizerGrayscale2ColormapFS from '../glsl/webgl1/hips/rasterizer/grayscale_to_colormap.frag'

// Post
import PostVS from '../glsl/webgl1/passes/post_vertex_100es.glsl'
import PostFS from '../glsl/webgl1/passes/post_fragment_100es.glsl'

let shaders = [
    // Catalog shaders
    {
        id: "CatalogAitoffVS",
        content: CatalogAitoffVS,
    },
    {
        id: "CatalogMercatVS",
        content: CatalogMercatVS,
    },
    {
        id: "CatalogArcVS",
        content: CatalogArcVS,
    },
    {
        id: "CatalogTanVS",
        content: CatalogTanVS,
    },
    {
        id: "CatalogMollVS",
        content: CatalogMollVS,
    },
    {
        id: "CatalogOrthoVS",
        content: CatalogOrthoVS,
    },
    {
        id: "CatalogOrthoFS",
        content: CatalogOrthoFS,
    },
    {
        id: "CatalogFS",
        content: CatalogFS,    
    },
    // Colormap shaders
    {
        id: "ColormapCatalogVS",
        content: ColormapCatalogVS,
    },
    {
        id: "ColormapCatalogFS",
        content: ColormapCatalogFS,
    },
    // Grid shader
    {
        id: "GridFS_CPU",
        content: GridFS_CPU,
    },
    {
        id: "GridVS_CPU",
        content: GridVS_CPU,
    },
    // HiPS shaders
    // Raytracer
    {
        id: "RayTracerVS",
        content: RayTracerVS,
    },
    {
        id: "RayTracerColorFS",
        content: RayTracerColorFS,
    },
    {
        id: "RayTracerGrayscale2ColormapFS",
        content: RayTracerGrayscale2ColormapFS,
    },
    /// Rasterizer
    {
        id: "RasterizerVS",
        content: RasterizerVS,
    },
    {
        id: "RasterizerColorFS",
        content: RasterizerColorFS,
    },
    {
        id: "RasterizerGrayscale2ColormapFS",
        content: RasterizerGrayscale2ColormapFS,
    },
    // Post
    {
        id: "PostVS",
        content: PostVS,
    },
    {
        id: "PostFS",
        content: PostFS,
    },
];

export function loadShadersWebGL1() {
    return shaders;
}
