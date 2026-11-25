<?php

declare(strict_types=1);

final class Issue2568CallbackException extends Exception {}

/**
 * @param-immediately-invoked-callable $callback
 */
function issue_2568_catch_callback(callable $callback): void
{
    try {
        $callback();
    } catch (Issue2568CallbackException) {
        // The wrapper deliberately consumes callback exceptions.
    }
}

/** @mago-expect analysis:unhandled-thrown-type */
function issue_2568_caller(): void
{
    issue_2568_catch_callback(static function (): void {
        throw new Issue2568CallbackException();
    });
}

/** @param-immediately-invoked-callable $callback */
function issue_2568_invoke_callback(callable $callback): void
{
    $callback();
}

/** @mago-expect analysis:unhandled-thrown-type */
function issue_2568_uncaught_userland_callback(): void
{
    issue_2568_invoke_callback(static function (): void {
        throw new Issue2568CallbackException();
    });
}
