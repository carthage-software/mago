<?php

interface SessionHandlerInterface
{
    public function close(): bool;

    public function destroy(string $id): bool;

    public function gc(int $max_lifetime): int|false;

    public function open(string $path, string $name): bool;

    public function read(string $id): string|false;

    public function write(string $id, string $data): bool;
}

interface SessionIdInterface
{
    public function create_sid(): string;
}

interface SessionUpdateTimestampHandlerInterface
{
    public function validateId(string $id): bool;

    public function updateTimestamp(string $id, string $data): bool;
}

class SessionHandler implements SessionHandlerInterface, SessionIdInterface
{
    public function close(): bool {}

    public function create_sid(): string {}

    public function destroy(string $id): bool {}

    public function gc(int $max_lifetime): int|false {}

    public function open(string $path, string $name): bool {}

    public function read(string $id): string|false {}

    public function write(string $id, string $data): bool {}
}

function session_name(?string $name = null): string|false {}

function session_module_name(?string $module = null): string|false {}

function session_save_path(?string $path = null): string|false {}

function session_id(?string $id = null): string|false {}

function session_regenerate_id(bool $delete_old_session = false): bool {}

function session_register_shutdown(): void {}

function session_decode(string $data): bool {}

function session_encode(): string|false {}

function session_start(array $options = []): bool {}

function session_create_id(string $prefix = ''): string|false {}

function session_gc(): int|false {}

function session_destroy(): bool {}

function session_unset(): bool {}

function session_set_save_handler(
    callable $open,
    callable $close,
    callable $read,
    callable $write,
    callable $destroy,
    callable $gc,
    ?callable $create_sid = null,
    ?callable $validate_sid = null,
    ?callable $update_timestamp = null,
): bool {}

function session_set_save_handler(SessionHandlerInterface $sessionhandler, bool $register_shutdown = true): bool {}

function session_cache_limiter(?string $value = null): string|false {}

function session_cache_expire(?int $value = null): int|false {}

function session_set_cookie_params(array $lifetime_or_options): bool {}

function session_set_cookie_params(
    int $lifetime_or_options,
    ?string $path = null,
    ?string $domain = null,
    ?bool $secure = null,
    ?bool $httponly = null,
): bool {}

/**
 * @return array{
 *  lifetime: int,
 *  path: string,
 *  domain: string,
 *  secure: bool,
 *  httponly: bool,
 *  samesite: string
 * }
 */
function session_get_cookie_params(): array {}

function session_write_close(): bool {}

function session_commit(): bool {}

function session_status(): int {}

function session_abort(): bool {}

function session_reset(): bool {}
