<script setup>
import { computed, ref, watch } from 'vue';

const props = defineProps({
  settings: {
    type: Object,
    required: true,
  },
  availableRules: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(['update:settings', 'toggle-rule', 'close']);

// Local state for exception inputs
const uncheckedExceptionsInput = ref(
  (props.settings.analyzer?.uncheckedExceptions || []).join(', ')
);
const uncheckedExceptionClassesInput = ref(
  (props.settings.analyzer?.uncheckedExceptionClasses || []).join(', ')
);

// Watch for external changes to settings
watch(() => props.settings.analyzer?.uncheckedExceptions, (newVal) => {
  uncheckedExceptionsInput.value = (newVal || []).join(', ');
});
watch(() => props.settings.analyzer?.uncheckedExceptionClasses, (newVal) => {
  uncheckedExceptionClassesInput.value = (newVal || []).join(', ');
});

const phpVersions = [
  { value: '8.0', label: 'PHP 8.0' },
  { value: '8.1', label: 'PHP 8.1' },
  { value: '8.2', label: 'PHP 8.2' },
  { value: '8.3', label: 'PHP 8.3' },
  { value: '8.4', label: 'PHP 8.4' },
  { value: '8.5', label: 'PHP 8.5' },
  { value: '8.6', label: 'PHP 8.6 (dev)' },
];

// Available analyzer plugins
const availablePlugins = [
  {
    id: 'stdlib',
    name: 'PHP Standard Library',
    description: 'Type providers for PHP built-in functions (strlen, array_*, json_*, etc.)',
    defaultEnabled: true,
  },
  {
    id: 'psl',
    name: 'PSL (PHP Standard Library)',
    description: 'Type providers for azjezz/psl package',
    defaultEnabled: false,
  },
  {
    id: 'flow-php',
    name: 'Flow-PHP',
    description: 'Type providers for flow-php/etl package',
    defaultEnabled: false,
  },
  {
    id: 'psr-container',
    name: 'PSR-11 Container',
    description: 'Type providers for psr/container package',
    defaultEnabled: false,
  },
  {
    id: 'laravel',
    name: 'Laravel',
    description: 'Type providers and analysis support for the Laravel framework',
    defaultEnabled: false,
  },
];

const analyzerOptions = [
  {
    key: 'findUnusedExpressions',
    label: 'Find unused expressions',
    description: 'Report expressions whose results are not used (e.g., $a + $b;)',
  },
  {
    key: 'findUnusedDefinitions',
    label: 'Find unused definitions',
    description: 'Report unused definitions like private methods never called',
  },
  {
    key: 'analyzeDeadCode',
    label: 'Analyze dead code',
    description: 'Analyze code that appears to be unreachable',
  },
  {
    key: 'memoizeProperties',
    label: 'Memoize properties',
    description: 'Track literal values of class properties for better type inference',
  },
  {
    key: 'allowPossiblyUndefinedArrayKeys',
    label: 'Allow possibly undefined array keys',
    description: 'Allow accessing array keys that may not be defined without error',
  },
  {
    key: 'checkThrows',
    label: 'Check @throws annotations',
    description: 'Report unhandled thrown exceptions not caught or documented',
  },
  {
    key: 'checkMissingOverride',
    label: 'Check missing #[Override]',
    description: 'Report missing #[Override] attributes on overriding methods (PHP 8.3+)',
  },
  {
    key: 'findUnusedParameters',
    label: 'Find unused parameters',
    description: 'Report function/method parameters that are never used',
  },
  {
    key: 'strictListIndexChecks',
    label: 'Strict list index checks',
    description: 'Require list indices to be provably non-negative',
  },
  {
    key: 'noBooleanLiteralComparison',
    label: 'Allow boolean literal comparison',
    description: 'Disable warnings for comparisons to true/false literals',
  },
  {
    key: 'enforceClassFinality',
    label: 'Enforce class finality',
    description: 'Report classes that are not marked final but have no subclasses',
  },
  {
    key: 'checkMissingTypeHints',
    label: 'Check missing type hints',
    description: 'Report missing type hints on parameters, properties, and returns',
  },
  {
    key: 'checkClosureMissingTypeHints',
    label: 'Check closure type hints',
    description: 'Also check closures for missing type hints (requires above)',
  },
  {
    key: 'checkArrowFunctionMissingTypeHints',
    label: 'Check arrow function type hints',
    description: 'Also check arrow functions for missing type hints (requires above)',
  },
  {
    key: 'registerSuperGlobals',
    label: 'Register superglobals',
    description: 'Make $_GET, $_POST, $_SERVER etc. available without global keyword',
  },
  {
    key: 'trustExistenceChecks',
    label: 'Trust existence checks',
    description: 'Narrow types based on method_exists(), property_exists(), function_exists(), and defined()',
  },
  {
    key: 'checkPropertyInitialization',
    label: 'Check property initialization',
    description: 'Report uninitialized typed properties that may be accessed before assignment',
  },
  {
    key: 'checkUseStatements',
    label: 'Check use statements',
    description: 'Report use statements that import non-existent classes, functions, or constants',
  }
];

const rulesByCategory = computed(() => {
  const grouped = {};
  for (const rule of props.availableRules) {
    if (!grouped[rule.category]) {
      grouped[rule.category] = [];
    }
    grouped[rule.category].push(rule);
  }
  for (const cat of Object.keys(grouped)) {
    grouped[cat].sort((a, b) => a.name.localeCompare(b.name));
  }
  return grouped;
});

const categories = computed(() => Object.keys(rulesByCategory.value).sort());

const ruleSearch = ref('');
const filteredRulesByCategory = computed(() => {
  const search = ruleSearch.value.toLowerCase().trim();
  if (!search) return rulesByCategory.value;

  const filtered = {};
  for (const [cat, rules] of Object.entries(rulesByCategory.value)) {
    const matching = rules.filter(
      (r) =>
        r.name.toLowerCase().includes(search) ||
        r.code.toLowerCase().includes(search) ||
        r.description.toLowerCase().includes(search)
    );
    if (matching.length > 0) {
      filtered[cat] = matching;
    }
  }
  return filtered;
});

const filteredCategories = computed(() => Object.keys(filteredRulesByCategory.value).sort());

function updatePhpVersion(event) {
  emit('update:settings', {
    ...props.settings,
    phpVersion: event.target.value,
  });
}

function updateAnalyzerSetting(key, event) {
  emit('update:settings', {
    ...props.settings,
    analyzer: {
      ...props.settings.analyzer,
      [key]: event.target.checked,
    },
  });
}

function isRuleDisabled(ruleCode) {
  return props.settings.linter?.disabledRules?.includes(ruleCode) ?? false;
}

function toggleRule(ruleCode) {
  emit('toggle-rule', ruleCode);
}

function enableAllRules() {
  emit('update:settings', {
    ...props.settings,
    linter: {
      ...props.settings.linter,
      disabledRules: [],
    },
  });
}

function disableAllRules() {
  const allCodes = props.availableRules.map((r) => r.code);
  emit('update:settings', {
    ...props.settings,
    linter: {
      ...props.settings.linter,
      disabledRules: allCodes,
    },
  });
}

function updateUncheckedExceptions() {
  const exceptions = uncheckedExceptionsInput.value
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  emit('update:settings', {
    ...props.settings,
    analyzer: {
      ...props.settings.analyzer,
      uncheckedExceptions: exceptions,
    },
  });
}

function updateUncheckedExceptionClasses() {
  const exceptions = uncheckedExceptionClassesInput.value
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  emit('update:settings', {
    ...props.settings,
    analyzer: {
      ...props.settings.analyzer,
      uncheckedExceptionClasses: exceptions,
    },
  });
}

function isPluginEnabled(pluginId) {
  const disableDefaults = props.settings.analyzer?.disableDefaultPlugins ?? false;
  const plugins = props.settings.analyzer?.plugins ?? [];
  const plugin = availablePlugins.find(p => p.id === pluginId);

  if (!plugin) return false;

  // If explicitly in plugins list, it's enabled
  if (plugins.includes(pluginId)) return true;

  // If defaults are disabled, only explicitly listed plugins are enabled
  if (disableDefaults) return false;

  // Otherwise, use the plugin's default state
  return plugin.defaultEnabled;
}

function togglePlugin(pluginId) {
  const disableDefaults = props.settings.analyzer?.disableDefaultPlugins ?? false;
  const plugins = [...(props.settings.analyzer?.plugins ?? [])];
  const plugin = availablePlugins.find(p => p.id === pluginId);

  if (!plugin) return;

  const isCurrentlyEnabled = isPluginEnabled(pluginId);
  const isInList = plugins.includes(pluginId);

  if (isCurrentlyEnabled) {
    // Disable the plugin
    if (isInList) {
      // Remove from explicit list
      const idx = plugins.indexOf(pluginId);
      plugins.splice(idx, 1);
    } else if (plugin.defaultEnabled && !disableDefaults) {
      // It was enabled by default, need to disable defaults and add all other default plugins
      emit('update:settings', {
        ...props.settings,
        analyzer: {
          ...props.settings.analyzer,
          disableDefaultPlugins: true,
          plugins: availablePlugins
            .filter(p => p.defaultEnabled && p.id !== pluginId)
            .map(p => p.id)
            .concat(plugins),
        },
      });
      return;
    }
  } else {
    // Enable the plugin
    if (!isInList) {
      plugins.push(pluginId);
    }
  }

  emit('update:settings', {
    ...props.settings,
    analyzer: {
      ...props.settings.analyzer,
      plugins,
    },
  });
}

function updateDisableDefaultPlugins(event) {
  const disableDefaults = event.target.checked;
  let plugins = [...(props.settings.analyzer?.plugins ?? [])];

  if (disableDefaults) {
    // When disabling defaults, add all currently-enabled default plugins to explicit list
    for (const plugin of availablePlugins) {
      if (plugin.defaultEnabled && !plugins.includes(plugin.id)) {
        plugins.push(plugin.id);
      }
    }
  }

  emit('update:settings', {
    ...props.settings,
    analyzer: {
      ...props.settings.analyzer,
      disableDefaultPlugins: disableDefaults,
      plugins,
    },
  });
}
</script>

<template>
  <div class="settings-overlay">
    <div class="settings-container">
      <div class="settings-header">
        <h2>Settings</h2>
        <button class="close-btn" @click="emit('close')" title="Close settings">
          <span>×</span>
        </button>
      </div>

      <div class="settings-content">
        <!-- PHP Version -->
        <section class="settings-section">
          <h3>PHP Version</h3>
          <div class="php-version-selector">
            <label
              v-for="version in phpVersions"
              :key="version.value"
              :class="['version-option', { selected: settings.phpVersion === version.value }]"
            >
              <input
                type="radio"
                name="phpVersion"
                :value="version.value"
                :checked="settings.phpVersion === version.value"
                @change="updatePhpVersion"
              />
              <span class="version-label">{{ version.label }}</span>
            </label>
          </div>
        </section>

        <!-- Analyzer Options -->
        <section class="settings-section">
          <h3>Analyzer Options</h3>
          <div class="options-grid">
            <label
              v-for="option in analyzerOptions"
              :key="option.key"
              class="option-item"
            >
              <div class="option-checkbox">
                <input
                  type="checkbox"
                  :checked="settings.analyzer[option.key]"
                  @change="updateAnalyzerSetting(option.key, $event)"
                />
                <span class="checkmark"></span>
              </div>
              <div class="option-content">
                <span class="option-label">{{ option.label }}</span>
                <span class="option-description">{{ option.description }}</span>
              </div>
            </label>
          </div>

          <!-- Exception filters (only visible when checkThrows is enabled) -->
          <div v-if="settings.analyzer?.checkThrows" class="exception-filters">
            <div class="exception-input-group">
              <label class="exception-label">
                <span class="label-text">Unchecked Exceptions</span>
                <span class="label-hint">Ignore these exceptions and all their subclasses (comma-separated)</span>
              </label>
              <input
                v-model="uncheckedExceptionsInput"
                type="text"
                class="exception-input"
                placeholder="e.g., LogicException, Psl\Type\Exception\ExceptionInterface"
                @blur="updateUncheckedExceptions"
                @keyup.enter="updateUncheckedExceptions"
              />
            </div>

            <div class="exception-input-group">
              <label class="exception-label">
                <span class="label-text">Unchecked Exception Classes</span>
                <span class="label-hint">Ignore only these exact classes, not their subclasses (comma-separated)</span>
              </label>
              <input
                v-model="uncheckedExceptionClassesInput"
                type="text"
                class="exception-input"
                placeholder="e.g., Psl\File\Exception\FileNotFoundException"
                @blur="updateUncheckedExceptionClasses"
                @keyup.enter="updateUncheckedExceptionClasses"
              />
            </div>
          </div>
        </section>

        <!-- Analyzer Plugins -->
        <section class="settings-section">
          <h3>Analyzer Plugins</h3>
          <p class="section-description">
            Plugins provide type information for library functions.
          </p>
          <div class="plugins-list">
            <label
              v-for="plugin in availablePlugins"
              :key="plugin.id"
              :class="['plugin-item', { enabled: isPluginEnabled(plugin.id) }]"
            >
              <div class="plugin-checkbox">
                <input
                  type="checkbox"
                  :checked="isPluginEnabled(plugin.id)"
                  @change="togglePlugin(plugin.id)"
                />
                <span class="checkmark"></span>
              </div>
              <div class="plugin-content">
                <div class="plugin-header">
                  <span class="plugin-name">{{ plugin.name }}</span>
                  <span v-if="plugin.defaultEnabled" class="plugin-badge">Default</span>
                </div>
                <span class="plugin-description">{{ plugin.description }}</span>
              </div>
            </label>
          </div>
        </section>

        <!-- Linter Rules -->
        <section class="settings-section linter-section">
          <div class="section-header">
            <h3>Linter Rules</h3>
            <div class="bulk-actions">
              <button class="action-btn" @click="enableAllRules">Enable All</button>
              <button class="action-btn" @click="disableAllRules">Disable All</button>
            </div>
          </div>

          <div class="rule-search">
            <input
              v-model="ruleSearch"
              type="text"
              placeholder="Search rules..."
              class="search-input"
            />
          </div>

          <div class="rules-container">
            <div v-if="filteredCategories.length === 0" class="no-rules">
              No rules match your search.
            </div>

            <div
              v-for="category in filteredCategories"
              :key="category"
              class="rule-category"
            >
              <h4 class="category-title">{{ category }}</h4>
              <div class="rules-list">
                <label
                  v-for="rule in filteredRulesByCategory[category]"
                  :key="rule.code"
                  :class="['rule-item', { disabled: isRuleDisabled(rule.code) }]"
                >
                  <div class="rule-checkbox">
                    <input
                      type="checkbox"
                      :checked="!isRuleDisabled(rule.code)"
                      @change="toggleRule(rule.code)"
                    />
                    <span class="checkmark"></span>
                  </div>
                  <div class="rule-content">
                    <div class="rule-header">
                      <span class="rule-name">{{ rule.name }}</span>
                      <code class="rule-code">{{ rule.code }}</code>
                    </div>
                    <span class="rule-description">{{ rule.description }}</span>
                  </div>
                </label>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: absolute;
  inset: 0;
  z-index: 50;
  background: var(--vp-c-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.settings-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
  flex-shrink: 0;
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.close-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--vp-c-text-2);
  font-size: 24px;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.settings-section {
  margin-bottom: 32px;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.settings-section h3 {
  margin: 0 0 16px;
  font-size: 14px;
  font-weight: 600;
  color: var(--vp-c-text-1);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Section Description */
.section-description {
  margin: -8px 0 16px;
  font-size: 13px;
  color: var(--vp-c-text-3);
}

/* Plugins List */
.plugins-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.plugin-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg);
  cursor: pointer;
  transition: all 0.15s;
}

.plugin-item:hover {
  border-color: var(--vp-c-brand-1);
  background: var(--vp-c-bg-soft);
}

.plugin-item.enabled {
  border-color: var(--vp-c-brand-1);
  background: color-mix(in srgb, var(--vp-c-brand-1) 5%, var(--vp-c-bg));
}

.plugin-checkbox {
  position: relative;
  flex-shrink: 0;
  margin-top: 2px;
}

.plugin-checkbox input {
  position: absolute;
  opacity: 0;
  cursor: pointer;
}

.plugin-checkbox .checkmark {
  display: block;
  width: 18px;
  height: 18px;
  border: 2px solid var(--vp-c-divider);
  border-radius: 4px;
  background: var(--vp-c-bg);
  transition: all 0.15s;
}

.plugin-checkbox input:checked + .checkmark {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.plugin-checkbox input:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 6px;
  top: 2px;
  width: 5px;
  height: 10px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.plugin-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.plugin-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--vp-c-text-1);
}

.plugin-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
}

.plugin-description {
  font-size: 12px;
  color: var(--vp-c-text-3);
  line-height: 1.4;
}

/* PHP Version Selector */
.php-version-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.version-option {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.version-option input {
  display: none;
}

.version-label {
  padding: 8px 16px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  font-size: 14px;
  color: var(--vp-c-text-2);
  background: var(--vp-c-bg);
  transition: all 0.15s;
}

.version-option:hover .version-label {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-text-1);
}

.version-option.selected .version-label {
  background: var(--vp-c-brand-soft);
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  font-weight: 500;
}

/* Analyzer Options Grid */
.options-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
}

.option-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.option-item:hover {
  border-color: var(--vp-c-brand-1);
}

.option-checkbox {
  position: relative;
  flex-shrink: 0;
}

.option-checkbox input {
  position: absolute;
  opacity: 0;
  cursor: pointer;
}

.option-checkbox .checkmark {
  display: block;
  width: 18px;
  height: 18px;
  border: 2px solid var(--vp-c-divider);
  border-radius: 4px;
  background: var(--vp-c-bg);
  transition: all 0.15s;
}

.option-checkbox input:checked + .checkmark {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.option-checkbox input:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 6px;
  top: 2px;
  width: 5px;
  height: 10px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.option-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--vp-c-text-1);
}

.option-description {
  font-size: 12px;
  color: var(--vp-c-text-3);
  line-height: 1.4;
}

/* Exception Filters */
.exception-filters {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--vp-c-divider);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.exception-input-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.exception-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.exception-label .label-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--vp-c-text-1);
}

.exception-label .label-hint {
  font-size: 12px;
  color: var(--vp-c-text-3);
}

.exception-input {
  width: 100%;
  padding: 10px 14px;
  font-size: 13px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  outline: none;
  transition: border-color 0.15s;
  font-family: 'Fira Code', monospace;
}

.exception-input:focus {
  border-color: var(--vp-c-brand-1);
}

.exception-input::placeholder {
  color: var(--vp-c-text-3);
  font-family: inherit;
}

/* Linter Section */
.linter-section {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.section-header h3 {
  margin: 0;
}

.bulk-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-text-1);
}

.rule-search {
  margin-bottom: 16px;
}

.search-input {
  width: 100%;
  padding: 10px 14px;
  font-size: 14px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus {
  border-color: var(--vp-c-brand-1);
}

.search-input::placeholder {
  color: var(--vp-c-text-3);
}

.rules-container {
  flex: 1;
  overflow-y: auto;
}

.no-rules {
  text-align: center;
  padding: 32px;
  color: var(--vp-c-text-3);
}

.rule-category {
  margin-bottom: 24px;
}

.rule-category:last-child {
  margin-bottom: 0;
}

.category-title {
  margin: 0 0 12px;
  font-size: 13px;
  font-weight: 600;
  color: var(--vp-c-text-2);
  padding-bottom: 8px;
  border-bottom: 1px solid var(--vp-c-divider);
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rule-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.rule-item:hover {
  border-color: var(--vp-c-brand-1);
}

.rule-item.disabled {
  opacity: 0.6;
}

.rule-checkbox {
  position: relative;
  flex-shrink: 0;
  margin-top: 2px;
}

.rule-checkbox input {
  position: absolute;
  opacity: 0;
  cursor: pointer;
}

.rule-checkbox .checkmark {
  display: block;
  width: 16px;
  height: 16px;
  border: 2px solid var(--vp-c-divider);
  border-radius: 3px;
  background: var(--vp-c-bg);
  transition: all 0.15s;
}

.rule-checkbox input:checked + .checkmark {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.rule-checkbox input:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 1px;
  width: 4px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.rule-content {
  flex: 1;
  min-width: 0;
}

.rule-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
  flex-wrap: wrap;
}

.rule-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--vp-c-text-1);
}

.rule-code {
  font-size: 11px;
  padding: 2px 6px;
  background: var(--vp-c-bg);
  border-radius: 4px;
  color: var(--vp-c-text-3);
  font-family: 'Fira Code', monospace;
}

.rule-description {
  font-size: 12px;
  color: var(--vp-c-text-3);
  line-height: 1.4;
}

@media (max-width: 768px) {
  .settings-content {
    padding: 16px;
  }

  .options-grid {
    grid-template-columns: 1fr;
  }

  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .php-version-selector {
    flex-direction: column;
  }

  .version-label {
    width: 100%;
    text-align: center;
  }
}
</style>
