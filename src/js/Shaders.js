/* Import all the shaders here*/ 
// Catalog shaders
import CatalogAitoffVS from '../render/src/shaders/catalogs/aitoff.vert';
import CatalogAitoffFS from '../render/src/shaders/catalogs/aitoff.frag';

import CatalogMercatVS from '../render/src/shaders/catalogs/mercator.vert';
import CatalogMercatFS from '../render/src/shaders/catalogs/mercator.frag';

import CatalogMollVS from '../render/src/shaders/catalogs/mollweide.vert';
import CatalogMollFS from '../render/src/shaders/catalogs/mollweide.frag';

import CatalogOrthoVS from '../render/src/shaders/catalogs/ortho.vert';
import CatalogOrthoFS from '../render/src/shaders/catalogs/ortho.frag';

// Colormap shaders
import ColormapBlackWhiteVS from '../render/src/shaders/colormaps/blackwhite.vert'
import ColormapBlackWhiteFS from '../render/src/shaders/colormaps/blackwhite.frag'

import ColormapBluePastelRedVS from '../render/src/shaders/colormaps/BluePastelRed.vert'
import ColormapBluePastelRedFS from '../render/src/shaders/colormaps/BluePastelRed.frag'

import ColormapIDL_CB_BrBGVS from '../render/src/shaders/colormaps/IDL_CB_BrBG.vert'
import ColormapIDL_CB_BrBGFS from '../render/src/shaders/colormaps/IDL_CB_BrBG.frag'

import ColormapIDL_CB_GnBuVS from '../render/src/shaders/colormaps/IDL_CB_GnBu.vert'
import ColormapIDL_CB_GnBuFS from '../render/src/shaders/colormaps/IDL_CB_GnBu.frag'

import ColormapIDL_CB_YIGnBuVS from '../render/src/shaders/colormaps/IDL_CB_YIGnBu.vert'
import ColormapIDL_CB_YIGnBuFS from '../render/src/shaders/colormaps/IDL_CB_YIGnBu.frag'

import ColormapRedTemperatureVS from '../render/src/shaders/colormaps/red.vert'
import ColormapRedTemperatureFS from '../render/src/shaders/colormaps/red.frag'
// Grid shader
import GridAitoffVS from '../render/src/shaders/grid/aitoff.vert'
import GridAitoffFS from '../render/src/shaders/grid/aitoff.frag'

import GridMollVS from '../render/src/shaders/grid/mollweide.vert'
import GridMollFS from '../render/src/shaders/grid/mollweide.frag'

import GridOrthoVS from '../render/src/shaders/grid/ortho.vert'
import GridOrthoFS from '../render/src/shaders/grid/ortho.frag'

import GridMercatorVS from '../render/src/shaders/grid/mercator.vert'
import GridMercatorFS from '../render/src/shaders/grid/mercator.frag'
// HiPS shaders
// Raytracer
import RayTracerVS from '../render/src/shaders/hips/raytracer/raytracer.vert'
import RayTracerFS from '../render/src/shaders/hips/raytracer/raytracer.frag'

import RayTracerFITSVS from '../render/src/shaders/hips/raytracer/raytracer.vert'
import RayTracerFITSFS from '../render/src/shaders/hips/raytracer/raytracer_fits.frag'

import RayTracerFITSIVS from '../render/src/shaders/hips/raytracer/raytracer.vert'
import RayTracerFITSIFS from '../render/src/shaders/hips/raytracer/raytracer_fits_i.frag'

// Rasterizer
import RasterizerOrthoVS from '../render/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerOrthoFS from '../render/src/shaders/hips/rasterizer/frag.glsl'
import RasterizerOrthoFITSVS from '../render/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerOrthoFITSFS from '../render/src/shaders/hips/rasterizer/frag_fits.glsl'
import RasterizerOrthoFITSIVS from '../render/src/shaders/hips/rasterizer/ortho.vert'
import RasterizerOrthoFITSIFS from '../render/src/shaders/hips/rasterizer/frag_fits_i.glsl'

import RasterizerMercatorVS from '../render/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerMercatorFS from '../render/src/shaders/hips/rasterizer/frag.glsl'
import RasterizerMercatorFITSVS from '../render/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerMercatorFITSFS from '../render/src/shaders/hips/rasterizer/frag_fits.glsl'
import RasterizerMercatorFITSIVS from '../render/src/shaders/hips/rasterizer/mercator.vert'
import RasterizerMercatorFITSIFS from '../render/src/shaders/hips/rasterizer/frag_fits_i.glsl'

import RasterizerAitoffVS from '../render/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerAitoffFS from '../render/src/shaders/hips/rasterizer/frag.glsl'
import RasterizerAitoffFITSVS from '../render/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerAitoffFITSFS from '../render/src/shaders/hips/rasterizer/frag_fits.glsl'
import RasterizerAitoffFITSIVS from '../render/src/shaders/hips/rasterizer/aitoff.vert'
import RasterizerAitoffFITSIFS from '../render/src/shaders/hips/rasterizer/frag_fits_i.glsl'

import RasterizerMollVS from '../render/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerMollFS from '../render/src/shaders/hips/rasterizer/frag.glsl'
import RasterizerMollFITSVS from '../render/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerMollFITSFS from '../render/src/shaders/hips/rasterizer/frag_fits.glsl'
import RasterizerMollFITSIVS from '../render/src/shaders/hips/rasterizer/mollweide.vert'
import RasterizerMollFITSIFS from '../render/src/shaders/hips/rasterizer/frag_fits_i.glsl'
// Misc
import TextVS from '../render/src/shaders/misc/text.vert'
import TextFS from '../render/src/shaders/misc/text.frag'

let shaders = [
    // Catalog shaders
    {
        name: "catalog_aitoff",
        vert: CatalogAitoffVS,
        frag: CatalogAitoffFS,
    },
    {
        name: "catalog_mercator",
        vert: CatalogMercatVS,
        frag: CatalogMercatFS,
    },
    {
        name: "catalog_mollweide",
        vert: CatalogMollVS,
        frag: CatalogMollFS,
    },
    {
        name: "catalog_ortho",
        vert: CatalogOrthoVS,
        frag: CatalogOrthoFS,
    },

    // Colormap shaders
    {
        name: "black_white_linear",
        vert: ColormapBlackWhiteVS,
        frag: ColormapBlackWhiteFS,
    },
    {
        name: "BluePastelRed",
        vert: ColormapBluePastelRedVS,
        frag: ColormapBluePastelRedFS,
    },
    {
        name: "IDL_CB_BrBG",
        vert: ColormapIDL_CB_BrBGVS,
        frag: ColormapIDL_CB_BrBGFS,
    },
    {
        name: "IDL_CB_GnBu",
        vert: ColormapIDL_CB_GnBuVS,
        frag: ColormapIDL_CB_GnBuFS,
    },
    {
        name: "IDL_CB_YIGnBu",
        vert: ColormapIDL_CB_YIGnBuVS,
        frag: ColormapIDL_CB_YIGnBuFS,
    },
    {
        name: "red_temperature",
        vert: ColormapRedTemperatureVS,
        frag: ColormapRedTemperatureFS,
    },

    // Grid shader
    {
        name: "grid_aitoff",
        vert: GridAitoffVS,
        frag: GridAitoffFS,
    },
    {
        name: "grid_ortho",
        vert: GridOrthoVS,
        frag: GridOrthoFS,
    },
    {
        name: "grid_mollweide",
        vert: GridMollVS,
        frag: GridMollFS,
    },
    {
        name: "grid_mercator",
        vert: GridMercatorVS,
        frag: GridMercatorFS,
    },
    // HiPS shaders
    // Raytracer
    {
        name: "raytracer",
        vert: RayTracerVS,
        frag: RayTracerFS,
    },
    {
        name: "raytracer_fits",
        vert: RayTracerFITSVS,
        frag: RayTracerFITSFS,
    },
    {
        name: "raytracer_fits_i",
        vert: RayTracerFITSIVS,
        frag: RayTracerFITSIFS,
    },
    /// Rasterizer
    // Aitoff
    {
        name: "rasterizer_aitoff",
        vert: RasterizerAitoffVS,
        frag: RasterizerAitoffFS,
    },
    {
        name: "rasterizer_aitoff_fits",
        vert: RasterizerAitoffFITSVS,
        frag: RasterizerAitoffFITSFS,
    },
    {
        name: "rasterizer_aitoff_fits_i",
        vert: RasterizerAitoffFITSIVS,
        frag: RasterizerAitoffFITSIFS,
    },
    // Mercator
    {
        name: "rasterizer_mercator",
        vert: RasterizerMercatorVS,
        frag: RasterizerMercatorFS,
    },
    {
        name: "rasterizer_mercator_fits",
        vert: RasterizerMercatorFITSVS,
        frag: RasterizerMercatorFITSFS,
    },
    {
        name: "rasterizer_mercator_fits_i",
        vert: RasterizerMercatorFITSIVS,
        frag: RasterizerMercatorFITSIFS,
    },
    // Mollweide
    {
        name: "rasterizer_mollweide",
        vert: RasterizerMollVS,
        frag: RasterizerMollFS,
    },
    {
        name: "rasterizer_mollweide_fits",
        vert: RasterizerMollFITSVS,
        frag: RasterizerMollFITSFS,
    },
    {
        name: "rasterizer_mollweide_fits_i",
        vert: RasterizerMollFITSIVS,
        frag: RasterizerMollFITSIFS,
    },
    // Ortho
    {
        name: "rasterizer_ortho",
        vert: RasterizerOrthoVS,
        frag: RasterizerOrthoFS,
    },
    {
        name: "rasterizer_ortho_fits",
        vert: RasterizerOrthoFITSVS,
        frag: RasterizerOrthoFITSFS,
    },
    {
        name: "rasterizer_ortho_fits_i",
        vert: RasterizerOrthoFITSIVS,
        frag: RasterizerOrthoFITSIFS,
    },
    // Misc
    {
        name: "text",
        vert: TextVS,
        frag: TextFS,
    },
];

export function loadShaders(webgl) {
    return shaders;
}
