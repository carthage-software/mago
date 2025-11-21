<?php

/**
 * @template T
 */
interface PromiseInterface
{
    /**
     * @template Ts
     *
     * @param (Closure(Throwable): Ts) $failure
     *
     * @return PromiseInterface<T|Ts>
     */
    public function catch(Closure $failure): PromiseInterface;
}

/**
 * @template T
 * @implements PromiseInterface<T>
 */
final class Awaitable implements PromiseInterface
{
    /**
     * {@inheritDoc}
     *
     * @template Ts
     *
     * @param Closure(Throwable): Ts $failure
     *
     * @return Awaitable<T|Ts>
     */
    #[Override]
    public function catch(Closure $failure): Awaitable
    {
        exit(0);
    }

    public function ignore(): self
    {
        exit(0);
    }
}

/**
 * @template T
 * @param (Closure(): T) $closure
 * @return Awaitable<T>
 */
function run(Closure $closure): Awaitable
{
    exit(0);
}

$awaitable = run(static function (): string {
    return 'hello, world';
});

$awaitable = $awaitable->catch(static function (Throwable $e): string {
    return $e->getMessage();
});

$awaitable->ignore();
