//import { loadShadersWebGL1 } from "./ShadersWebGL1";
import { loadShadersWebGL2 } from "./ShadersWebGL2";
// Import resources images
import kernel from '../img/kernel.png';
import colormaps from '../img/colormaps/colormaps.png';

export let WebGLCtx = (function() {
    /** Constructor */
    async function WebGLCtx () {
        // Check for webgl2 support
        const webGL2support = checkForWebGL2Support();

        if (webGL2support) {
            return await import('../../pkg-webgl2');
        } else {
            // WebGL1 not supported
            // According to caniuse, https://caniuse.com/webgl2, webgl2 is supported by 89% of users
            throw "WebGL2 not supported by your browser";
            //return await import('../core/pkg-webgl1');
        }
    };

    WebGLCtx.checkForWebGL2Support = checkForWebGL2Support;

    WebGLCtx.init = function(ctx, div) {
        //const shaders = WebGLCtx.checkForWebGL2Support() ? loadShadersWebGL2() : loadShadersWebGL1();
        const shaders = loadShadersWebGL2();
        return new ctx.WebClient(
            div,
            shaders,
            {
                'kernel': kernel,
                'colormaps': colormaps,
            }
        );
    }

    return WebGLCtx;
})();

function checkForWebGL2Support() {        
    const gl = document
        .createElement('canvas')
        .getContext('webgl2');
    return gl;
    /*
    // Run WebGL1 version only
    return false;
    */
}


