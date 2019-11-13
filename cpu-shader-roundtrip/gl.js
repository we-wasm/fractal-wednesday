window.onload = () => {
  const canvas = document.querySelector("#gl");
  console.log(canvas);
  const gl = canvas.getContext("webgl2", { antialias: false });

  if (gl === null) {
    return console.log(
      "Unable to initialize WebGL. Your browser or machine may not support it."
    );
  }

  gl.clearColor(0.0, 0.0, 0.0, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);
};
