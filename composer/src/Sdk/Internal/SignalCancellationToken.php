<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal;

use Closure;
use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\CancelledException;
use Throwable;

/**
 * @internal
 */
final class SignalCancellationToken implements CancellationTokenInterface
{
    private bool $cancelled = false;

    private ?CancelledException $exception = null;

    /**
     * @var array<int<1, max>, Closure(CancelledException): void>
     */
    private array $callbacks = [];

    /**
     * @var int<0, max>
     */
    private int $nextSubscription = 0;

    public function cancel(?Throwable $cause = null): void
    {
        if ($this->cancelled) {
            return;
        }

        $this->cancelled = true;
        $exception = new CancelledException($cause);
        $this->exception = $exception;
        $callbacks = $this->callbacks;
        $this->callbacks = [];
        foreach ($callbacks as $callback) {
            $callback($exception);
        }
    }

    public function isCancelled(): bool
    {
        return $this->cancelled;
    }

    public function throwIfCancelled(): void
    {
        if ($this->exception !== null) {
            throw $this->exception;
        }
    }

    public function subscribe(Closure $callback): int
    {
        if ($this->exception !== null) {
            $callback($this->exception);

            return 0;
        }

        $subscription = ++$this->nextSubscription;
        $this->callbacks[$subscription] = $callback;

        return $subscription;
    }

    public function unsubscribe(int $subscription): void
    {
        unset($this->callbacks[$subscription]);
    }
}
