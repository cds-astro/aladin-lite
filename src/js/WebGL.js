//import { loadShadersWebGL1 } from "./ShadersWebGL1";
import { loadShadersWebGL2 } from "./ShadersWebGL2";
// Import resources images
import kernel from '../img/kernel.png';
import colormaps from '../img/colormaps/colormaps.png';
import letters from '../img/letters.png';
import lettersMetadata from '../img/letters.json';

export let WebGLCtx = (function() {
    // constructor
    function WebGLCtx(ctx, div) {
        const shaders = loadShadersWebGL2();
        const lettersMeta = JSON.stringify(lettersMetadata);

        this.webclient = new ctx.WebClient(
            div,
            shaders,
            {
                'kernel': kernel,
                'colormaps': colormaps,
                'letters': letters,
                'letters_metadata': lettersMeta,
            }
        );
    };

    return WebGLCtx;
})();

