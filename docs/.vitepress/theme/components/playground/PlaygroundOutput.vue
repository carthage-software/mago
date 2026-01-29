<script setup>
import { computed, ref, onMounted, watch, nextTick } from 'vue';

const props = defineProps({
  analyzerResults: {
    type: Object,
    default: null,
  },
  formatterResults: {
    type: Object,
    default: null,
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  activeTab: {
    type: String,
    default: 'issues',
  },
  code: {
    type: String,
    default: '',
  },
  phpVersion: {
    type: String,
    default: '8.4',
  },
});

const emit = defineEmits(['highlight-line', 'update-active-tab']);

const activeFilter = ref(null);
const formattedCodeRef = ref(null);
let Prism = null;

const severityOrder = {
  error: 0,
  warning: 1,
  note: 2,
  help: 3,
};

const allIssues = computed(() => {
  if (!props.analyzerResults) return [];
  return props.analyzerResults.issues || [];
});

function matchesSource(issueSource, filterSource) {
  if (issueSource === 'both') return true;
  return issueSource === filterSource;
}

const displayedIssues = computed(() => {
  let issues = allIssues.value;

  if (activeFilter.value) {
    issues = issues.filter((i) => matchesSource(i.source, activeFilter.value));
  }

  return [...issues].sort((a, b) => {
    const aOrder = severityOrder[(a.level || 'note').toLowerCase()] ?? 3;
    const bOrder = severityOrder[(b.level || 'note').toLowerCase()] ?? 3;
    return aOrder - bOrder;
  });
});

const linterCount = computed(() => {
  return allIssues.value.filter((i) => i.source === 'linter' || i.source === 'both').length;
});

const analyzerCount = computed(() => {
  return allIssues.value.filter((i) => i.source === 'analyzer' || i.source === 'both').length;
});

function formatDuration(ms) {
  if (ms === null || ms === undefined) return null;
  if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

const analysisTime = computed(() => formatDuration(props.analyzerResults?.analysisTimeMs));
const formatTime = computed(() => formatDuration(props.formatterResults?.timeMs));

const showError = computed(() => {
  return props.activeTab === 'formatter'
    ? Boolean(props.formatterResults?.error)
    : Boolean(props.analyzerResults?.error);
});

function toggleFilter(source) {
  if (activeFilter.value === source) {
    activeFilter.value = null; // Toggle off
  } else {
    activeFilter.value = source;
  }
}

function setActiveTab(tab) {
  emit('update-active-tab', tab);
}

function getLevelClass(level) {
  const levelLower = (level || 'note').toLowerCase();
  return `level-${levelLower}`;
}

function getLevelIcon(level) {
  const levelLower = (level || 'note').toLowerCase();
  switch (levelLower) {
    case 'error':
      return '●';
    case 'warning':
      return '▲';
    default:
      return '○';
  }
}

function getDisplaySource(source) {
  if (source === 'both') {
    if (activeFilter.value === 'linter') return 'linter';
    if (activeFilter.value === 'analyzer') return 'analyzer';
    return 'both';
  }
  return source;
}

function formatCode(code) {
  if (!code) return null;
  return code;
}

function handleIssueHover(issue) {
  if (issue.annotations && issue.annotations.length > 0) {
    const ann = issue.annotations[0];
    emit('highlight-line', {
      startLine: ann.startLine,
      startColumn: ann.startColumn,
      endLine: ann.endLine,
      endColumn: ann.endColumn,
    });
  } else {
    emit('highlight-line', null);
  }
}

function handleIssueLeave() {
  emit('highlight-line', null);
}

function highlightFormatted(el) {
  if (!el) return;
  const code = (props.formatterResults?.code ?? props.code) || '';
  if (Prism && Prism.languages.php) {
    el.innerHTML = Prism.highlight(code, Prism.languages.php, 'php');
  } else {
    el.textContent = code;
  }
}

onMounted(async () => {
  if (typeof window !== 'undefined') {
    const prismModule = await import('prismjs');
    Prism = prismModule.default || prismModule;
    await import('prismjs/components/prism-markup-templating');
    await import('prismjs/components/prism-php');
  }
  await nextTick();
  highlightFormatted(formattedCodeRef.value);
});

watch(
  () => [props.activeTab, props.code, props.formatterResults?.code],
  async () => {
    await nextTick();
    highlightFormatted(formattedCodeRef.value);
  }
);
</script>

<template>
  <div class="playground-output">
    <div class="output-header">
      <div class="tabs">
        <button
          :class="['btn', 'btn-secondary', 'tab-btn', { active: activeTab === 'issues' }]"
          @click="setActiveTab('issues')"
        >
          Issues
        </button>
        <button
          :class="['btn', 'btn-secondary', 'tab-btn', { active: activeTab === 'formatter' }]"
          @click="setActiveTab('formatter')"
        >
          Formatter
        </button>
      </div>
      <div class="header-right">
        <button
          v-if="showError"
          class="badge error-badge"
          title="Invalid input"
        >
          ERROR
        </button>
        <span v-if="activeTab === 'issues' && analysisTime" class="execution-time" title="Analysis time">
          ⚡ {{ analysisTime }}
        </span>
        <span v-if="activeTab === 'formatter' && formatTime" class="execution-time" title="Format time">
          ⚡ {{ formatTime }}
        </span>
        <div v-if="activeTab === 'issues'" class="header-badges">
          <button
            :class="['badge', 'linter-badge', { active: activeFilter === null || activeFilter === 'linter', inactive: activeFilter === 'analyzer' }]"
            :title="activeFilter === 'linter' ? 'Show all issues' : 'Show only linter issues'"
            @click="toggleFilter('linter')"
          >
            <span class="badge-icon">L</span>
            {{ linterCount }}
          </button>
          <button
            :class="['badge', 'analyzer-badge', { active: activeFilter === null || activeFilter === 'analyzer', inactive: activeFilter === 'linter' }]"
            :title="activeFilter === 'analyzer' ? 'Show all issues' : 'Show only analyzer issues'"
            @click="toggleFilter('analyzer')"
          >
            <span class="badge-icon">A</span>
            {{ analyzerCount }}
          </button>
        </div>
      </div>
    </div>

    <div class="output-content">
      <div v-if="isLoading && activeTab === 'issues'" class="loading">
        <div class="spinner"></div>
        <span>Analyzing...</span>
      </div>

      <div v-else-if="activeTab === 'issues' && !analyzerResults" class="placeholder">
        <div class="spinner"></div>
        <p>Preparing analysis...</p>
      </div>

      <div v-else-if="activeTab === 'issues' && displayedIssues.length === 0 && allIssues.length === 0" class="no-issues">
        <span class="success-icon">✓</span>
        <p>No issues found!</p>
      </div>

      <div v-else-if="activeTab === 'issues' && displayedIssues.length === 0 && allIssues.length > 0" class="no-issues filtered">
        <p>No {{ activeFilter }} issues to show</p>
        <button class="clear-filter" @click="activeFilter = null">Show all issues</button>
      </div>

      <div v-if="activeTab === 'issues' && displayedIssues.length > 0" class="issues-list">
        <div
          v-for="(issue, index) in displayedIssues"
          :key="index"
          :class="['issue-item', getLevelClass(issue.level)]"
          @mouseenter="handleIssueHover(issue)"
          @mouseleave="handleIssueLeave"
        >
          <div class="issue-header">
            <span :class="['issue-icon', getLevelClass(issue.level)]">
              {{ getLevelIcon(issue.level) }}
            </span>
            <template v-if="getDisplaySource(issue.source) === 'both'">
              <span class="issue-source source-linter">Linter</span>
              <span class="issue-source source-analyzer">Analyzer</span>
            </template>
            <template v-else>
              <span :class="['issue-source', getDisplaySource(issue.source) === 'analyzer' ? 'source-analyzer' : 'source-linter']">
                {{ getDisplaySource(issue.source) === 'analyzer' ? 'Analyzer' : 'Linter' }}
              </span>
            </template>
            <span v-if="issue.code" class="issue-code">{{ formatCode(issue.code) }}</span>
            <span class="issue-level">{{ issue.level }}</span>
          </div>
          <div class="issue-message">{{ issue.message }}</div>
          <div v-if="issue.annotations && issue.annotations.filter(a => a.message).length" class="issue-annotations">
            <div v-for="(ann, ai) in issue.annotations.filter(a => a.message)" :key="ai" class="issue-annotation">
              <span class="annotation-location">Line {{ ann.startLine }}:{{ ann.startColumn }}</span>
              <span class="annotation-message">{{ ann.message }}</span>
            </div>
          </div>
          <div v-if="issue.notes && issue.notes.length" class="issue-notes">
            <div v-for="(note, ni) in issue.notes" :key="ni" class="issue-note">
              {{ note }}
            </div>
          </div>
          <div v-if="issue.help" class="issue-help">
            {{ issue.help }}
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'formatter'" class="formatted-view">
        <pre class="formatted-pre"><code ref="formattedCodeRef" class="language-php"></code></pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.playground-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--vp-c-bg);
}

.output-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
}

.tabs {
  display: flex;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary {
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-divider);
}

.tab-btn:hover {
  background: var(--vp-c-bg-soft);
  border-color: #10b981;
}

.tab-btn.active {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.header-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.execution-time {
  font-size: 12px;
  font-weight: 500;
  color: var(--vp-c-text-2);
  padding: 2px 8px;
  background: var(--vp-c-bg);
  border-radius: 10px;
}

.header-badges {
  display: flex;
  gap: 8px;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 500;
  border: none;
  cursor: pointer;
  transition: opacity 0.2s, transform 0.1s;
}

.badge:hover {
  transform: scale(1.05);
}

.badge:active {
  transform: scale(0.95);
}

.badge.inactive {
  opacity: 0.4;
}

.badge-icon {
  font-weight: 700;
  font-size: 10px;
}

.linter-badge {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.linter-badge.inactive {
  background: rgba(59, 130, 246, 0.05);
}

.analyzer-badge {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
}

.analyzer-badge.inactive {
  background: rgba(168, 85, 247, 0.05);
}

.error-badge {
  background: rgba(220, 38, 38, 0.1);
  color: #dc2626;
}

.output-content {
  flex: 1;
  overflow: auto;
  padding: 0;
}

.formatted-view {
  padding: 0;
  height: 100%;
  overflow: auto;
}

.formatted-pre {
  margin: 0;
  padding: 16px;
  height: 100%;
  background: var(--vp-c-bg);
}

.formatted-pre :deep(.token.comment),
.formatted-pre :deep(.token.prolog),
.formatted-pre :deep(.token.doctype),
.formatted-pre :deep(.token.cdata) {
  color: var(--vp-c-text-3);
}

.formatted-pre :deep(.token.punctuation) {
  color: var(--vp-c-text-2);
}

.formatted-pre :deep(.token.property),
.formatted-pre :deep(.token.tag),
.formatted-pre :deep(.token.boolean),
.formatted-pre :deep(.token.number),
.formatted-pre :deep(.token.constant),
.formatted-pre :deep(.token.symbol),
.formatted-pre :deep(.token.deleted) {
  color: #e06c75;
}

.formatted-pre :deep(.token.selector),
.formatted-pre :deep(.token.attr-name),
.formatted-pre :deep(.token.string),
.formatted-pre :deep(.token.char),
.formatted-pre :deep(.token.builtin),
.formatted-pre :deep(.token.inserted) {
  color: #98c379;
}

.formatted-pre :deep(.token.operator),
.formatted-pre :deep(.token.entity),
.formatted-pre :deep(.token.url),
.formatted-pre :deep(.language-css .token.string),
.formatted-pre :deep(.style .token.string) {
  color: #56b6c2;
}

.formatted-pre :deep(.token.atrule),
.formatted-pre :deep(.token.attr-value),
.formatted-pre :deep(.token.keyword) {
  color: #c678dd;
}

.formatted-pre :deep(.token.function),
.formatted-pre :deep(.token.class-name) {
  color: #61afef;
}

.formatted-pre :deep(.token.regex),
.formatted-pre :deep(.token.important),
.formatted-pre :deep(.token.variable) {
  color: #c678dd;
}

.loading,
.placeholder,
.no-issues {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: var(--vp-c-text-2);
  gap: 12px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--vp-c-divider);
  border-top-color: var(--vp-c-brand-1);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.success-icon {
  font-size: 32px;
  color: #10b981;
}

.no-issues p {
  margin: 0;
}

.no-issues.filtered {
  color: var(--vp-c-text-3);
}

.clear-filter {
  margin-top: 8px;
  padding: 6px 12px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-size: 13px;
}

.clear-filter:hover {
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
}

.issues-list {
  padding: 0;
}

.issue-item {
  padding: 12px 16px;
  border-bottom: 1px solid var(--vp-c-divider);
  border-left: 3px solid transparent;
}

.issue-item.level-error {
  border-left-color: #dc2626;
}

.issue-item.level-warning {
  border-left-color: #d97706;
}

.issue-item.level-note,
.issue-item.level-help {
  border-left-color: var(--vp-c-text-3);
}

.issue-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.issue-icon {
  font-size: 10px;
}

.issue-icon.level-error {
  color: #dc2626;
}

.issue-icon.level-warning {
  color: #d97706;
}

.issue-icon.level-note,
.issue-icon.level-help {
  color: var(--vp-c-text-3);
}

.issue-source {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 3px;
  letter-spacing: 0.3px;
}

.issue-source.source-linter {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.issue-source.source-analyzer {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
}

.issue-code {
  font-family: 'Fira Code', monospace;
  font-size: 11px;
  color: var(--vp-c-text-2);
}

.issue-level {
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--vp-c-text-3);
}

.issue-message {
  font-size: 14px;
  color: var(--vp-c-text-1);
  line-height: 1.5;
}

.issue-annotations {
  margin-top: 8px;
}

.issue-annotation {
  font-size: 12px;
  color: var(--vp-c-text-2);
  padding: 4px 0;
  display: flex;
  gap: 8px;
}

.annotation-location {
  font-family: 'Fira Code', monospace;
  font-size: 11px;
  color: var(--vp-c-text-3);
  flex-shrink: 0;
}

.annotation-message {
  color: var(--vp-c-text-2);
}

.issue-notes {
  margin-top: 8px;
  padding-left: 12px;
  border-left: 2px solid var(--vp-c-divider);
}

.issue-note {
  font-size: 13px;
  color: var(--vp-c-text-2);
  padding: 2px 0;
}

.issue-help {
  margin-top: 8px;
  font-size: 13px;
  color: var(--vp-c-text-2);
  font-style: italic;
}
</style>
