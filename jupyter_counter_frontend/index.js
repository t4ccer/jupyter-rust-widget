import init, { render_counter_impl } from './pkg/jupyter_counter_frontend.js';
import wasmInlineSource from './pkg/jupyter_counter_frontend_bg.wasm';

export async function render_counter(model, el) {
  await init(wasmInlineSource);
  render_counter_impl(model, el);
}
