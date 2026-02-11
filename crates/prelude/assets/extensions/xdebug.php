<?php

function xdebug_info(string $category = 'null') {}

function config_get_hash(): array {}

function xdebug_get_stack_depth(): int {}

function xdebug_get_function_stack(array $options = []): array {}

/**
 * @return void
 */
function xdebug_print_function_stack(string $message = 'user triggered', int $options = 0) {}

/**
 * @return array
 */
function xdebug_get_declared_vars(): array {}

/**
 * @return mixed
 */
function xdebug_call_file(int $depth = 2) {}

/**
 * @return mixed
 */
function xdebug_call_class(int $depth = 2) {}

/**
 * @return mixed
 */
function xdebug_call_function(int $depth = 2) {}

/**
 * @return mixed
 */
function xdebug_call_line(int $depth = 2) {}

/**
 * @param list<non-empty-string> $listOfFunctionsToMonitor
 *
 * @return void
 */
function xdebug_start_function_monitor(array $listOfFunctionsToMonitor) {}

/**
 * @return void
 */
function xdebug_stop_function_monitor() {}

function xdebug_get_monitored_functions(): array {}

/**
 * @return void
 */
function xdebug_var_dump(mixed ...$variable) {}

/**
 * @param non-empty-string ...$varname
 *
 * @return void
 */
function xdebug_debug_zval(string ...$varname) {}

/**
 * @param non-empty-string ...$varname
 *
 * @return void
 */
function xdebug_debug_zval_stdout(string ...$varname) {}

/**
 * @return void
 */
function xdebug_enable() {}

/**
 * @return void
 */
function xdebug_disable() {}

/**
 * @return bool
 */
function xdebug_is_enabled() {}

/**
 * @return void
 */
function xdebug_start_error_collection() {}

/**
 * @return void
 */
function xdebug_stop_error_collection() {}

function xdebug_get_collected_errors(bool $emptyList = false): array {}

function xdebug_break(): bool {}

function xdebug_start_trace(?string $traceFile = null, int $options = 0): ?string {}

function xdebug_stop_trace(): string {}

/**
 * @return string|null
 */
function xdebug_get_tracefile_name() {}

/**
 * @return string|false
 */
function xdebug_get_profiler_filename() {}

/**
 * @return bool
 */
function xdebug_dump_aggr_profiling_data($prefix = null) {}

/**
 * @return bool
 */
function xdebug_clear_aggr_profiling_data() {}

/**
 * @return int
 */
function xdebug_memory_usage(): int {}

/**
 * @return int
 */
function xdebug_peak_memory_usage(): int {}

/**
 * @return float
 */
function xdebug_time_index(): float {}

/**
 * @return void
 */
function xdebug_start_code_coverage(int $options = 0) {}

/**
 * @return void
 */
function xdebug_stop_code_coverage(bool $cleanUp = true) {}

/**
 * @return bool
 */
function xdebug_code_coverage_started(): bool {}

/**
 * @return array
 */
function xdebug_get_code_coverage(): array {}

/**
 * @return int
 */
function xdebug_get_function_count(): int {}

/**
 * @return void
 */
function xdebug_dump_superglobals() {}

/**
 * @return array
 */
function xdebug_get_headers(): array {}

function xdebug_get_formatted_function_stack() {}

/**
 * @return bool
 */
function xdebug_is_debugger_active(): bool {}

function xdebug_start_gcstats(?string $gcstatsFile = null) {}

function xdebug_stop_gcstats(): string {}

function xdebug_get_gcstats_filename() {}

function xdebug_get_gc_run_count(): int {}

function xdebug_get_gc_total_collected_roots(): int {}

/**
 * @return void
 */
function xdebug_set_filter(int $group, int $listType, array $configuration) {}

function xdebug_connect_to_client(): bool {}

function xdebug_notify(mixed $data): bool {}

const XDEBUG_STACK_NO_DESC = 1;

const XDEBUG_TRACE_APPEND = 1;

const XDEBUG_TRACE_COMPUTERIZED = 2;

const XDEBUG_TRACE_HTML = 4;

const XDEBUG_TRACE_NAKED_FILENAME = 8;

const XDEBUG_CC_UNUSED = 1;

const XDEBUG_CC_DEAD_CODE = 2;

const XDEBUG_CC_BRANCH_CHECK = 4;

const XDEBUG_FILTER_TRACING = 768;

const XDEBUG_FILTER_STACK = 512;

const XDEBUG_FILTER_CODE_COVERAGE = 256;

const XDEBUG_FILTER_NONE = 0;

const XDEBUG_PATH_WHITELIST = 1;

const XDEBUG_PATH_BLACKLIST = 2;

const XDEBUG_NAMESPACE_WHITELIST = 17;

const XDEBUG_NAMESPACE_BLACKLIST = 18;

const XDEBUG_NAMESPACE_EXCLUDE = 18;

const XDEBUG_NAMESPACE_INCLUDE = 17;

const XDEBUG_PATH_EXCLUDE = 2;

const XDEBUG_PATH_INCLUDE = 1;
