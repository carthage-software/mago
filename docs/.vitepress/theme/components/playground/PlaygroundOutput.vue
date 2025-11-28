<script setup>
import { computed, ref } from 'vue';

const props = defineProps({
  results: {
    type: Object,
    default: null,
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['highlight-line']);

const activeFilter = ref(null);

const severityOrder = {
  error: 0,
  warning: 1,
  note: 2,
  help: 3,
};

const allIssues = computed(() => {
  if (!props.results) return [];
  return props.results.issues || [];
});

const displayedIssues = computed(() => {
  let issues = allIssues.value;

  if (activeFilter.value) {
    issues = issues.filter((i) => i.source === activeFilter.value);
  }

  return [...issues].sort((a, b) => {
    const aOrder = severityOrder[(a.level || 'note').toLowerCase()] ?? 3;
    const bOrder = severityOrder[(b.level || 'note').toLowerCase()] ?? 3;
    return aOrder - bOrder;
  });
});

const linterCount = computed(() => {
  return allIssues.value.filter((i) => i.source === 'linter').length;
});

const analyzerCount = computed(() => {
  return allIssues.value.filter((i) => i.source === 'analyzer').length;
});

const analysisTime = computed(() => {
  if (!props.results || props.results.analysisTimeMs === undefined) return null;
  const ms = props.results.analysisTimeMs;
  if (ms < 1) {
    return `${(ms * 1000).toFixed(0)}µs`;
  } else if (ms < 1000) {
    return `${ms.toFixed(1)}ms`;
  } else {
    return `${(ms / 1000).toFixed(2)}s`;
  }
});

function toggleFilter(source) {
  if (activeFilter.value === source) {
    activeFilter.value = null; // Toggle off
  } else {
    activeFilter.value = source;
  }
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

function getSourceLabel(source) {
  return source === 'analyzer' ? 'Analyzer' : 'Linter';
}

function getSourceClass(source) {
  return source === 'analyzer' ? 'source-analyzer' : 'source-linter';
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
</script>

<template>
  <div class="playground-output">
    <div class="output-header">
      <span class="header-title">Issues</span>
      <div class="header-right">
        <span v-if="analysisTime" class="analysis-time" title="Analysis time">
          ⚡ {{ analysisTime }}
        </span>
        <div class="header-badges">
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
      <div v-if="isLoading" class="loading">
        <div class="spinner"></div>
        <span>Analyzing...</span>
      </div>

      <div v-else-if="!results" class="placeholder">
        <div class="spinner"></div>
        <p>Preparing analysis...</p>
      </div>

      <div v-else-if="displayedIssues.length === 0 && allIssues.length === 0" class="no-issues">
        <span class="success-icon">✓</span>
        <p>No issues found!</p>
      </div>

      <div v-else-if="displayedIssues.length === 0 && allIssues.length > 0" class="no-issues filtered">
        <p>No {{ activeFilter }} issues to show</p>
        <button class="clear-filter" @click="activeFilter = null">Show all issues</button>
      </div>

      <div v-else class="issues-list">
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
            <span :class="['issue-source', getSourceClass(issue.source)]">
              {{ getSourceLabel(issue.source) }}
            </span>
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

.analysis-time {
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

.output-content {
  flex: 1;
  overflow: auto;
  padding: 0;
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
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  letter-spacing: 0.3px;
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
