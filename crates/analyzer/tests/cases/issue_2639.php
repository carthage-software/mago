<?php

declare(strict_types=1);

namespace Issue2639;

function refIntoMissingKey(): void
{
    $payload = ['id' => 1, 'items' => [['name' => 'x']]];
    $extra = &$payload['items'][0]['extra'];
    $extra = ['a' => 1];
}

function refIntoEmptyArray(): void
{
    $payload = [];
    $x = &$payload['a']['b'];
    $x = 1;
}

function missingKeyValue(): null
{
    $payload = [];
    $x = &$payload['a'];

    return $x;
}

function nestedMissingKeyValue(): null
{
    $payload = [];
    $x = &$payload['a']['b'];

    return $x;
}

/** @return array{a: null} */
function createdKey(): array
{
    $payload = [];
    $x = &$payload['a'];

    return $payload;
}

/** @return array{a: array{b: null}} */
function createdNestedKeys(): array
{
    $payload = [];
    $x = &$payload['a']['b'];

    return $payload;
}

/**
 * @param array{a?: int} $payload
 * @return int|null
 */
function optionalKeyValue(array $payload): ?int
{
    $x = &$payload['a'];

    return $x;
}

/** @return array{a: int|null} */
function createdOptionalKey(bool $c): array
{
    $payload = $c ? ['a' => 1] : [];
    $x = &$payload['a'];

    return $payload;
}

/** @return 1 */
function existingKeyValue(): int
{
    $payload = ['a' => 1];
    $x = &$payload['a'];

    return $x;
}

function plainReadStillFails(): void
{
    $payload = ['a' => 1];
    // @mago-expect analysis:undefined-string-array-index
    $x = $payload['b'];
}

function missingIndexExpressionStillFails(): void
{
    $payload = [];
    $keys = ['a' => 'b'];
    // @mago-expect analysis:undefined-string-array-index
    $x = &$payload[(string) $keys['b']];
}

function missingListIndexValue(): null
{
    $payload = [1];
    $x = &$payload[2];

    return $x;
}

/** @param array<string, int> $payload */
function genericKeyMayBeMissing(array $payload): int
{
    $x = &$payload['a'];

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $x;
}

final class Properties
{
    public array $payload = [];

    public function __construct(public readonly array $fixed = []) {}

    /** @return array{a: null} */
    public function bindProperty(): array
    {
        $this->payload = [];
        $x = &$this->payload['a'];

        return $this->payload;
    }

    public function readonlyProperty(): void
    {
        // @mago-expect analysis:invalid-property-write
        $_x = &$this->fixed['a'];
    }
}

function stop(): never
{
    exit;
}

function neverReturningIndex(): void
{
    $payload = [];
    // @mago-expect analysis:impossible-assignment
    $x = &$payload[stop()];
}
