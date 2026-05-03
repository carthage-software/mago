import { basicSetup, EditorView } from "codemirror";
import { EditorState, StateField, StateEffect } from "@codemirror/state";
import { keymap, Decoration, hoverTooltip } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting, indentUnit } from "@codemirror/language";
import { indentMore, indentLess } from "@codemirror/commands";
import { php } from "@codemirror/lang-php";
import { tags as t } from "@lezer/highlight";

const INDENT_UNIT = "    ";

const tabIndents = {
  key: "Tab",
  run: ({ state, dispatch }) => {
    if (state.selection.ranges.some((range) => !range.empty)) {
      return indentMore({ state, dispatch });
    }
    dispatch(
      state.update(state.replaceSelection(INDENT_UNIT), {
        scrollIntoView: true,
        userEvent: "input",
      })
    );
    return true;
  },
  shift: indentLess,
};

const MODULE_BASE = new URL(".", import.meta.url);
const assetUrl = (relative) => new URL(relative, MODULE_BASE).href;

const API_BASE_URL = "https://carthage.software/api/playground";
const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const DEBOUNCE_MS = 300;
const STORAGE_KEY = "mago-playground-state";

const I18N = (() => {
  const fallback = {
    status_loading: "Loading the analyzer (~15 MB).",
    status_formatting: "Formatting.",
    status_formatted: "Formatted.",
    status_sharing: "Sharing.",
    status_copied: "Link copied to clipboard.",
    status_share_url: "Share URL:",
    err_share: "Share failed:",
    err_format: "Format error:",
    err_analysis: "Analysis error:",
    err_load: "Failed to load analyzer:",
    no_issues_title: "No issues found.",
    no_issues_sub: "The analyzer ran clean against your code.",
    filtered_title: "All matching issues are filtered out.",
    filtered_sub: "Re-enable a filter chip above to see them.",
  };
  try {
    const raw = document.getElementById("pg-i18n")?.textContent;
    if (!raw) return fallback;
    return { ...fallback, ...JSON.parse(raw) };
  } catch {
    return fallback;
  }
})();

const DEFAULT_CODE = `<?php

declare(strict_types=1);

namespace App;

use InvalidArgumentException;

final readonly class Assert
{
    /**
     * @psalm-template T
     *
     * @psalm-param class-string<T> $class
     *
     * @psalm-assert T $value
     *
     * @throws InvalidArgumentException
     */
    public static function isInstanceOf(mixed $value, string $class, string $message = ''): void
    {
        if (!$value instanceof $class) {
            throw new InvalidArgumentException($message);
        }
    }
}

class Foo extends Bar
{
}

/** @throws InvalidArgumentException */
function example(object $foo): Foo
{
    Assert::isInstanceOf($foo, Foo::class);

    if (\\is_string($foo)) {
        var_dump('impossible!');
    }

    return $foo;;
}
`;

const ANALYZER_OPTIONS = [
  { key: "findUnusedExpressions", name: "Find unused expressions", desc: "Report expressions whose result is discarded (e.g. $a + $b;)." },
  { key: "findUnusedDefinitions", name: "Find unused definitions", desc: "Report private definitions that are never referenced." },
  { key: "findOverlyWideReturnTypes", name: "Find overly wide return types", desc: "Warn when a declared return type contains branches the body never produces." },
  { key: "analyzeDeadCode", name: "Analyse dead code", desc: "Analyse code that appears to be unreachable." },
  { key: "memoizeProperties", name: "Memoize properties", desc: "Track literal values of class properties for sharper inference." },
  { key: "allowPossiblyUndefinedArrayKeys", name: "Allow possibly undefined array keys", desc: "Deprecated. Allow accessing keys that may be missing without flagging it. Prefer strict-array-index-existence." },
  { key: "checkThrows", name: "Check @throws", desc: "Report exceptions that are not caught and not declared with @throws." },
  { key: "checkMissingOverride", name: "Check missing #[Override]", desc: "Report missing #[Override] attributes on overriding methods (PHP 8.3+)." },
  { key: "findUnusedParameters", name: "Find unused parameters", desc: "Report parameters that are never read." },
  { key: "strictListIndexChecks", name: "Strict list index checks", desc: "Require list indices to be provably non-negative." },
  { key: "strictArrayIndexExistence", name: "Strict array index existence", desc: "Treat array/list reads with non-guaranteed keys as T|null and warn. Replaces allow-possibly-undefined-array-keys = false." },
  { key: "noBooleanLiteralComparison", name: "No boolean literal comparison", desc: "Disallow direct comparisons to boolean literals like $a === true." },
  { key: "checkMissingTypeHints", name: "Check missing type hints", desc: "Report missing type hints on parameters, properties, return types." },
  { key: "checkClosureMissingTypeHints", name: "Check closure type hints", desc: "Extend the missing-type-hint check to closures." },
  { key: "checkArrowFunctionMissingTypeHints", name: "Check arrow function type hints", desc: "Extend the missing-type-hint check to arrow functions." },
  { key: "allowImplicitPipeCallableTypes", name: "Allow implicit pipe callable types", desc: "Skip the closure / arrow type-hint checks when the callable is the right operand of |>." },
  { key: "registerSuperGlobals", name: "Register superglobals", desc: "Register $_GET, $_POST, $_SERVER, etc. automatically." },
  { key: "trustExistenceChecks", name: "Trust existence checks", desc: "Narrow types based on method_exists, property_exists, function_exists, defined." },
  { key: "checkPropertyInitialization", name: "Check property initialization", desc: "Verify typed properties are initialised in a constructor or initialiser." },
  { key: "checkUseStatements", name: "Check use statements", desc: "Report use statements that import non-existent classes, functions, constants." },
  { key: "checkExperimental", name: "Check @experimental usage", desc: "Report use of @experimental APIs from non-experimental contexts." },
  { key: "checkNameCasing", name: "Check name casing", desc: "Report incorrect casing when referencing classes, functions, etc." },
  { key: "enforceClassFinality", name: "Enforce class finality", desc: "Report classes that aren't final, abstract, or @api with no children." },
  { key: "requireApiOrInternal", name: "Require @api or @internal", desc: "Require abstract classes, interfaces, and traits to be annotated." },
  { key: "allowSideEffectsInConditions", name: "Allow side effects in conditions", desc: "When off, report impure calls inside if/while/for/ternary/match conditions." },
];

const PLUGINS = [
  { id: "stdlib", name: "stdlib", desc: "Type providers for PHP built-ins (strlen, array_*, json_*, …).", defaultEnabled: true },
  { id: "psl", name: "php-standard-library", desc: "Type providers for the php-standard-library package.", defaultEnabled: false },
  { id: "flow-php", name: "flow-php/etl", desc: "Type providers for the flow-php/etl ETL framework.", defaultEnabled: false },
  { id: "psr-container", name: "PSR-11 container", desc: "Type providers for the psr/container package.", defaultEnabled: false },
];

const DEFAULT_STATE = () => ({
  code: DEFAULT_CODE,
  phpVersion: "8.4",
  analyzer: {
    findUnusedExpressions: true,
    findUnusedDefinitions: true,
    findOverlyWideReturnTypes: false,
    analyzeDeadCode: false,
    memoizeProperties: true,
    allowPossiblyUndefinedArrayKeys: true,
    checkThrows: false,
    uncheckedExceptions: [],
    uncheckedExceptionClasses: [],
    checkMissingOverride: false,
    findUnusedParameters: false,
    strictListIndexChecks: false,
    strictArrayIndexExistence: false,
    noBooleanLiteralComparison: false,
    checkMissingTypeHints: false,
    checkClosureMissingTypeHints: false,
    checkArrowFunctionMissingTypeHints: false,
    allowImplicitPipeCallableTypes: false,
    registerSuperGlobals: true,
    trustExistenceChecks: true,
    classInitializers: [],
    checkPropertyInitialization: false,
    checkUseStatements: false,
    checkExperimental: false,
    checkNameCasing: false,
    enforceClassFinality: false,
    requireApiOrInternal: false,
    allowSideEffectsInConditions: true,
    disableDefaultPlugins: false,
    plugins: [],
  },
  linter: { disabledRules: ["file-name"], integrations: [] },
  filters: { linter: true, analyzer: true },
});

const root = document.getElementById("playground");
const editorEl = document.getElementById("pg-editor");
const cursorEl = document.getElementById("pg-cursor");
const phpSelectEl = document.getElementById("pg-php-version");
const formatBtn = document.getElementById("pg-format");
const shareBtn = document.getElementById("pg-share");
const settingsToggleBtn = document.getElementById("pg-settings-toggle");
const settingsCloseBtn = document.getElementById("pg-settings-close");
const settingsEl = document.getElementById("pg-settings");
const panesEl = document.getElementById("pg-panes");
const outputEl = document.getElementById("pg-output");
const statusEl = document.getElementById("pg-status");
const timeEl = document.getElementById("pg-time");
const linterChipBtn = document.getElementById("pg-filter-linter");
const analyzerChipBtn = document.getElementById("pg-filter-analyzer");
const linterCountEl = document.getElementById("pg-count-linter");
const analyzerCountEl = document.getElementById("pg-count-analyzer");
const analyzerOptionsEl = document.getElementById("pg-analyzer-options");
const pluginsEl = document.getElementById("pg-plugins");
const integrationsEl = document.getElementById("pg-integrations");
const exceptionsGroupEl = document.getElementById("pg-exceptions");
const uncheckedInputEl = document.getElementById("pg-unchecked-exceptions");
const uncheckedClassInputEl = document.getElementById("pg-unchecked-exception-classes");
const classInitializersEl = document.getElementById("pg-class-initializers");
const rulesSearchEl = document.getElementById("pg-rules-search");
const rulesListEl = document.getElementById("pg-rules-list");
const rulesCountEl = document.getElementById("pg-rules-count");
const rulesEnableBtn = document.getElementById("pg-rules-enable");
const rulesDisableBtn = document.getElementById("pg-rules-disable");

let state = DEFAULT_STATE();
let availableRules = [];
let availableIntegrations = [];
let issues = [];
let lastAnalysisMs = null;
let view = null;
let wasm = null;
let analyzeTimer = null;
let suppressEditorUpdate = false;
let loadedFromShareHash = false;

async function loadWasm() {
  if (wasm) return wasm;
  setStatus(I18N.status_loading, null);
  const module = await import(assetUrl("mago_wasm.js"));
  await module.default({ module_or_path: assetUrl("mago_wasm_bg.wasm") });
  wasm = module;
  return wasm;
}

function buildRunSettings() {
  return {
    phpVersion: state.phpVersion,
    analyzer: state.analyzer,
    linter: {
      disabledRules: state.linter.disabledRules,
      integrations: state.linter.integrations,
    },
  };
}

async function runAnalysis() {
  if (!wasm) return;
  try {
    const start = performance.now();
    const result = wasm.run(state.code, buildRunSettings());
    lastAnalysisMs = performance.now() - start;
    issues = Array.isArray(result) ? result : [];
    renderIssues();
    applyIssueDecorations();
    clearStatus();
  } catch (error) {
    setStatus(`${I18N.err_analysis} ${error.message || error}`, "err");
  }
}

function scheduleAnalysis() {
  clearShareHashIfStale();
  if (!wasm) return;
  if (analyzeTimer) clearTimeout(analyzeTimer);
  analyzeTimer = setTimeout(() => {
    analyzeTimer = null;
    runAnalysis();
    persistStateToLocalStorage();
  }, DEBOUNCE_MS);
}

async function formatCode() {
  if (!wasm) return;
  setStatus(I18N.status_formatting, null);
  try {
    const formatted = wasm.format(state.code, state.phpVersion);
    state.code = formatted;
    if (view) {
      suppressEditorUpdate = true;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: formatted },
      });
      suppressEditorUpdate = false;
    }
    setStatus(I18N.status_formatted, "ok");
    setTimeout(() => clearStatus(), 1200);
    clearShareHashIfStale();
    runAnalysis();
    persistStateToLocalStorage();
  } catch (error) {
    setStatus(`${I18N.err_format} ${error.message || error}`, "err");
  }
}

function setStatus(text, kind) {
  statusEl.textContent = text;
  if (kind) statusEl.dataset.state = kind;
  else delete statusEl.dataset.state;
}

function clearStatus() {
  statusEl.textContent = "";
  delete statusEl.dataset.state;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const SEVERITY_RANK = { error: 0, warning: 1, note: 2, help: 3 };
const LEVEL_MARK = { error: "●", warning: "▲", note: "○", help: "○" };

function issueSortKey(issue) {
  const level = (issue.level || "note").toLowerCase();
  const annotation = (issue.annotations && issue.annotations[0]) || {};
  return {
    severity: SEVERITY_RANK[level] ?? 9,
    startLine: Number.isFinite(annotation.startLine) ? annotation.startLine : Infinity,
    startColumn: Number.isFinite(annotation.startColumn) ? annotation.startColumn : Infinity,
    endLine: Number.isFinite(annotation.endLine) ? annotation.endLine : Infinity,
    endColumn: Number.isFinite(annotation.endColumn) ? annotation.endColumn : Infinity,
    code: issue.code || "",
    message: issue.message || "",
  };
}

function compareIssues(a, b) {
  const ka = issueSortKey(a);
  const kb = issueSortKey(b);
  return (
    ka.severity - kb.severity ||
    ka.startLine - kb.startLine ||
    ka.startColumn - kb.startColumn ||
    ka.endLine - kb.endLine ||
    ka.endColumn - kb.endColumn ||
    ka.code.localeCompare(kb.code) ||
    ka.message.localeCompare(kb.message)
  );
}

function filteredIssues() {
  return issues
    .filter((issue) => {
      const src = (issue.source || "linter").toLowerCase();
      if (src === "both") return state.filters.linter || state.filters.analyzer;
      if (src === "linter") return state.filters.linter;
      if (src === "analyzer") return state.filters.analyzer;
      return true;
    })
    .slice()
    .sort(compareIssues);
}

function renderIssues() {
  const linterCount = issues.filter((i) => {
    const s = (i.source || "linter").toLowerCase();
    return s === "linter" || s === "both";
  }).length;
  const analyzerCount = issues.filter((i) => {
    const s = (i.source || "linter").toLowerCase();
    return s === "analyzer" || s === "both";
  }).length;

  linterCountEl.textContent = linterCount;
  analyzerCountEl.textContent = analyzerCount;
  linterChipBtn.setAttribute("aria-pressed", String(state.filters.linter));
  analyzerChipBtn.setAttribute("aria-pressed", String(state.filters.analyzer));

  if (lastAnalysisMs != null) {
    timeEl.hidden = false;
    timeEl.textContent = formatDuration(lastAnalysisMs);
  } else {
    timeEl.hidden = true;
  }

  const visible = filteredIssues();
  outputEl.innerHTML = "";

  if (issues.length === 0) {
    const success = document.createElement("div");
    success.className = "pg__success";
    const mark = document.createElement("div");
    mark.className = "pg__success-mark";
    mark.textContent = "✓";
    const title = document.createElement("p");
    title.className = "pg__success-text";
    title.textContent = I18N.no_issues_title;
    const sub = document.createElement("p");
    sub.className = "pg__success-sub";
    sub.textContent = I18N.no_issues_sub;
    success.append(mark, title, sub);
    outputEl.appendChild(success);
    return;
  }

  if (visible.length === 0) {
    const empty = document.createElement("div");
    empty.className = "pg__placeholder";
    const title = document.createElement("p");
    title.textContent = I18N.filtered_title;
    const sub = document.createElement("p");
    sub.className = "pg__placeholder-sub";
    sub.textContent = I18N.filtered_sub;
    empty.append(title, sub);
    outputEl.appendChild(empty);
    return;
  }

  const list = document.createElement("div");
  list.className = "pg__issue-list";
  for (const issue of visible) {
    list.appendChild(renderIssue(issue));
  }
  outputEl.appendChild(list);
}

function renderIssue(issue) {
  const wrapper = document.createElement("div");
  const level = (issue.level || "note").toLowerCase();
  wrapper.className = "pg__issue";
  wrapper.dataset.level = level;

  const annotation = issue.annotations && issue.annotations[0];

  const head = document.createElement("div");
  head.className = "pg__issue-head";
  const source = (issue.source || "linter").toLowerCase();
  const sources = source === "both" ? ["linter", "analyzer"] : [source];
  head.innerHTML = `
    <span class="pg__issue-mark">${LEVEL_MARK[level] || "○"}</span>
    ${sources.map((s) => `<span class="pg__issue-source pg__issue-source--${s}">${s}</span>`).join("")}
    ${issue.code ? `<span class="pg__issue-code">${escapeHtml(issue.code)}</span>` : ""}
    ${annotation ? `<span class="pg__issue-loc">${annotation.startLine}:${annotation.startColumn}</span>` : ""}
  `;
  wrapper.appendChild(head);

  const message = document.createElement("div");
  message.className = "pg__issue-message";
  message.textContent = issue.message || "";
  wrapper.appendChild(message);

  const annotationsWithMessages = (issue.annotations || []).filter((a) => a.message);
  if (annotationsWithMessages.length > 0) {
    const extras = document.createElement("div");
    extras.className = "pg__issue-extras";
    for (const ann of annotationsWithMessages) {
      const row = document.createElement("div");
      row.className = "pg__issue-annotation";
      row.innerHTML = `<span class="pg__issue-annotation-loc">${ann.startLine}:${ann.startColumn}</span><span>${escapeHtml(ann.message)}</span>`;
      extras.appendChild(row);
    }
    wrapper.appendChild(extras);
  }

  if (issue.notes && issue.notes.length) {
    const extras = document.createElement("div");
    extras.className = "pg__issue-extras";
    for (const note of issue.notes) {
      const row = document.createElement("div");
      row.textContent = note;
      extras.appendChild(row);
    }
    wrapper.appendChild(extras);
  }

  if (issue.help) {
    const help = document.createElement("div");
    help.className = "pg__issue-help";
    help.textContent = issue.help;
    wrapper.appendChild(help);
  }

  wrapper.addEventListener("mouseenter", () => highlightIssueRange(issue));
  wrapper.addEventListener("mouseleave", clearIssueRangeHighlight);

  return wrapper;
}

function formatDuration(ms) {
  if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

function updateCursorIndicator() {
  if (!view) {
    cursorEl.textContent = "";
    return;
  }
  const head = view.state.selection.main.head;
  const line = view.state.doc.lineAt(head);
  cursorEl.textContent = `Ln ${line.number}, Col ${head - line.from + 1}`;
}

function buildAnalyzerOptions() {
  analyzerOptionsEl.innerHTML = "";
  for (const opt of ANALYZER_OPTIONS) {
    const label = document.createElement("label");
    label.className = "pg__toggle";
    label.innerHTML = `
      <span class="pg__toggle-text">
        <span class="pg__toggle-name">${escapeHtml(opt.name)}</span>
        <span class="pg__toggle-desc">${escapeHtml(opt.desc)}</span>
      </span>
      <span class="pg__switch">
        <input type="checkbox" data-key="${opt.key}">
        <span class="pg__switch-mark"></span>
      </span>
    `;
    const input = label.querySelector("input");
    input.checked = !!state.analyzer[opt.key];
    input.addEventListener("change", (event) => {
      state.analyzer[opt.key] = event.target.checked;
      if (opt.key === "checkThrows") {
        exceptionsGroupEl.hidden = !event.target.checked;
      }
      scheduleAnalysis();
    });
    analyzerOptionsEl.appendChild(label);
  }
}

function buildPlugins() {
  pluginsEl.innerHTML = "";
  for (const plugin of PLUGINS) {
    const label = document.createElement("label");
    label.className = "pg__toggle";
    label.innerHTML = `
      <span class="pg__toggle-text">
        <span class="pg__toggle-name">${escapeHtml(plugin.name)}${plugin.defaultEnabled ? ' <span class="pg__toggle-desc" style="display:inline">(default)</span>' : ""}</span>
        <span class="pg__toggle-desc">${escapeHtml(plugin.desc)}</span>
      </span>
      <span class="pg__switch">
        <input type="checkbox" data-plugin="${plugin.id}">
        <span class="pg__switch-mark"></span>
      </span>
    `;
    const input = label.querySelector("input");
    input.checked = isPluginEnabled(plugin.id);
    input.addEventListener("change", (event) => {
      togglePlugin(plugin.id, event.target.checked);
      scheduleAnalysis();
    });
    pluginsEl.appendChild(label);
  }
}

function isPluginEnabled(id) {
  const plugin = PLUGINS.find((p) => p.id === id);
  if (!plugin) return false;
  if (state.analyzer.plugins.includes(id)) return true;
  if (state.analyzer.disableDefaultPlugins) return false;
  return !!plugin.defaultEnabled;
}

function togglePlugin(id, enabled) {
  const plugin = PLUGINS.find((p) => p.id === id);
  if (!plugin) return;
  const inList = state.analyzer.plugins.includes(id);
  if (enabled) {
    if (!inList) state.analyzer.plugins.push(id);
  } else if (inList) {
    state.analyzer.plugins = state.analyzer.plugins.filter((p) => p !== id);
  } else if (plugin.defaultEnabled && !state.analyzer.disableDefaultPlugins) {
    state.analyzer.disableDefaultPlugins = true;
    state.analyzer.plugins = PLUGINS.filter((p) => p.defaultEnabled && p.id !== id).map((p) => p.id).concat(state.analyzer.plugins);
  }
}

function buildIntegrationsList() {
  if (!integrationsEl) return;
  integrationsEl.innerHTML = "";
  if (!availableIntegrations.length) return;
  for (const integration of availableIntegrations) {
    const label = document.createElement("label");
    label.className = "pg__toggle";
    label.innerHTML = `
      <span class="pg__toggle-text">
        <span class="pg__toggle-name">${escapeHtml(integration.name)}</span>
      </span>
      <span class="pg__switch">
        <input type="checkbox" data-integration="${integration.id}">
        <span class="pg__switch-mark"></span>
      </span>
    `;
    const input = label.querySelector("input");
    input.checked = state.linter.integrations.includes(integration.id);
    input.addEventListener("change", (event) => {
      const enabled = event.target.checked;
      const inList = state.linter.integrations.includes(integration.id);
      if (enabled && !inList) {
        state.linter.integrations.push(integration.id);
      } else if (!enabled && inList) {
        state.linter.integrations = state.linter.integrations.filter((i) => i !== integration.id);
      }
      scheduleAnalysis();
    });
    integrationsEl.appendChild(label);
  }
}

function buildRulesList() {
  if (!availableRules.length) return;
  const search = (rulesSearchEl.value || "").toLowerCase().trim();
  const grouped = {};
  for (const rule of availableRules) {
    if (search) {
      const haystack = `${rule.code} ${rule.name || ""} ${rule.description || ""}`.toLowerCase();
      if (!haystack.includes(search)) continue;
    }
    const cat = rule.category || "Other";
    (grouped[cat] = grouped[cat] || []).push(rule);
  }
  const keys = Object.keys(grouped).sort();
  rulesListEl.innerHTML = "";
  if (keys.length === 0) {
    const empty = document.createElement("p");
    empty.className = "pg__hint pg__hint--center";
    empty.textContent = "No rules match the current filter.";
    rulesListEl.appendChild(empty);
    return;
  }
  for (const cat of keys) {
    const wrap = document.createElement("div");
    wrap.className = "pg__rules-category";
    const head = document.createElement("div");
    head.className = "pg__rules-category-head";
    head.textContent = humanCategory(cat);
    wrap.appendChild(head);
    grouped[cat].sort((a, b) => (a.code || "").localeCompare(b.code || ""));
    for (const rule of grouped[cat]) {
      const row = document.createElement("label");
      row.className = "pg__rule";
      const disabled = state.linter.disabledRules.includes(rule.code);
      if (disabled) row.dataset.disabled = "true";
      row.innerHTML = `
        <span class="pg__rule-text">
          <span class="pg__rule-name">${escapeHtml(rule.code)}</span>
          <span class="pg__rule-desc">${escapeHtml(rule.description || "")}</span>
        </span>
        <span class="pg__switch">
          <input type="checkbox" ${disabled ? "" : "checked"}>
          <span class="pg__switch-mark"></span>
        </span>
      `;
      const input = row.querySelector("input");
      input.addEventListener("change", (event) => {
        if (event.target.checked) {
          state.linter.disabledRules = state.linter.disabledRules.filter((c) => c !== rule.code);
          delete row.dataset.disabled;
        } else if (!state.linter.disabledRules.includes(rule.code)) {
          state.linter.disabledRules.push(rule.code);
          row.dataset.disabled = "true";
        }
        scheduleAnalysis();
      });
      wrap.appendChild(row);
    }
    rulesListEl.appendChild(wrap);
  }
  rulesCountEl.textContent = `${availableRules.length - state.linter.disabledRules.length} / ${availableRules.length}`;
}

function humanCategory(cat) {
  switch (cat) {
    case "BestPractices": return "Best practices";
    default: return cat;
  }
}

function syncListInput(input, list) {
  input.value = (list || []).join(", ");
}

function parseListInput(input) {
  return (input.value || "").split(",").map((s) => s.trim()).filter(Boolean);
}

function attachSettingsHandlers() {
  phpSelectEl.addEventListener("change", (event) => {
    state.phpVersion = event.target.value;
    scheduleAnalysis();
  });

  uncheckedInputEl.addEventListener("change", () => {
    state.analyzer.uncheckedExceptions = parseListInput(uncheckedInputEl);
    scheduleAnalysis();
  });

  uncheckedClassInputEl.addEventListener("change", () => {
    state.analyzer.uncheckedExceptionClasses = parseListInput(uncheckedClassInputEl);
    scheduleAnalysis();
  });

  classInitializersEl.addEventListener("change", () => {
    state.analyzer.classInitializers = parseListInput(classInitializersEl);
    scheduleAnalysis();
  });

  rulesSearchEl.addEventListener("input", () => buildRulesList());

  rulesEnableBtn.addEventListener("click", () => {
    state.linter.disabledRules = [];
    buildRulesList();
    scheduleAnalysis();
  });

  rulesDisableBtn.addEventListener("click", () => {
    state.linter.disabledRules = availableRules.map((r) => r.code);
    buildRulesList();
    scheduleAnalysis();
  });

  settingsToggleBtn.addEventListener("click", toggleSettings);
  settingsCloseBtn.addEventListener("click", closeSettings);
}

function toggleSettings() {
  if (settingsEl.hasAttribute("hidden")) openSettings();
  else closeSettings();
}

function openSettings() {
  settingsEl.hidden = false;
  panesEl.dataset.settings = "open";
  settingsToggleBtn.setAttribute("aria-pressed", "true");
}

function closeSettings() {
  settingsEl.hidden = true;
  delete panesEl.dataset.settings;
  settingsToggleBtn.setAttribute("aria-pressed", "false");
}

function attachFilterHandlers() {
  linterChipBtn.addEventListener("click", () => {
    state.filters.linter = !state.filters.linter;
    renderIssues();
  });
  analyzerChipBtn.addEventListener("click", () => {
    state.filters.analyzer = !state.filters.analyzer;
    renderIssues();
  });
}

function packState() {
  return {
    s: 3,
    c: state.code,
    v: state.phpVersion,
    a: state.analyzer,
    l: { d: state.linter.disabledRules, i: state.linter.integrations },
  };
}

function applyPackedState(packed) {
  if (!packed || typeof packed !== "object") return;
  if (typeof packed.c === "string") state.code = packed.c;
  if (typeof packed.v === "string") state.phpVersion = packed.v;
  if (packed.a && typeof packed.a === "object") {
    for (const key of Object.keys(state.analyzer)) {
      if (key in packed.a) state.analyzer[key] = packed.a[key];
    }
  }
  if (packed.s === 3 && packed.l && typeof packed.l === "object") {
    if (Array.isArray(packed.l.d)) state.linter.disabledRules = packed.l.d.slice();
    if (Array.isArray(packed.l.i)) state.linter.integrations = packed.l.i.slice();
  }
}

async function decompressState(encoded) {
  let base64 = encoded.replace(/-/g, "+").replace(/_/g, "/");
  while (base64.length % 4) base64 += "=";
  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return JSON.parse(await new Response(stream).text());
}

function persistStateToLocalStorage() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(packState()));
  } catch {
  }
}

function loadStateFromLocalStorage() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return false;
    applyPackedState(JSON.parse(raw));
    return true;
  } catch {
    return false;
  }
}

async function loadStateFromHash() {
  const hash = location.hash.slice(1);
  if (!hash) return false;

  if (UUID_REGEX.test(hash)) {
    try {
      const response = await fetch(`${API_BASE_URL}/share/${hash}`);
      if (response.ok) {
        const data = await response.json();
        if (data.state) {
          applyPackedState(data.state);
          loadedFromShareHash = true;
          return true;
        }
      }
    } catch {
    }
    return false;
  }

  try {
    applyPackedState(await decompressState(hash));
    loadedFromShareHash = true;
    return true;
  } catch {
    return false;
  }
}

function clearShareHashIfStale() {
  if (!loadedFromShareHash) return;
  if (location.hash) {
    history.replaceState(null, "", location.pathname);
  }
  loadedFromShareHash = false;
}

async function shareLink() {
  setStatus(I18N.status_sharing, null);
  try {
    const response = await fetch(`${API_BASE_URL}/share`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ state: packState() }),
    });
    if (!response.ok) {
      throw new Error(`status ${response.status}`);
    }
    const payload = await response.json();
    if (!payload.uuid) throw new Error("missing uuid");
    history.replaceState(null, "", `#${payload.uuid}`);
    loadedFromShareHash = true;
    const url = `${location.origin}${location.pathname}#${payload.uuid}`;
    try {
      await navigator.clipboard.writeText(url);
      setStatus(I18N.status_copied, "ok");
    } catch {
      setStatus(`${I18N.status_share_url} ${url}`, "ok");
    }
  } catch (error) {
    setStatus(`${I18N.err_share} ${error.message || error}`, "err");
  }
}

const setIssueDecorations = StateEffect.define();

const issueDecorationField = StateField.define({
  create() {
    return Decoration.none;
  },
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(setIssueDecorations)) {
        deco = effect.value;
      }
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

function annotationRange(doc, ann) {
  if (!ann.startLine || !ann.endLine) return null;
  const docLines = doc.lines;
  const sLine = Math.min(Math.max(ann.startLine | 0, 1), docLines);
  const eLine = Math.min(Math.max(ann.endLine | 0, 1), docLines);
  const startLine = doc.line(sLine);
  const endLine = doc.line(eLine);
  const fromCol = Math.max((ann.startColumn | 0) || 1, 1) - 1;
  const toCol = Math.max((ann.endColumn | 0) || 1, 1) - 1;
  const from = Math.min(startLine.from + fromCol, startLine.to);
  const to = Math.min(endLine.from + toCol, endLine.to);
  if (to <= from) return null;
  return { from, to };
}

function issuesToDecorations(view, issues) {
  const doc = view.state.doc;
  const ranges = [];
  for (const issue of issues) {
    const level = (issue.level || "note").toLowerCase();
    for (const ann of issue.annotations || []) {
      const range = annotationRange(doc, ann);
      if (!range) continue;
      ranges.push(
        Decoration.mark({ class: `pg-cm-issue pg-cm-issue--${level}` }).range(range.from, range.to),
      );
    }
  }
  return Decoration.set(ranges, true);
}

function applyIssueDecorations() {
  if (!view) return;
  view.dispatch({ effects: setIssueDecorations.of(issuesToDecorations(view, issues)) });
}

const setHoverHighlight = StateEffect.define();

const hoverHighlightField = StateField.define({
  create() {
    return Decoration.none;
  },
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(setHoverHighlight)) {
        deco = effect.value;
      }
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

function highlightIssueRange(issue) {
  if (!view) return;
  const ann = (issue.annotations || [])[0];
  if (!ann) return;
  const range = annotationRange(view.state.doc, ann);
  if (!range) return;
  const doc = view.state.doc;
  const startLine = doc.lineAt(range.from).number;
  const endLine = doc.lineAt(range.to).number;
  const lineMarks = [];
  for (let n = startLine; n <= endLine; n++) {
    lineMarks.push(Decoration.line({ class: "pg-cm-hover" }).range(doc.line(n).from));
  }
  view.dispatch({
    effects: [
      setHoverHighlight.of(Decoration.set(lineMarks, true)),
      EditorView.scrollIntoView(range.from, { y: "nearest", yMargin: 40 }),
    ],
  });
}

function clearIssueRangeHighlight() {
  if (!view) return;
  view.dispatch({ effects: setHoverHighlight.of(Decoration.none) });
}

const issueHoverTooltip = hoverTooltip((hoverView, pos) => {
  const doc = hoverView.state.doc;
  const matches = [];
  for (const issue of issues) {
    for (const ann of issue.annotations || []) {
      const range = annotationRange(doc, ann);
      if (!range) continue;
      if (pos >= range.from && pos <= range.to) {
        matches.push({ issue, range });
        break;
      }
    }
  }
  if (matches.length === 0) return null;

  const span = matches[0].range;
  return {
    pos: span.from,
    end: span.to,
    above: true,
    create() {
      const dom = document.createElement("div");
      dom.className = "pg-cm-tooltip";
      for (const { issue } of matches) {
        const row = document.createElement("div");
        const level = (issue.level || "note").toLowerCase();
        row.className = `pg-cm-tooltip__row pg-cm-tooltip__row--${level}`;
        const code = issue.code ? `[${issue.code}] ` : "";
        row.textContent = code + (issue.message || "");
        dom.appendChild(row);
      }
      return { dom };
    },
  };
});

const cmTheme = EditorView.theme({
  "&": {
    color: "var(--ink)",
    backgroundColor: "var(--bg)",
    fontFamily: "var(--mono)",
    fontSize: "0.82rem",
    height: "100%",
  },
  "&.cm-focused": { outline: "none" },

  ".cm-scroller": {
    fontFamily: "var(--mono)",
    lineHeight: "1.55",
  },
  ".cm-content": {
    color: "var(--ink)",
    caretColor: "var(--accent)",
    paddingBlock: "var(--s-3)",
  },
  ".cm-line": {
    paddingInline: "var(--s-2)",
  },

  ".cm-cursor, .cm-dropCursor": {
    borderLeftColor: "var(--accent)",
    borderLeftWidth: "2px",
  },

  "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": {
    backgroundColor: "var(--accent-soft)",
  },
  ".cm-selectionBackground": {
    backgroundColor: "var(--accent-soft)",
  },
  ".cm-content ::selection, .cm-line ::selection": {
    backgroundColor: "transparent",
    color: "inherit",
  },

  ".cm-activeLine": { backgroundColor: "var(--bg-soft)" },
  ".cm-activeLineGutter": {
    backgroundColor: "var(--bg-inset)",
    color: "var(--ink)",
  },

  ".cm-gutters": {
    backgroundColor: "var(--bg-soft)",
    color: "var(--ink-faint)",
    border: "none",
    borderRight: "1px solid var(--rule-hair)",
  },
  ".cm-lineNumbers .cm-gutterElement": {
    padding: "0 var(--s-2) 0 var(--s-3)",
    minWidth: "2.5ch",
    color: "var(--ink-faint)",
  },
  ".cm-foldGutter .cm-gutterElement": { color: "var(--ink-faint)" },
  ".cm-foldGutter .cm-gutterElement:hover": { color: "var(--ink)" },

  ".cm-matchingBracket, &.cm-focused .cm-matchingBracket": {
    backgroundColor: "var(--accent-soft)",
    outline: "1px solid var(--accent)",
    color: "inherit",
  },
  ".cm-nonmatchingBracket": {
    backgroundColor: "rgba(192, 57, 43, 0.18)",
    color: "inherit",
  },

  ".cm-tooltip": {
    backgroundColor: "var(--bg)",
    border: "1px solid var(--rule)",
    color: "var(--ink)",
    fontFamily: "var(--mono)",
    fontSize: "0.78rem",
  },
  ".cm-tooltip-arrow:before": { borderTopColor: "var(--rule)" },
  ".cm-tooltip-arrow:after": { borderTopColor: "var(--bg)" },
  ".cm-tooltip-autocomplete": {
    "& > ul > li": {
      padding: "4px 8px",
      color: "var(--ink)",
    },
    "& > ul > li[aria-selected]": {
      backgroundColor: "var(--ink)",
      color: "var(--bg)",
    },
  },
  ".cm-tooltip-section": {
    borderTopColor: "var(--rule-hair)",
  },

  ".cm-panels": {
    backgroundColor: "var(--bg-soft)",
    color: "var(--ink)",
  },
  ".cm-panels.cm-panels-top": { borderBottom: "1px solid var(--rule-hair)" },
  ".cm-panels.cm-panels-bottom": { borderTop: "1px solid var(--rule-hair)" },
  ".cm-search input, .cm-search button, .cm-search label": {
    fontFamily: "var(--mono)",
    fontSize: "0.78rem",
    color: "var(--ink)",
  },
  ".cm-search input": {
    backgroundColor: "var(--bg)",
    border: "1px solid var(--rule-hair)",
  },
});

const cmHighlightStyle = HighlightStyle.define([
  { tag: t.comment, color: "var(--syn-comment)", fontStyle: "italic" },
  { tag: t.lineComment, color: "var(--syn-comment)", fontStyle: "italic" },
  { tag: t.blockComment, color: "var(--syn-comment)", fontStyle: "italic" },
  { tag: t.docComment, color: "var(--syn-comment)", fontStyle: "italic" },

  { tag: t.keyword, color: "var(--syn-keyword)" },
  { tag: t.controlKeyword, color: "var(--syn-keyword)" },
  { tag: t.modifier, color: "var(--syn-keyword)" },
  { tag: t.operatorKeyword, color: "var(--syn-keyword)" },
  { tag: t.definitionKeyword, color: "var(--syn-keyword)" },

  { tag: t.string, color: "var(--syn-string)" },
  { tag: t.special(t.string), color: "var(--syn-string)" },
  { tag: t.regexp, color: "var(--syn-string)" },
  { tag: t.escape, color: "var(--syn-string)" },

  { tag: t.number, color: "var(--syn-number)" },
  { tag: t.bool, color: "var(--syn-number)" },
  { tag: t.null, color: "var(--syn-number)" },
  { tag: t.literal, color: "var(--syn-number)" },
  { tag: t.atom, color: "var(--syn-number)" },

  { tag: t.variableName, color: "var(--syn-variable)" },
  { tag: t.special(t.variableName), color: "var(--syn-variable)" },
  { tag: t.local(t.variableName), color: "var(--syn-variable)" },

  { tag: t.typeName, color: "var(--syn-support)" },
  { tag: t.className, color: "var(--syn-support)" },
  { tag: t.namespace, color: "var(--syn-support)" },

  { tag: t.function(t.variableName), color: "var(--syn-entity)" },
  { tag: t.function(t.propertyName), color: "var(--syn-entity)" },
  { tag: t.propertyName, color: "var(--syn-entity)" },

  { tag: t.operator, color: "var(--syn-keyword)" },
  { tag: t.derefOperator, color: "var(--syn-punctuation)" },

  { tag: t.punctuation, color: "var(--syn-punctuation)" },
  { tag: t.bracket, color: "var(--syn-punctuation)" },
  { tag: t.brace, color: "var(--syn-punctuation)" },
  { tag: t.paren, color: "var(--syn-punctuation)" },
  { tag: t.squareBracket, color: "var(--syn-punctuation)" },
  { tag: t.angleBracket, color: "var(--syn-punctuation)" },
  { tag: t.separator, color: "var(--syn-punctuation)" },

  { tag: t.tagName, color: "var(--syn-keyword)" },
  { tag: t.attributeName, color: "var(--syn-keyword)" },
  { tag: t.attributeValue, color: "var(--syn-string)" },
  { tag: t.meta, color: "var(--syn-comment)" },
  { tag: t.processingInstruction, color: "var(--syn-comment)" },

  { tag: t.heading, color: "var(--syn-keyword)", fontWeight: "600" },
  { tag: t.strong, fontWeight: "600" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.link, color: "var(--syn-string)" },
  { tag: t.url, color: "var(--syn-string)" },
  { tag: t.invalid, color: "#c0392b", textDecoration: "underline wavy" },
]);

function attachEditor() {
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged && !suppressEditorUpdate) {
      state.code = update.state.doc.toString();
      scheduleAnalysis();
    }
    if (update.selectionSet || update.docChanged) {
      updateCursorIndicator();
    }
  });

  const startState = EditorState.create({
    doc: state.code,
    extensions: [
      basicSetup,
      cmTheme,
      syntaxHighlighting(cmHighlightStyle),
      indentUnit.of(INDENT_UNIT),
      EditorState.tabSize.of(4),
      keymap.of([tabIndents]),
      php(),
      EditorView.lineWrapping,
      issueDecorationField,
      hoverHighlightField,
      issueHoverTooltip,
      updateListener,
    ],
  });

  view = new EditorView({ state: startState, parent: editorEl });
  updateCursorIndicator();
}

function attachShortcuts() {
  window.addEventListener("keydown", (event) => {
    const meta = event.metaKey || event.ctrlKey;
    if (!meta) return;
    if (event.shiftKey && (event.key === "f" || event.key === "F")) {
      event.preventDefault();
      formatCode();
    } else if (event.key === "s" || event.key === "S") {
      event.preventDefault();
      shareLink();
    } else if (event.key === ",") {
      event.preventDefault();
      toggleSettings();
    }
  });
}

async function boot() {
  if (!root) return;

  state = DEFAULT_STATE();

  const hadHash = await loadStateFromHash();
  if (!hadHash) {
    loadStateFromLocalStorage();
  }

  phpSelectEl.value = state.phpVersion;
  syncListInput(uncheckedInputEl, state.analyzer.uncheckedExceptions);
  syncListInput(uncheckedClassInputEl, state.analyzer.uncheckedExceptionClasses);
  syncListInput(classInitializersEl, state.analyzer.classInitializers);
  exceptionsGroupEl.hidden = !state.analyzer.checkThrows;

  buildAnalyzerOptions();
  buildPlugins();
  attachSettingsHandlers();
  attachFilterHandlers();
  attachShortcuts();

  formatBtn.addEventListener("click", formatCode);
  shareBtn.addEventListener("click", shareLink);

  await attachEditor();

  try {
    await loadWasm();
    try {
      const rules = wasm.getRules();
      availableRules = Array.isArray(rules) ? rules : [];
      try {
        const integrations = wasm.getIntegrations();
        availableIntegrations = Array.isArray(integrations) ? integrations : [];
      } catch (error) {
        console.warn("getIntegrations failed:", error);
        availableIntegrations = [];
      }
      buildRulesList();
      buildIntegrationsList();
    } catch (error) {
      console.warn("getRules failed:", error);
    }
    runAnalysis();
  } catch (error) {
    setStatus(`${I18N.err_load} ${error.message || error}`, "err");
  }
}

boot();
