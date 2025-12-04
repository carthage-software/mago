<script setup>
import { onMounted, onUnmounted, watch, ref } from 'vue';
import { useMagoWasm } from '../../composables/useMagoWasm.js';
import { createPlaygroundState } from '../../composables/usePlaygroundState.js';
import { useUrlState } from '../../composables/useUrlState.js';
import PlaygroundToolbar from './PlaygroundToolbar.vue';
import PlaygroundSettings from './PlaygroundSettings.vue';
import PlaygroundEditor from './PlaygroundEditor.vue';
import PlaygroundOutput from './PlaygroundOutput.vue';

const store = createPlaygroundState();
const { state } = store;

const { isLoading: wasmLoading, isReady: wasmReady, analyze, format, loadWasm, getRules } = useMagoWasm();
const { shareError, shareSuccess, isSharing, shareUrl, generateShareUrl, loadFromUrl, copyToClipboard, clearShareUrl, shouldClearShareUrl } = useUrlState();

const hasRunOnce = ref(false);
const highlightedRange = ref(null);

let debounceTimer = null;
const DEBOUNCE_MS = 300;

onMounted(async () => {
  const urlState = await loadFromUrl();
  if (urlState) {
    store.restoreState(urlState);
  }

  try {
    await loadWasm();
    store.setWasmReady(true);
  } catch (e) {
    console.error('Failed to load WASM:', e);
  }
});

onUnmounted(() => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});

watch(wasmReady, async (ready) => {
  store.setWasmReady(ready);
  if (ready && !hasRunOnce.value) {
    hasRunOnce.value = true;
    try {
      const rules = await getRules();
      store.setAvailableRules(rules);
    } catch (e) {
      console.error('Failed to load linter rules:', e);
    }
    runAnalysis();
  }
});

watch(
  () => state.code,
  () => {
    if (!wasmReady.value || !hasRunOnce.value) return;

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    debounceTimer = setTimeout(() => {
      runAnalysis();
    }, DEBOUNCE_MS);
  }
);

watch(
  () => state.settings,
  () => {
    if (!wasmReady.value || !hasRunOnce.value) return;
    runAnalysis();
  },
  { deep: true }
);

watch(
  () => store.getShareableState(),
  (newState) => {
    if (shouldClearShareUrl(newState)) {
      clearShareUrl();
    }
  },
  { deep: true }
);

async function runAnalysis() {
  if (!wasmReady.value) return;

  store.setLoading(true);
  try {
    const results = await analyze(state.code, state.settings);
    store.setResults(results);
  } catch (e) {
    console.error('Analysis failed:', e);
    store.setResults({
      linterIssues: { issues: [] },
      analyzerIssues: { issues: [] },
      parseError: { level: 'error', message: e.message || 'Analysis failed' },
    });
  } finally {
    store.setLoading(false);
  }
}

async function handleFormat() {
  if (!wasmReady.value || state.isLoading) return;

  store.setLoading(true);
  try {
    const formatted = await format(state.code, state.settings.phpVersion);
    store.setCode(formatted);
  } catch (e) {
    console.error('Format failed:', e);
  } finally {
    store.setLoading(false);
  }

  runAnalysis();
}

async function handleShare() {
  try {
    const shareableState = store.getShareableState();
    const url = await generateShareUrl(shareableState);
    await copyToClipboard(url);
  } catch (e) {
  }
}

function handleToggleSettings() {
  store.toggleSettings();
}

function handleCodeChange(code) {
  store.setCode(code);
}

function handleSettingsChange(settings) {
  store.setPhpVersion(settings.phpVersion);
  Object.entries(settings.analyzer).forEach(([key, value]) => {
    store.setAnalyzerSetting(key, value);
  });
  if (settings.linter?.disabledRules) {
    store.setLinterDisabledRules(settings.linter.disabledRules);
  }
}

function handleToggleRule(ruleCode) {
  store.toggleLinterRule(ruleCode);
}

function handleCloseSettings() {
  store.closeSettings();
}

function handleTabChange(tab) {
  store.setActiveTab(tab);
}

function handleHighlightLine(range) {
  highlightedRange.value = range;
}

function handleKeydown(e) {
  const isMeta = e.metaKey || e.ctrlKey;

  if (isMeta && e.shiftKey && (e.key === 'f' || e.key === 'F')) {
    e.preventDefault();
    handleFormat();
  } else if (isMeta && e.key === 's') {
    e.preventDefault();
    handleShare();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <div class="mago-playground">
    <div v-if="wasmLoading" class="loading-overlay">
      <div class="loading-content">
        <div class="loading-spinner"></div>
        <h3>Loading Mago</h3>
        <p>Initializing the analyzer...</p>
      </div>
    </div>

    <div v-if="isSharing" class="loading-overlay sharing-overlay">
      <div class="loading-content">
        <div class="loading-spinner"></div>
        <h3>Generating Share Link</h3>
        <p>Please wait...</p>
      </div>
    </div>

    <PlaygroundToolbar
      :is-loading="state.isLoading || wasmLoading"
      :is-sharing="isSharing"
      :wasm-ready="wasmReady"
      :settings-open="state.settingsOpen"
      :share-success="shareSuccess"
      :share-error="shareError"
      :share-url="shareUrl"
      @format="handleFormat"
      @share="handleShare"
      @toggle-settings="handleToggleSettings"
    />

    <PlaygroundSettings
      v-if="state.settingsOpen"
      :settings="state.settings"
      :available-rules="state.availableRules"
      @update:settings="handleSettingsChange"
      @toggle-rule="handleToggleRule"
      @close="handleCloseSettings"
    />

    <div v-if="!state.settingsOpen" class="playground-panels">
      <PlaygroundEditor
        class="panel-editor"
        :model-value="state.code"
        :disabled="wasmLoading"
        :highlight-range="highlightedRange"
        @update:model-value="handleCodeChange"
      />
      <PlaygroundOutput
        class="panel-output"
        :results="state.results"
        :active-tab="state.activeTab"
        :is-loading="state.isLoading"
        @update:active-tab="handleTabChange"
        @highlight-line="handleHighlightLine"
      />
    </div>

    <div v-if="!state.settingsOpen" class="playground-footer">
      <span class="shortcut-hint">
        <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> Format
      </span>
    </div>
  </div>
</template>

<style scoped>
.mago-playground {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 1400px;
  height: 700px;
  margin: 32px auto;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  overflow: hidden;
  background: var(--vp-c-bg);
}

/* WASM Loading Overlay */
.loading-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--vp-c-bg);
  border-radius: 8px;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  text-align: center;
}

.loading-content h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.loading-content p {
  margin: 0;
  font-size: 14px;
  color: var(--vp-c-text-2);
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 4px solid var(--vp-c-divider);
  border-top-color: var(--vp-c-brand-1);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.sharing-overlay {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
  animation: fadeIn 0.15s ease;
}

.sharing-overlay .loading-content {
  background: var(--vp-c-bg);
  padding: 32px 48px;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
}

.sharing-overlay .loading-spinner {
  width: 32px;
  height: 32px;
  border-width: 3px;
}

.sharing-overlay .loading-content h3 {
  font-size: 16px;
}

.sharing-overlay .loading-content p {
  font-size: 13px;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.playground-panels {
  display: flex;
  flex: 1;
  min-height: 0;
}

.panel-editor,
.panel-output {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.panel-editor {
  border-right: 1px solid var(--vp-c-divider);
}

.playground-footer {
  display: flex;
  justify-content: center;
  gap: 24px;
  padding: 8px 16px;
  background: var(--vp-c-bg-soft);
  border-top: 1px solid var(--vp-c-divider);
}

.shortcut-hint {
  font-size: 12px;
  color: var(--vp-c-text-3);
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  font-family: inherit;
  font-size: 11px;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-divider);
  border-radius: 4px;
  margin: 0 2px;
}

/* Responsive: stack panels on mobile */
@media (max-width: 768px) {
  .mago-playground {
    height: auto;
    min-height: 600px;
    margin: 16px auto;
  }

  .playground-panels {
    flex-direction: column;
  }

  .panel-editor {
    flex: 1;
    border-right: none;
    border-bottom: 1px solid var(--vp-c-divider);
    min-height: 250px;
  }

  .panel-output {
    flex: 1;
    min-height: 250px;
  }

  .playground-footer {
    display: none;
  }
}
</style>
