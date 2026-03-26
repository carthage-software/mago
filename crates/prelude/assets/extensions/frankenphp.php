<?php

/**
 * @param int<100, 599> $status
 */
function headers_send(int $status = 200): int {}

function frankenphp_handle_request(callable $callback): bool {}

function frankenphp_finish_request(): bool {}

function frankenphp_request_headers(): array {}

function frankenphp_response_headers(): array|false {}
