<?php

declare(strict_types=1);

/**
 * @template T of array<string, mixed> = array{name: string}
 */
final class Record
{
    /**
     * @param T $payload
     */
    public function __construct(public readonly array $payload) {}
}

/**
 * @param Record $r
 *
 * @return string
 */
function name_of(Record $r): string
{
    return $r->payload['name'];
}

echo name_of(new Record(['name' => 'mago']));
