<?php

declare(strict_types=1);

final class ClosureArgumentFailure extends Exception
{
}

/**
 * @template T
 *
 * @param Closure(): T $callback
 *
 * @return T
 */
function closure_argument_run(Closure $callback): mixed
{
    return $callback();
}

/**
 * @param list<Closure(): int> $callbacks
 *
 * @return list<int>
 */
function closure_argument_run_all(array $callbacks): array
{
    $results = [];
    foreach ($callbacks as $callback) {
        $results[] = $callback();
    }

    return $results;
}

final class ClosureArgumentTimer
{
    public function __construct(
        public readonly Closure $onTimeout,
    ) {}
}

/**
 * @throws ClosureArgumentFailure
 */
function closure_argument(int $value): int
{
    return closure_argument_run(
        /** @throws ClosureArgumentFailure */
        static function () use ($value): int {
            if ($value < 0) {
                throw new ClosureArgumentFailure();
            }

            return $value;
        },
    );
}

/**
 * @throws ClosureArgumentFailure
 */
function arrow_function_argument(int $value): int
{
    return closure_argument_run(
        /** @throws ClosureArgumentFailure */
        static fn(): int => $value < 0 ? throw new ClosureArgumentFailure() : $value,
    );
}

/**
 * @return list<int>
 *
 * @throws ClosureArgumentFailure
 */
function closure_in_array_argument(int $value): array
{
    return closure_argument_run_all([
        /** @throws ClosureArgumentFailure */
        static fn(): int => $value < 0 ? throw new ClosureArgumentFailure() : $value,
    ]);
}

/**
 * @throws ClosureArgumentFailure
 */
function closure_constructor_argument(): ClosureArgumentTimer
{
    return new ClosureArgumentTimer(
        /** @throws ClosureArgumentFailure */
        static function (): never {
            throw new ClosureArgumentFailure();
        },
    );
}

/**
 * @throws ClosureArgumentFailure
 */
function closure_not_passed(): Closure
{
    return
        /** @throws ClosureArgumentFailure */
        static function (): never {
            throw new ClosureArgumentFailure();
        };
}

/**
 * @throws ClosureArgumentFailure
 */
function closure_argument_caught(int $value): int
{
    try {
        return closure_argument_run(
            /** @throws ClosureArgumentFailure */
            static fn(): int => $value < 0 ? throw new ClosureArgumentFailure() : $value,
        );
    } catch (ClosureArgumentFailure) {
        return 0;
    }
}
