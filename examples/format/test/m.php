<?php

$this->state->subscribe(static function (null|Throwable $error, mixed $value) use ($state, $always): void {
    try {
        if ($error) {
            $state->error($error);
        } else {
            /**
             * @var mixed $value
             */
            $state->complete($value);
        }
    } catch (Throwable $throwable) {
        $state->error($throwable);
    }
});

function sort_by(iterable $iterable, Closure $scalar_func, null|Closure $comparator = null): array
{
    $comparator ??=
        /**
         * @param Ts $a
         * @param Ts $b
         */
        static fn($a, $b): int => $a <=> $b;
}

// if ($bin > 25) $diff += 0x61 - 0x41 - 26; // 6
$diff += ((25 - $bin) >> 8) & 6;
// if ($bin > 51) $diff += 0x30 - 0x61 - 26; // -75
$diff -= ((51 - $bin) >> 8) & 75;
// if ($bin > 61) $diff += 0x2b - 0x30 - 10; // -15
$diff -= ((61 - $bin) >> 8) & 15;
// if ($bin > 62) $diff += 0x2f - 0x2b - 1; // 3
$diff += ((62 - $bin) >> 8) & 3;

$writable = str_contains($meta['mode'], 'x') ||
    str_contains($meta['mode'], 'w') ||
    str_contains($meta['mode'], 'c') ||
    str_contains($meta['mode'], 'a') ||
    str_contains($meta['mode'], '+');

interface CloseReadStreamHandleInterface extends
    CloseStreamHandleInterface,
    IO\CloseReadHandleInterface,
    ReadStreamHandleInterface
{
}

trait TemporalConvenienceMethodsTrait
{
    public function toString(
        null|DateStyle $date_style = null,
        null|TimeStyle $time_style = null,
        null|Timezone $timezone = null,
        null|Locale $locale = null,
    ): string {
        $timestamp = $this->getTimestamp();

        /**
         * @psalm-suppress InvalidOperand
         * @psalm-suppress ImpureMethodCall
         */
        return Internal\create_intl_date_formatter(
            $date_style,
            $time_style,
            null,
            $timezone,
            $timezone,
            $timezone,
            $locale,
        )->format($timestamp->getSeconds() + ($timestamp->getNanoseconds() / NANOSECONDS_PER_SECOND));

        return Internal\create_intl_date_formatter(
            $date_style,
            $time_style,
            null,
            $timezone,
            $timezone,
            $timezone,
            $locale,
        )->format($timestamp->getSeconds() + ($timestamp->getNanoseconds() / NANOSECONDS_PER_SECOND));
    }

    public function bar(): void
    {
        $err |= ($char0 | $char1 | $char2) >> 8;
    }
}

/**
 * @psalm-suppress InvalidOperand
 * @psalm-suppress ImpureMethodCall
 */
Internal\create_intl_date_formatter($date_style, $time_style, null, $timezone, $locale)->format(
    $timestamp->getSeconds() + ($timestamp->getNanoseconds() / NANOSECONDS_PER_SECOND),
);

Internal\create_intl_date_formatter($date_style, $time_style, null, $timezone, $locale)->format(
    $timestamp->getSeconds() + ($timestamp->getNanoseconds() / NANOSECONDS_PER_SECOND),
);

$err |= ($char0 | $char1 | $char2) >> 8;

fooo(match (delta) {
    12 => 12,
    13 => 13,
    14 => 14,
    15 => 15,
    16 => 16,
    17 => 17,
    18 => 18,
    19 => 19,
    18 => 18,
    19 => 19,
    18 => new class
    {
        public function foo(): void
        {
            $err |= ($char0 | $char1 | $char2) >> 8;
        }
    },
    19 => static function () {
        Internal\create_intl_date_formatter($date_style, $time_style, null, $timezone, $locale)->format(
            $timestamp->getSeconds() + ($timestamp->getNanoseconds() / NANOSECONDS_PER_SECOND),
        );

        return 19;
    },
});

$a = match (delta) {
    12 => 12,
    19 => 19,
    18 => 18,
    19 => 19,
};

return array_diff_key(from_iterable($first), from_iterable($second), ...Vec\map(
    $rest,
    /**
     * @param iterable<Tk, Tv> $iterable
     *
     * @return array<Tk, Tv>
     */
    static fn(iterable $iterable): array => from_iterable($iterable),
));

$id = $state->subscribe(
    /**
     * @param Tv|null $_result
     */
    static function (null|Throwable $_error, mixed $_result, string $id) use ($key, $awaitable, $queue): void {
        unset($queue->pending[$id]);

        if ($queue->suspension) {
            $queue->suspension->resume([$key, $awaitable]);
            $queue->suspension = null;
            return;
        }

        $queue->items[] = [$key, $awaitable];
    },
);

$id = $state->subscribe(
    /**
     * @param Tv|null $_result
     */
    static function (
        null|Throwable $_error,
        mixed $_result,
        string $id,
    ) use (
        $key,
        $awaitable,
        $queue,
    ): void {
        unset($queue->pending[$id]);

        if ($queue->suspension) {
            $queue->suspension->resume([$key, $awaitable]);
            $queue->suspension = null;
            return;
        }

        $queue->items[] = [$key, $awaitable];
    },
);

$a = $foo
    ? $bar
    : $baz;

$a = $foo ?
    $bar :
    $baz;

$predicate = $predicate
    ?? /**
        * @param Tk $_k
        * @param Tv $v
        */
        static fn(mixed $_k, mixed $v): bool => (bool) $v;

$a = 1 + 2;
$a = 1 +
     2;

$a = 1
    + 2;

$predicate = $predicate
    ?? static fn(mixed $_k, mixed $v): bool => (bool) $v;

$writable = str_contains($meta['mode'], 'x') ||
    str_contains($meta['mode'], 'w') ||
    str_contains($meta['mode'], 'c') ||
    str_contains($meta['mode'], 'a') ||
    str_contains($meta['mode'], '+');

$writable = str_contains($meta['mode'], 'x')
    || str_contains($meta['mode'], 'w')
    || str_contains($meta['mode'], 'c')
    || str_contains($meta['mode'], 'a')
    || str_contains($meta['mode'], '+');
