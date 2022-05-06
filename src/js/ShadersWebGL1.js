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
import GridVS from '../glsl/webgl1/grid/grid.vert'
import GridAitoffFS from '../glsl/webgl1/grid/aitoff.frag'
import GridMollFS from '../glsl/webgl1/grid/mollweide.frag'
import GridOrthoFS from '../glsl/webgl1/grid/ortho.frag'
import GridMercatorFS from '../glsl/webgl1/grid/mercator.frag'
import GridArcFS from '../glsl/webgl1/grid/arc.frag'
import GridTanFS from '../glsl/webgl1/grid/tan.frag'
import GridVS_CPU from '../glsl/webgl1/grid/grid_cpu.vert'
import GridFS_CPU from '../glsl/webgl1/grid/grid_cpu.frag'

// HiPS shaders
// Raytracer
import RayTracerVS from '../glsl/webgl1/hips/raytracer/raytracer.vert'
import RayTracerColorFS from '../glsl/webgl1/hips/raytracer/color.frag'
import RayTracerGrayscale2ColorFS from '../glsl/webgl1/hips/raytracer/grayscale_to_color.frag'
import RayTracerGrayscale2ColormapFS from '../glsl/webgl1/hips/raytracer/grayscale_to_colormap.frag'
// Rasterizer
import RasterizerOrthoVS from '../glsl/webgl1/hips/rasterizer/ortho.vert'
import RasterizerMercatorVS from '../glsl/webgl1/hips/rasterizer/mercator.vert'
import RasterizerAitoffVS from '../glsl/webgl1/hips/rasterizer/aitoff.vert'
import RasterizerGnomonicVS from '../glsl/webgl1/hips/rasterizer/gnomonic.vert'
import RasterizerArcVS from '../glsl/webgl1/hips/rasterizer/arc.vert'
import RasterizerMollVS from '../glsl/webgl1/hips/rasterizer/mollweide.vert'
import RasterizerColorFS from '../glsl/webgl1/hips/rasterizer/color.frag'
import RasterizerGrayscale2ColorFS from '../glsl/webgl1/hips/rasterizer/grayscale_to_color.frag'
import RasterizerGrayscale2ColormapFS from '../glsl/webgl1/hips/rasterizer/grayscale_to_colormap.frag'

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
        id: "GridVS",
        content: GridVS,
    },
    {
        id: "GridAitoffFS",
        content: GridAitoffFS,
    },
    {
        id: "GridMollFS",
        content: GridMollFS,
    },
    {
        id: "GridOrthoFS",
        content: GridOrthoFS,
    },
    {
        id: "GridMercatorFS",
        content: GridMercatorFS,
    },
    {
        id: "GridArcFS",
        content: GridArcFS,
    },
    {
        id: "GridTanFS",
        content: GridTanFS,
    },
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
        id: "RayTracerGrayscale2ColorFS",
        content: RayTracerGrayscale2ColorFS,
    },
    {
        id: "RayTracerGrayscale2ColormapFS",
        content: RayTracerGrayscale2ColormapFS,
    },
    /// Rasterizer
    {
        id: "RasterizerOrthoVS",
        content: RasterizerOrthoVS,
    },
    {
        id: "RasterizerMercatorVS",
        content: RasterizerMercatorVS,
    },
    {
        id: "RasterizerAitoffVS",
        content: RasterizerAitoffVS,
    },
    {
        id: "RasterizerArcVS",
        content: RasterizerArcVS,
    },
    {
        id: "RasterizerGnomonicVS",
        content: RasterizerGnomonicVS,
    },
    {
        id: "RasterizerMollVS",
        content: RasterizerMollVS,
    },
    {
        id: "RasterizerColorFS",
        content: RasterizerColorFS,
    },
    {
        id: "RasterizerGrayscale2ColorFS",
        content: RasterizerGrayscale2ColorFS,
    },
    {
        id: "RasterizerGrayscale2ColormapFS",
        content: RasterizerGrayscale2ColormapFS,
    },
];

export function loadShadersWebGL1() {
    return shaders;
}
