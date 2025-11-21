import init, { Computation } from "./pkg/string_art_bindings.js";

onmessage = async (e) => {
  await init();
  let computation = new Computation(e.data);
  postMessage(computation.rect());
  let step = computation.next();
  while (step != null) {
    postMessage(step);
    step = computation.next();
  }
};
