<?php

function readline(?string $prompt): string|false {}

/**
 * @return string|int|bool|array{
 *   line_buffer: string,
 *   point: int,
 *   end: int,
 *   mark: int,
 *   done: int,
 *   pending_input: int,
 *   prompt: string,
 *   terminal_name: string,
 *   completion_append_character: string,
 *   completion_suppress_append: bool,
 *   erase_empty_line: int,
 *   library_version: string,
 *   readline_name: string,
 *   attempted_completion_over: int,
 * }
 */
function readline_info(?string $var_name = null, int|string|bool|null $value = null): int|string|bool|array {}

function readline_add_history(string $prompt): bool {}

function readline_clear_history(): bool {}

function readline_list_history(): array {}

function readline_read_history(?string $filename): bool {}

function readline_write_history(?string $filename): bool {}

function readline_completion_function(callable $callback): bool {}

function readline_callback_handler_install(string $prompt, callable $callback): bool {}

function readline_callback_read_char(): void {}

function readline_callback_handler_remove(): bool {}

function readline_redisplay(): void {}

function readline_on_new_line(): void {}

const READLINE_LIB = 'readline';
