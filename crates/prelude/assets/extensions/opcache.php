<?php

function opcache_compile_file(string $filename): bool {}

function opcache_invalidate(string $filename, bool $force = false): bool {}

function opcache_reset(): bool {}

/**
 * @return false|array{
 *    opcache_enabled: bool,
 *    file_cache: string,
 *    file_cache_only: bool,
 *    cache_full: bool,
 *    restart_pending: bool,
 *    restart_in_progress: bool,
 *    memory_usage: array{
 *        used_memory: int,
 *        free_memory: int,
 *        wasted_memory: int,
 *        current_wasted_percentage: float,
 *    },
 *    interned_strings_usage: array{
 *        buffer_size: int,
 *        used_memory: int,
 *        free_memory: int,
 *        number_of_strings: int,
 *    },
 *    opcache_statistics: array{
 *        num_cached_scripts: int,
 *        num_cached_keys: int,
 *        max_cached_keys: int,
 *        hits: int,
 *        misses: int,
 *        blacklist_misses: int,
 *        blacklist_miss_ratio: float,
 *        opcache_hit_rate: float,
 *        start_time: int,
 *        last_restart_time: int,
 *        oom_restarts: int,
 *        hash_restarts: int,
 *        manual_restarts: int,
 *    },
 *    scripts: array<string, mixed>,
 *    jit: array{
 *        enabled: bool,
 *        on: bool,
 *        kind: int,
 *        buffer_size: int,
 *        buffer_free: int,
 *        opt_level: int,
 *        opt_flags: bool,
 *    },
 *    preload_statistics?: array<string, mixed>,
 * }
 */
function opcache_get_status(bool $include_scripts = true): array|false {}

/**
 * @return false|array{
 *    directives: array{
 *        'opcache.enable': bool,
 *        'opcache.enable_cli': bool,
 *        'opcache.use_cwd': bool,
 *        'opcache.validate_timestamps': bool,
 *        'opcache.validate_permission': bool,
 *        'opcache.validate_root': bool,
 *        'opcache.dups_fix': bool,
 *        'opcache.revalidate_path': bool,
 *        'opcache.log_verbosity_level': int,
 *        'opcache.memory_consumption': int,
 *        'opcache.interned_strings_buffer': int,
 *        'opcache.max_accelerated_files': int,
 *        'opcache.max_wasted_percentage': float,
 *        'opcache.force_restart_timeout': int,
 *        'opcache.revalidate_freq': int,
 *        'opcache.preferred_memory_model': string,
 *        'opcache.blacklist_filename': string,
 *        'opcache.max_file_size': int,
 *        'opcache.error_log': string,
 *        'opcache.protect_memory': bool,
 *        'opcache.save_comments': bool,
 *        'opcache.record_warnings': bool,
 *        'opcache.enable_file_override': bool,
 *        'opcache.optimization_level': int,
 *        'opcache.lockfile_path': string,
 *        'opcache.file_cache': string,
 *        'opcache.file_cache_only': bool,
 *        'opcache.file_cache_consistency_checks': bool,
 *        'opcache.file_update_protection': int,
 *        'opcache.opt_debug_level': int,
 *        'opcache.restrict_api': string,
 *        'opcache.huge_code_pages': bool,
 *        'opcache.preload': string,
 *        'opcache.preload_user': string,
 *        'opcache.jit': string,
 *        'opcache.jit_buffer_size': int,
 *        'opcache.jit_debug': int,
 *        'opcache.jit_bisect_limit': int,
 *        'opcache.jit_blacklist_root_trace': int,
 *        'opcache.jit_blacklist_side_trace': int,
 *        'opcache.jit_hot_func': int,
 *        'opcache.jit_hot_loop': int,
 *        'opcache.jit_hot_return': int,
 *        'opcache.jit_hot_side_exit': int,
 *        'opcache.jit_max_exit_counters': int,
 *        'opcache.jit_max_loop_unrolls': int,
 *        'opcache.jit_max_polymorphic_calls': int,
 *        'opcache.jit_max_recursive_calls': int,
 *        'opcache.jit_max_recursive_returns': int,
 *        'opcache.jit_max_root_traces': int,
 *        'opcache.jit_max_side_traces': int,
 *        'opcache.jit_prof_threshold': float,
 *        'opcache.jit_max_trace_length': int,
 *    },
 *    version: string[],
 *    blacklist: array<string, mixed>,
 * }
 */
function opcache_get_configuration(): array|false {}

function opcache_is_script_cached(string $filename): bool {}

function opcache_jit_blacklist(Closure $closure): void {}
