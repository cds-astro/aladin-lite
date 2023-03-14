/* Import all the shaders here*/ 
// Catalog shaders
import CatalogAitoffVS from '../glsl/webgl2/catalogs/aitoff.vert';
import CatalogMercatVS from '../glsl/webgl2/catalogs/mercator.vert';
import CatalogArcVS from '../glsl/webgl2/catalogs/arc.vert';
import CatalogTanVS from '../glsl/webgl2/catalogs/tan.vert';
import CatalogMollVS from '../glsl/webgl2/catalogs/mollweide.vert';
import CatalogHEALPixVS from '../glsl/webgl2/catalogs/healpix.vert';
import CatalogOrthoVS from '../glsl/webgl2/catalogs/ortho.vert';
import CatalogOrthoFS from '../glsl/webgl2/catalogs/ortho.frag';
import CatalogFS from '../glsl/webgl2/catalogs/catalog.frag';

// Colormap shaders
import ColormapCatalogVS from '../glsl/webgl2/colormaps/colormap.vert'
import ColormapCatalogFS from '../glsl/webgl2/colormaps/colormap.frag'

// Grid shader
import GridVS_CPU from '../glsl/webgl2/grid/grid_cpu.vert'
import GridFS_CPU from '../glsl/webgl2/grid/grid_cpu.frag'

// HiPS shaders
// Raytracer
import RayTracerVS from '../glsl/webgl2/hips/raytracer/raytracer.vert'
import RayTracerColorFS from '../glsl/webgl2/hips/raytracer/color.frag'
import RayTracerGrayscale2ColormapFS from '../glsl/webgl2/hips/raytracer/grayscale_to_colormap.frag'
import RayTracerGrayscale2ColormapIntegerFS from '../glsl/webgl2/hips/raytracer/grayscale_to_colormap_i.frag'
import RayTracerGrayscale2ColormapUnsignedFS from '../glsl/webgl2/hips/raytracer/grayscale_to_colormap_u.frag'
import RayTracerFontVS from '../glsl/webgl2/hips/raytracer/backcolor.vert'
import RayTracerFontFS from '../glsl/webgl2/hips/raytracer/backcolor.frag'

// Rasterizer
import RasterizerVS from '../glsl/webgl2/hips/rasterizer/raster.vert'
import RasterizerColorFS from '../glsl/webgl2/hips/rasterizer/color.frag'
import RasterizerGrayscale2ColormapFS from '../glsl/webgl2/hips/rasterizer/grayscale_to_colormap.frag'
import RasterizerGrayscale2ColormapIntegerFS from '../glsl/webgl2/hips/rasterizer/grayscale_to_colormap_i.frag'
import RasterizerGrayscale2ColormapUnsignedFS from '../glsl/webgl2/hips/rasterizer/grayscale_to_colormap_u.frag'

// Shader passes
import PostVS from '../glsl/webgl2/passes/post_vertex_100es.glsl'
import PostFS from '../glsl/webgl2/passes/post_fragment_100es.glsl'

// Shader fits image
import FitsVS from '../glsl/webgl2/fits/vert.glsl'
import FitsFS from '../glsl/webgl2/fits/frag_sampler.glsl'
import FitsFSUnsigned from '../glsl/webgl2/fits/frag_usampler.glsl'
import FitsFSInteger from '../glsl/webgl2/fits/frag_isampler.glsl'

let shaders = [
    // Catalog shaders
    {
        id: "CatalogAitoffVS",
        content: CatalogAitoffVS,
    },
    {
        id: "CatalogHEALPixVS",
        content: CatalogHEALPixVS,
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
        id: "GridVS_CPU",
        content: GridVS_CPU,
    },
    {
        id: "GridFS_CPU",
        content: GridFS_CPU,
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
    {
        id: "RayTracerGrayscale2ColormapIntegerFS",
        content: RayTracerGrayscale2ColormapIntegerFS,
    },
    {
        id: "RayTracerGrayscale2ColormapUnsignedFS",
        content: RayTracerGrayscale2ColormapUnsignedFS,
    },
    {
        id: "RayTracerFontVS",
        content: RayTracerFontVS,
    },
    {
        id: "RayTracerFontFS",
        content: RayTracerFontFS,
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
    {
        id: "RasterizerGrayscale2ColormapIntegerFS",
        content: RasterizerGrayscale2ColormapIntegerFS,
    },
    {
        id: "RasterizerGrayscale2ColormapUnsignedFS",
        content: RasterizerGrayscale2ColormapUnsignedFS,
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
    // Fits
    {
        id: "FitsVS",
        content: FitsVS,
    },
    {
        id: "FitsFS",
        content: FitsFS,
    },
    {
        id: "FitsFSUnsigned",
        content: FitsFSUnsigned,
    },
    {
        id: "FitsFSInteger",
        content: FitsFSInteger,
    },
];

export function loadShadersWebGL2() {
    return shaders;
}
