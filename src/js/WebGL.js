// Import resources images
import kernel from '../img/kernel.png';

export let WebGLCtx = (function() {
    // constructor
    function WebGLCtx(ctx, div) {
        this.webclient = new ctx.WebClient(
            div,
            {
                'kernel': kernel,
            }
        );
    };

    return WebGLCtx;
})();

