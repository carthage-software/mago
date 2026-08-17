<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Closure;
use Mago\Sdk\Exception\CancelledException;

/**
 * Cooperative cancellation for an in-flight extension request.
 *
 * @api
 */
interface CancellationTokenInterface
{
    /**
     * Whether cancellation has been requested.
     */
    public function isCancelled(): bool;

    /**
     * Throw when cancellation has been requested.
     *
     * @throws CancelledException
     */
    public function throwIfCancelled(): void;

    /**
     * Invoke a callback when cancellation is requested.
     *
     * The callback runs immediately when the token has already been cancelled.
     * A return value of zero indicates that no subscription was retained.
     *
     * @param Closure(CancelledException): void $callback
     *
     * @return int<0, max>
     */
    public function subscribe(Closure $callback): int;

    /**
     * Remove a previously registered callback.
     *
     * @param int<0, max> $subscription
     */
    public function unsubscribe(int $subscription): void;
}
