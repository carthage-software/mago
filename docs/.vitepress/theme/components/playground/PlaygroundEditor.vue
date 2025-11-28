<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  highlightRange: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(['update:modelValue']);

const editorRef = ref(null);
const gutterRef = ref(null);
const containerRef = ref(null);
let jar = null;
let isUpdating = false;
let Prism = null;

const lineCount = computed(() => {
  const lines = props.modelValue.split('\n').length;
  return Math.max(lines, 1);
});

const lineNumbers = computed(() => {
  return Array.from({ length: lineCount.value }, (_, i) => i + 1);
});

function isLineHighlighted(lineNum) {
  if (!props.highlightRange) return false;
  return lineNum >= props.highlightRange.startLine && lineNum <= props.highlightRange.endLine;
}

function syncScroll() {
  if (containerRef.value && gutterRef.value) {
    gutterRef.value.scrollTop = containerRef.value.scrollTop;
  }
}

onMounted(async () => {
  if (typeof window === 'undefined') return;

  const [{ CodeJar }, prismModule] = await Promise.all([
    import('codejar'),
    import('prismjs'),
  ]);

  Prism = prismModule.default;

  await import('prismjs/components/prism-markup-templating');
  await import('prismjs/components/prism-php');

  function highlight(el) {
    const code = el.textContent || '';
    if (Prism && Prism.languages.php) {
      el.innerHTML = Prism.highlight(code, Prism.languages.php, 'php');
    }
  }

  if (editorRef.value) {
    jar = CodeJar(editorRef.value, highlight, {
      tab: '    ',
      indentOn: /[(\[{]$/,
    });

    jar.updateCode(props.modelValue);

    jar.onUpdate((code) => {
      if (!isUpdating) {
        emit('update:modelValue', code);
      }
    });
  }

  if (containerRef.value) {
    containerRef.value.addEventListener('scroll', syncScroll);
  }
});

onUnmounted(() => {
  if (jar) {
    jar.destroy();
    jar = null;
  }
  if (containerRef.value) {
    containerRef.value.removeEventListener('scroll', syncScroll);
  }
});

watch(
  () => props.modelValue,
  (newValue) => {
    if (jar && jar.toString() !== newValue) {
      isUpdating = true;
      jar.updateCode(newValue);
      isUpdating = false;
    }
  }
);
</script>

<template>
  <div :class="['playground-editor', { disabled: disabled }]">
    <div class="editor-wrapper">
      <div ref="gutterRef" class="line-numbers">
        <div
          v-for="n in lineNumbers"
          :key="n"
          :class="['line-number', { highlighted: isLineHighlighted(n) }]"
        >{{ n }}</div>
      </div>
      <div ref="containerRef" class="editor-container">
        <div
          ref="editorRef"
          class="editor"
          spellcheck="false"
          :contenteditable="!disabled"
        ></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.playground-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--vp-c-bg-soft);
  transition: opacity 0.2s;
}

.playground-editor.disabled {
  opacity: 0.6;
  pointer-events: none;
}

.editor-wrapper {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.line-numbers {
  flex-shrink: 0;
  padding: 16px 0;
  background: var(--vp-c-bg-alt);
  border-right: 1px solid var(--vp-c-divider);
  text-align: right;
  user-select: none;
  overflow: hidden;
}

.line-number {
  font-family: 'Fira Code', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.6;
  padding: 0 12px 0 16px;
  color: var(--vp-c-text-3);
  min-width: 3ch;
  transition: background-color 0.15s, color 0.15s;
}

.line-number.highlighted {
  background-color: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  font-weight: 600;
}

.editor-container {
  flex: 1;
  overflow: auto;
  padding: 0;
}

.editor {
  font-family: 'Fira Code', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.6;
  padding: 16px;
  min-height: 100%;
  outline: none;
  white-space: pre-wrap;
  word-wrap: break-word;
  tab-size: 4;
}

/* Prism.js theme overrides for PHP */
.editor :deep(.token.comment),
.editor :deep(.token.prolog),
.editor :deep(.token.doctype),
.editor :deep(.token.cdata) {
  color: var(--vp-c-text-3);
}

.editor :deep(.token.punctuation) {
  color: var(--vp-c-text-2);
}

.editor :deep(.token.property),
.editor :deep(.token.tag),
.editor :deep(.token.boolean),
.editor :deep(.token.number),
.editor :deep(.token.constant),
.editor :deep(.token.symbol),
.editor :deep(.token.deleted) {
  color: #e06c75;
}

.editor :deep(.token.selector),
.editor :deep(.token.attr-name),
.editor :deep(.token.string),
.editor :deep(.token.char),
.editor :deep(.token.builtin),
.editor :deep(.token.inserted) {
  color: #98c379;
}

.editor :deep(.token.operator),
.editor :deep(.token.entity),
.editor :deep(.token.url),
.editor :deep(.language-css .token.string),
.editor :deep(.style .token.string) {
  color: #56b6c2;
}

.editor :deep(.token.atrule),
.editor :deep(.token.attr-value),
.editor :deep(.token.keyword) {
  color: #c678dd;
}

.editor :deep(.token.function),
.editor :deep(.token.class-name) {
  color: #61afef;
}

.editor :deep(.token.regex),
.editor :deep(.token.important),
.editor :deep(.token.variable) {
  color: #c678dd;
}
</style>
