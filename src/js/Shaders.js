/* Import all the shaders here*/ 
// Catalog shaders
import CatalogAitoffVS from '../render/src/shaders/catalogs/aitoff.vert';
import CatalogMercatVS from '../render/src/shaders/catalogs/mercator.vert';
import CatalogMollVS from '../render/src/shaders/catalogs/mollweide.vert';
import CatalogOrthoVS from '../render/src/shaders/catalogs/ortho.vert';
import CatalogFS from '../render/src/shaders/catalogs/catalog.frag';

// Colormap shaders
import ColormapVS from '../render/src/shaders/colormaps/colormap.vert'
import ColormapBlackWhiteFS from '../render/src/shaders/colormaps/blackwhite.frag'
import ColormapBluePastelRedFS from '../render/src/shaders/colormaps/BluePastelRed.frag'
import ColormapIDL_CB_BrBGFS from '../render/src/shaders/colormaps/IDL_CB_BrBG.frag'
import ColormapIDL_CB_GnBuFS from '../render/src/shaders/colormaps/IDL_CB_GnBu.frag'
import ColormapIDL_CB_YIGnBuFS from '../render/src/shaders/colormaps/IDL_CB_YIGnBu.frag'
import ColormapRedTemperatureFS from '../render/src/shaders/colormaps/red.frag'

// Grid shader
import GridVS from '../render/src/shaders/grid/grid.vert'
import GridAitoffFS from '../render/src/shaders/grid/aitoff.frag'
import GridMollFS from '../render/src/shaders/grid/mollweide.frag'
import GridOrthoFS from '../render/src/shaders/grid/ortho.frag'
import GridMercatorFS from '../render/src/shaders/grid/mercator.frag'
// HiPS shaders
// Raytracer
import RayTracerVS from '../render/src/shaders/hips/raytracer/raytracer.vert'
import RayTracerColorFS from '../render/src/shaders/hips/raytracer/color.frag'
import RayTracerGrayscale2ColorFS from '../render/src/shaders/hips/raytracer/grayscale_to_color.frag'
import RayTracerGrayscale2ColormapFS from '../render/src/shaders/hips/raytracer/grayscale_to_colormap.frag'

// Rasterizer
import RasterizerOrthoVS from '../render/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerMercatorVS from '../render/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerAitoffVS from '../render/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerArcVS from '../render/src/shaders/hips/rasterizer/arc.vert'
import RasterizerMollVS from '../render/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerColorFS from '../render/src/shaders/hips/rasterizer/color.frag'
import RasterizerGrayscale2ColorFS from '../render/src/shaders/hips/rasterizer/grayscale_to_color.frag'
import RasterizerGrayscale2ColormapFS from '../render/src/shaders/hips/rasterizer/grayscale_to_colormap.frag'
// Misc
import TextVS from '../render/src/shaders/misc/text.vert'
import TextFS from '../render/src/shaders/misc/text.frag'

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
        id: "CatalogMollVS",
        content: CatalogMollVS,
    },
    {
        id: "CatalogOrthoVS",
        content: CatalogOrthoVS,
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
