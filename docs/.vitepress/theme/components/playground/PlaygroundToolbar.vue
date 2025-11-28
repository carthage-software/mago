<script setup>
defineProps({
  isLoading: {
    type: Boolean,
    default: false,
  },
  wasmReady: {
    type: Boolean,
    default: false,
  },
  settingsOpen: {
    type: Boolean,
    default: false,
  },
  shareSuccess: {
    type: Boolean,
    default: false,
  },
  shareError: {
    type: String,
    default: null,
  },
});

const emit = defineEmits(['format', 'share', 'toggle-settings']);
</script>

<template>
  <div class="playground-toolbar">
    <div class="toolbar-left">
      <button
        class="btn btn-secondary"
        :disabled="isLoading || !wasmReady"
        @click="emit('format')"
      >
        <span class="btn-icon">⎘</span>
        Format
      </button>
    </div>

    <div class="toolbar-right">
      <div v-if="shareSuccess" class="share-feedback success">
        Copied to clipboard!
      </div>
      <div v-else-if="shareError" class="share-feedback error">
        {{ shareError }}
      </div>

      <button
        class="btn btn-icon-only"
        title="Share"
        :disabled="isLoading"
        @click="emit('share')"
      >
        <span class="btn-icon">🔗</span>
      </button>

      <button
        :class="['btn', 'btn-icon-only', { active: settingsOpen }]"
        title="Settings"
        @click="emit('toggle-settings')"
      >
        <span class="btn-icon">⚙</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.playground-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--vp-c-bg-soft);
  border-bottom: 1px solid var(--vp-c-divider);
  gap: 8px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-divider);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--vp-c-bg-soft);
  border-color: var(--vp-c-brand-1);
}

.btn-icon-only {
  padding: 8px;
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  border: 1px solid var(--vp-c-divider);
}

.btn-icon-only:hover:not(:disabled) {
  color: var(--vp-c-text-1);
  border-color: var(--vp-c-brand-1);
}

.btn-icon-only.active {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.btn-icon {
  font-size: 14px;
}

.share-feedback {
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  animation: fadeIn 0.2s ease;
}

.share-feedback.success {
  background: #10b98120;
  color: #10b981;
}

.share-feedback.error {
  background: #e5393520;
  color: #e53935;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 480px) {
  .btn span:not(.btn-icon) {
    display: none;
  }

  .btn {
    padding: 8px 12px;
  }
}
</style>
