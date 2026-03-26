<?php

namespace Psl\Async {
    /**
     * @template T
     */
    final readonly class Awaitable
    {
        /**
         * @return T
         */
        public function await(): mixed
        {
            /** @var T */
            return null;
        }
    }

    /**
     * @template Tk of array-key
     * @template Tv
     *
     * @param iterable<Tk, Awaitable<Tv>> $awaitables
     *
     * @return array<Tk, Tv>
     */
    function all(iterable $awaitables): array
    {
        return [];
    }

    /**
     * @template Tk of array-key
     * @template Tv
     *
     * @param iterable<Tk, (Closure(): Tv)> $tasks
     *
     * @return array<Tk, Tv>
     */
    function concurrently(iterable $tasks): array
    {
        return [];
    }
}

namespace {
    function take_string(string $value): void {}

    function take_int(int $value): void {}

    /**
     * @return Psl\Async\Awaitable<string>
     */
    function get_awaitable_string(): Psl\Async\Awaitable
    {
        return get_awaitable_string();
    }

    /**
     * @return Psl\Async\Awaitable<int>
     */
    function get_awaitable_int(): Psl\Async\Awaitable
    {
        return get_awaitable_int();
    }

    function test_concurrently_keyed(): void
    {
        $result = Psl\Async\concurrently([
            'name' => fn(): string => 'hello',
            'age' => fn(): int => 42,
        ]);

        take_string($result['name']);
        take_int($result['age']);
    }

    function test_concurrently_list(): void
    {
        $result = Psl\Async\concurrently([
            fn(): string => 'hello',
            fn(): int => 42,
        ]);

        take_string($result[0]);
        take_int($result[1]);
    }

    function test_all_keyed(): void
    {
        $result = Psl\Async\all([
            'name' => get_awaitable_string(),
            'age' => get_awaitable_int(),
        ]);

        take_string($result['name']);
        take_int($result['age']);
    }

    function test_all_list(): void
    {
        $result = Psl\Async\all([
            get_awaitable_string(),
            get_awaitable_int(),
        ]);

        take_string($result[0]);
        take_int($result[1]);
    }
}
