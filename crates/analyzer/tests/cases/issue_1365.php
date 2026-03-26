<?php

declare(strict_types=1);

class Tag1365 {}

/**
 * @return list<Tag1365>
 */
function buildTags1365(string $name): array
{
    /** @var list<Tag1365> $result */
    $result = [];

    $normalizedTags = match ($name) {
        'a'     => [getNullableTag1365()],
        default => [new Tag1365()],
    };

    return [
        ...$result,
        ...array_values(array_filter($normalizedTags)),
    ];
}

function getNullableTag1365(): ?Tag1365
{
    return new Tag1365();
}
