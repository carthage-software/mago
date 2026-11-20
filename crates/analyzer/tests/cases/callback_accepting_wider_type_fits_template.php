<?php

declare(strict_types=1);

/**
 * @template T of int
 *
 * @param Closure(T): void $callback
 */
function wider_callback_closure_template(Closure $callback): void
{
    $callback(1);
}

/**
 * @template T of string|array<string>
 *
 * @param callable(T): void $callback
 */
function wider_callback_callable_template(callable $callback): void
{
    $callback('value');
}

/**
 * @param callable(array<array-key, mixed>|scalar): void $wide
 */
function wider_callback_calls(callable $wide): void
{
    wider_callback_callable_template($wide);
    wider_callback_callable_template(static function (mixed $_value): void {});
    wider_callback_closure_template(static function (int|string $_value): void {});
}
