import { useMagoWasm } from './useMagoWasm.js';

export function useMagoTasks() {
  const { analyze: doAnalyze, format: doFormat, loadWasm, isReady } = useMagoWasm();

  async function ensureReady() {
    await loadWasm();
    return isReady.value;
  }

  async function withTiming(fn) {
    const start = performance.now();
    try {
      const result = await fn();
      const end = performance.now();
      return { result, timeMs: end - start, error: null };
    } catch (e) {
      const end = performance.now();
      return { result: null, timeMs: end - start, error: e?.message || 'Task failed' };
    }
  }

  async function analyze(code, settings) {
    await ensureReady();
    const { result, timeMs, error } = await withTiming(() => doAnalyze(code, settings));
    if (error) {
      return { issues: [], analysisTimeMs: timeMs, error };
    }
    return {
      issues: result?.issues || [],
      analysisTimeMs: timeMs ?? null,
      error: null,
    };
  }

  async function format(code, phpVersion) {
    await ensureReady();
    const { result, timeMs, error } = await withTiming(() => doFormat(code, phpVersion));
    if (error) {
      return { code, timeMs, error };
    }
    return { code: result, timeMs, error: null };
  }
  
  return {
    analyze,
    format,
  };
}
