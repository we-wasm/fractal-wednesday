window.onload = () => {
  const canvas = document.querySelector("#gl");
  const gl = canvas.getContext("webgl2");

  if (gl === null) {
    return console.log(
      "Unable to initialize WebGL. Your browser or machine may not support it."
    );
  }

  const brightness = document.querySelector("#brightness");
  brightness.addEventListener("input", e =>
    changeBrightness(gl, e.target.value)
  );

  changeBrightness(gl, brightness.value);
};

function changeBrightness(gl, brightness) {
  if (!gl) {
    return;
  }
  const scaled = isNaN(+brightness) ? 0 : +brightness / 100;

  gl.clearColor(scaled, scaled, scaled, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);
}
