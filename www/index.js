import * as wasm from "wasm-langton-ant";


function turmite(target) {
    wasm.debug();

    let canvas = document.getElementById(target);
    let ctx = canvas.getContext("2d");
    let turmite = wasm.Turmite.new(canvas.width, canvas.height, 4);
    let cb = () => {
        if (turmite.is_active()) {
            turmite.tick(ctx);
            requestAnimationFrame(cb);
        }
    };
    cb();
}

turmite("turmite");
