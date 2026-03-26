<?php

function apcu_clear_cache(): bool {}

function apcu_cache_info(bool $limited = false): array|false {}

function apcu_key_info(string $key): ?array {}

function apcu_sma_info(bool $limited = false): array|false {}

function apcu_enabled(): bool {}

/** @param array|string $key */
function apcu_store($key, mixed $value = UNKNOWN, int $ttl = 0): array|bool {}

/** @param array|string $key */
function apcu_add($key, mixed $value = UNKNOWN, int $ttl = 0): array|bool {}

/** @param bool $success */
function apcu_inc(string $key, int $step = 1, &$success = null, int $ttl = 0): int|false {}

/** @param bool $success */
function apcu_dec(string $key, int $step = 1, &$success = null, int $ttl = 0): int|false {}

function apcu_cas(string $key, int $old, int $new): bool {}

/**
 * @param array|string $key
 * @param bool $success
 */
function apcu_fetch($key, &$success = null): mixed {}

/** @param array|string $key */
function apcu_exists($key): array|bool {}

/** @param APCUIterator|array|string $key */
function apcu_delete($key): array|bool {}

function apcu_entry(string $key, callable $callback, int $ttl = 0): mixed {}

function apcu_inc_request_time(int $by = 1): void {}

/**
 * @implements Iterator<string|int, mixed>
 */
class APCUIterator implements Iterator
{
    /**
     * @param array|string|null $search
     */
    public function __construct(
        $search = null,
        int $format = APC_ITER_ALL,
        int $chunk_size = 0,
        int $list = APC_LIST_ACTIVE,
    ) {}

    public function rewind(): void {}

    public function next(): void {}

    public function valid(): bool {}

    public function key(): string|int {}

    public function current(): mixed {}

    public function getTotalHits(): int {}

    public function getTotalSize(): int {}

    public function getTotalCount(): int {}
}
