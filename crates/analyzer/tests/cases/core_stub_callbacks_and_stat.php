<?php

declare(strict_types=1);

interface Encoder
{
}

final class JsonEncoder implements Encoder
{
}

final class Registry
{
    /** @param WeakReference<Encoder> $encoder */
    public function __construct(
        private WeakReference $encoder,
    ) {}

    public function encoder(): ?Encoder
    {
        return $this->encoder->get();
    }
}

function registry(JsonEncoder $encoder): Registry
{
    return new Registry(WeakReference::create($encoder));
}

function describe(int $count, string $label): string
{
    return $label . ': ' . $count;
}

function described(): string
{
    return call_user_func(describe(...), 3, 'items') . call_user_func_array(describe(...), [4, 'more']);
}

/**
 * @param Iterator<string, int> $iterator
 */
function current_key(Iterator $iterator): string
{
    return $iterator->key();
}

/**
 * @param resource $stream
 */
function stream_size($stream): int
{
    $stat = fstat($stream);
    if ($stat === false) {
        return 0;
    }

    return $stat[7] + $stat['size'];
}

function modified_at(string $path): ?int
{
    $stat = stat($path);

    return $stat === false ? null : $stat[9];
}
