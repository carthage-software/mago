import { reactive, readonly, provide, inject } from "vue";

const PLAYGROUND_KEY = Symbol("playground");

const DEFAULT_CODE = `<?php

/**
 * @return list<string>
 */
function take_int(int $i) {
    return [$i, "hello"];
}

$data = ["some text", 5];
take_int($data[0]);

$condition = rand(0, 5);
if ($condition) {
  /** @psalm-trace $condition */
  echo "Condition is truthy!";
} elseif ($condition) { // @mago-expect lint:no-else-clause
}
`;

export function createPlaygroundState(initialCode = DEFAULT_CODE) {
  const state = reactive({
    code: initialCode,
    settings: {
      phpVersion: "8.4",
      analyzer: {
        findUnusedExpressions: true,
        findUnusedDefinitions: true,
        analyzeDeadCode: false,
        memoizeProperties: true,
        allowPossiblyUndefinedArrayKeys: true,
        checkThrows: false,
        uncheckedExceptions: [],
        uncheckedExceptionClasses: [],
        checkMissingOverride: false,
        findUnusedParameters: false,
        strictListIndexChecks: false,
        noBooleanLiteralComparison: false,
        checkMissingTypeHints: false,
        checkClosureMissingTypeHints: false,
        checkArrowFunctionMissingTypeHints: false,
        registerSuperGlobals: true,
        trustExistenceChecks: true,
        checkPropertyInitialization: false,
        checkUseStatements: false,
        disableDefaultPlugins: false,
        plugins: [],
      },
      linter: {
        disabledRules: ["file-name"],
      },
    },
    results: null,
    isLoading: false,
    wasmReady: false,
    activeTab: "linter",
    settingsOpen: false,
    availableRules: [],
  });

  return {
    state,
    readonlyState: readonly(state),

    setCode(code) {
      state.code = code;
    },

    setPhpVersion(version) {
      state.settings.phpVersion = version;
    },

    setAnalyzerSetting(key, value) {
      if (key in state.settings.analyzer) {
        state.settings.analyzer[key] = value;
      }
    },

    setLinterDisabledRules(rules) {
      state.settings.linter.disabledRules = rules;
    },

    toggleLinterRule(ruleCode) {
      const idx = state.settings.linter.disabledRules.indexOf(ruleCode);
      if (idx >= 0) {
        state.settings.linter.disabledRules.splice(idx, 1);
      } else {
        state.settings.linter.disabledRules.push(ruleCode);
      }
    },

    setAvailableRules(rules) {
      state.availableRules = rules;
    },

    setResults(results) {
      state.results = results;
    },

    setLoading(loading) {
      state.isLoading = loading;
    },

    setWasmReady(ready) {
      state.wasmReady = ready;
    },

    setActiveTab(tab) {
      state.activeTab = tab;
    },

    toggleSettings() {
      state.settingsOpen = !state.settingsOpen;
    },

    closeSettings() {
      state.settingsOpen = false;
    },

    restoreState(data) {
      if (data.c) state.code = data.c;
      if (data.v) state.settings.phpVersion = data.v;
      if (data.a) {
        for (const key of Object.keys(state.settings.analyzer)) {
          if (key in data.a) {
            state.settings.analyzer[key] = data.a[key];
          }
        }
      }

      if (data.l && data.l.d) {
        state.settings.linter.disabledRules = data.l.d;
      }

      if (data.t) state.activeTab = data.t;
    },

    getShareableState() {
      return {
        c: state.code,
        v: state.settings.phpVersion,
        a: state.settings.analyzer,
        l: { d: state.settings.linter.disabledRules },
        t: state.activeTab,
      };
    },
  };
}

export function providePlayground(store) {
  provide(PLAYGROUND_KEY, store);
}

export function usePlayground() {
  const store = inject(PLAYGROUND_KEY);
  if (!store) {
    throw new Error("usePlayground must be used within a playground provider");
  }
  return store;
}
