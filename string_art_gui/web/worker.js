import init, { Computation } from "./pkg/string_art_bindings.js";

onmessage = async (e) => {
  await init();
  console.log("wasm ready");
  let computation = new Computation(e.data);
  console.log("built wasm objects");
  let step = computation.next();
  while (step != null) {
    console.log("iteration done");
    postMessage(step);
    step = computation.next();
  }
};
