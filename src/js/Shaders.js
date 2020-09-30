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
import RayTracerFS from '../render/src/shaders/hips/raytracer/raytracer.frag'
import RayTracerFITSFS from '../render/src/shaders/hips/raytracer/raytracer_fits.frag'
import RayTracerFITSIFS from '../render/src/shaders/hips/raytracer/raytracer_fits_i.frag'

// Rasterizer
import RasterizerOrthoVS from '../render/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerMercatorVS from '../render/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerAitoffVS from '../render/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerMollVS from '../render/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerColorFS from '../render/src/shaders/hips/rasterizer/frag_color.glsl'
import RasterizerGrayscaleToColorFS from '../render/src/shaders/hips/rasterizer/frag_grayscale_to_color.glsl'
import RasterizerGrayscaleToColormapFS from '../render/src/shaders/hips/rasterizer/frag_grayscale_to_colormap.glsl'
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
        id: "RayTracerFS",
        content: RayTracerFS,
    },
    {
        id: "RayTracerFITSFS",
        content: RayTracerFITSFS,
    },
    {
        id: "RayTracerFITSIFS",
        content: RayTracerFITSIFS,
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
        id: "RasterizerMollVS",
        content: RasterizerMollVS,
    },
    {
        id: "RasterizerColorFS",
        content: RasterizerColorFS,
    },
    {
        id: "RasterizerGrayscaleToColorFS",
        content: RasterizerGrayscaleToColorFS,
    },
    {
        id: "RasterizerGrayscaleToColormapFS",
        content: RasterizerGrayscaleToColormapFS,
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
