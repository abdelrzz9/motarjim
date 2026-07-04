// Stub WASM module - replaced by real build with wasm-pack
// Exports on window for script tag loading
window.motarjim_wasm_init = async function() {};
window.WasmCompiler = class WasmCompiler {
  compile(html, css, platform) {
    throw new Error('WASM module not built. Run wasm-pack build in crates/motarjim-wasm.');
  }
};
