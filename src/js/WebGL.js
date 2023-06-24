//import { loadShadersWebGL1 } from "./ShadersWebGL1";
import { loadShadersWebGL2 } from "./ShadersWebGL2";
// Import resources images
import kernel from '../img/kernel.png';

export let WebGLCtx = (function() {
    // constructor
    function WebGLCtx(ctx, div) {
        const shaders = loadShadersWebGL2();

        this.webclient = new ctx.WebClient(
            div,
            shaders,
            {
                'kernel': kernel,
            }
        );
    };

    return WebGLCtx;
})();

