/* Import all the shaders here*/ 
// Catalog shaders
import CatalogAitoffVS from '../core/src/shaders/catalogs/aitoff.vert';
import CatalogMercatVS from '../core/src/shaders/catalogs/mercator.vert';
import CatalogArcVS from '../core/src/shaders/catalogs/arc.vert';
import CatalogTanVS from '../core/src/shaders/catalogs/tan.vert';
import CatalogMollVS from '../core/src/shaders/catalogs/mollweide.vert';
import CatalogOrthoVS from '../core/src/shaders/catalogs/ortho.vert';
import CatalogOrthoFS from '../core/src/shaders/catalogs/ortho.frag';
import CatalogFS from '../core/src/shaders/catalogs/catalog.frag';

// Colormap shaders
import ColormapVS from '../core/src/shaders/colormaps/colormap.vert'
import ColormapBlackWhiteFS from '../core/src/shaders/colormaps/blackwhite.frag'
import ColormapBluePastelRedFS from '../core/src/shaders/colormaps/BluePastelRed.frag'
import ColormapIDL_CB_BrBGFS from '../core/src/shaders/colormaps/IDL_CB_BrBG.frag'
import ColormapIDL_CB_GnBuFS from '../core/src/shaders/colormaps/IDL_CB_GnBu.frag'
import ColormapIDL_CB_YIGnBuFS from '../core/src/shaders/colormaps/IDL_CB_YIGnBu.frag'
import ColormapRedTemperatureFS from '../core/src/shaders/colormaps/red.frag'

// Grid shader
import GridVS from '../core/src/shaders/grid/grid.vert'
import GridAitoffFS from '../core/src/shaders/grid/aitoff.frag'
import GridMollFS from '../core/src/shaders/grid/mollweide.frag'
import GridOrthoFS from '../core/src/shaders/grid/ortho.frag'
import GridMercatorFS from '../core/src/shaders/grid/mercator.frag'
import GridArcFS from '../core/src/shaders/grid/arc.frag'
import GridTanFS from '../core/src/shaders/grid/tan.frag'
import GridVS_CPU from '../core/src/shaders/grid/grid_cpu.vert'
import GridFS_CPU from '../core/src/shaders/grid/grid_cpu.frag'

// HiPS shaders
// Raytracer
import RayTracerVS from '../core/src/shaders/hips/raytracer/raytracer.vert'
import RayTracerColorFS from '../core/src/shaders/hips/raytracer/color.frag'
import RayTracerGrayscale2ColorFS from '../core/src/shaders/hips/raytracer/grayscale_to_color.frag'
import RayTracerGrayscale2ColormapFS from '../core/src/shaders/hips/raytracer/grayscale_to_colormap.frag'
import RayTracerGrayscale2ColorIntegerFS from '../core/src/shaders/hips/raytracer/grayscale_to_color_i.frag'
import RayTracerGrayscale2ColormapIntegerFS from '../core/src/shaders/hips/raytracer/grayscale_to_colormap_i.frag'

// Rasterizer
import RasterizerOrthoVS from '../core/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerMercatorVS from '../core/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerAitoffVS from '../core/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerGnomonicVS from '../core/src/shaders/hips/rasterizer/gnomonic.vert'
import RasterizerArcVS from '../core/src/shaders/hips/rasterizer/arc.vert'
import RasterizerMollVS from '../core/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerColorFS from '../core/src/shaders/hips/rasterizer/color.frag'
import RasterizerGrayscale2ColorFS from '../core/src/shaders/hips/rasterizer/grayscale_to_color.frag'
import RasterizerGrayscale2ColormapFS from '../core/src/shaders/hips/rasterizer/grayscale_to_colormap.frag'
import RasterizerGrayscale2ColorIntegerFS from '../core/src/shaders/hips/rasterizer/grayscale_to_color_i.frag'
import RasterizerGrayscale2ColormapIntegerFS from '../core/src/shaders/hips/rasterizer/grayscale_to_colormap_i.frag'
// Misc
import TextVS from '../core/src/shaders/misc/text.vert'
import TextFS from '../core/src/shaders/misc/text.frag'

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
        id: "ColormapVS",
        content: ColormapVS,
    },
    {
        id: "ColormapBlackWhiteFS",
        content: ColormapBlackWhiteFS
    },
    {
        id: "ColormapBluePastelRedFS",
        content: ColormapBluePastelRedFS
    },
    {
        id: "ColormapIDL_CB_BrBGFS",
        content: ColormapIDL_CB_BrBGFS
    },
    {
        id: "ColormapIDL_CB_GnBuFS",
        content: ColormapIDL_CB_GnBuFS
    },
    {
        id: "ColormapIDL_CB_YIGnBuFS",
        content: ColormapIDL_CB_YIGnBuFS
    },
    {
        id: "ColormapRedTemperatureFS",
        content: ColormapRedTemperatureFS
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
    {
        id: "RayTracerGrayscale2ColorIntegerFS",
        content: RayTracerGrayscale2ColorIntegerFS,
    },
    {
        id: "RayTracerGrayscale2ColormapIntegerFS",
        content: RayTracerGrayscale2ColormapIntegerFS,
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
    {
        id: "RasterizerGrayscale2ColorIntegerFS",
        content: RasterizerGrayscale2ColorIntegerFS,
    },
    {
        id: "RasterizerGrayscale2ColormapIntegerFS",
        content: RasterizerGrayscale2ColormapIntegerFS,
    },
    // Misc
    {
        id: "TextVS",
        content: TextVS,
    },
    {
        id: "TextFS",
        content: TextFS,
    },
];

export function loadShaders() {
    return shaders;
}
