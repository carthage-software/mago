import { ref } from 'vue';

let wasmModule = null;
let loadPromise = null;

export function useMagoWasm() {
  const isLoading = ref(false);
  const isReady = ref(!!wasmModule);
  const error = ref(null);

  async function loadWasm() {
    if (typeof window === 'undefined') return null;
    if (wasmModule) return wasmModule;
    if (loadPromise) return loadPromise;

    isLoading.value = true;
    error.value = null;

    loadPromise = (async () => {
      try {
        const importFn = new Function('path', 'return import(path)');
        const wasm = await importFn('/wasm/mago_wasm.js');
        await wasm.default('/wasm/mago_wasm_bg.wasm');

        wasmModule = {
          run: wasm.run,
          format: wasm.format,
          getRules: wasm.getRules,
        };
        isReady.value = true;
        return wasmModule;
      } catch (e) {
        error.value = e;
        throw e;
      } finally {
        isLoading.value = false;
      }
    })();

    return loadPromise;
  }

  async function analyze(code, settings) {
    const wasm = await loadWasm();
    const startTime = performance.now();
    const issues = wasm.run(code, settings);
    const endTime = performance.now();

    return {
      issues,
      analysisTimeMs: endTime - startTime,
    };
  }

  async function format(code, phpVersion) {
    const wasm = await loadWasm();
    return wasm.format(code, phpVersion);
  }

  async function getRules() {
    const wasm = await loadWasm();
    return wasm.getRules();
  }

  return {
    isLoading,
    isReady,
    error,
    loadWasm,
    analyze,
    format,
    getRules,
  };
}
