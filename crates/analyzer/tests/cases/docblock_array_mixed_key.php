<?php

declare(strict_types=1);

final class MixedKeyGroup
{
}

final class MixedKeyResult
{
    /**
     * Legacy DB-library style: the key is written as `mixed`, but PHP array
     * keys can only be `int|string`, so this is `array<array-key, int>`.
     *
     * @return array<mixed, int>
     */
    public function fetchIndexed(): array
    {
        return [1 => 2];
    }
}

final class MixedKeyGrouper
{
    /** @return array<mixed, MixedKeyGroup> */
    public function groupBy(string $key): array
    {
        return [$key => new MixedKeyGroup()];
    }
}

/**
 * @param array<string, MixedKeyGroup> $individual
 *
 * @return array<MixedKeyGroup>
 */
function mergeMixedKeyGroups(MixedKeyGrouper $grouper, array $individual): array
{
    return [...$grouper->groupBy('id'), ...$individual];
}

/** @return array<int> */
function collectMixedKeyValues(MixedKeyResult $result): array
{
    $values = [];
    foreach ($result->fetchIndexed() as $key => $value) {
        $values[$key] = $value;
    }

    return $values;
}

/** @return list<array-key> */
function collectMixedKeys(MixedKeyResult $result): array
{
    $keys = [];
    foreach ($result->fetchIndexed() as $key => $value) {
        $keys[] = $key;
    }

    return $keys;
}

/**
 * Unlike true arrays, iterables (e.g. generators) may yield object keys, so an
 * explicit `mixed` key is preserved instead of normalized to `array-key`.
 *
 * @param iterable<mixed, MixedKeyGroup> $groups
 */
function indexByIterableKey(iterable $groups): void
{
    /** @mago-expect analysis:mixed-assignment */
    foreach ($groups as $key => $group) {
        /** @mago-expect analysis:mixed-argument */
        takeArrayKey($key);
    }
}

function takeArrayKey(int|string $key): string
{
    return (string) $key;
}

echo count(collectMixedKeyValues(new MixedKeyResult()));
echo count(collectMixedKeys(new MixedKeyResult()));
echo count(mergeMixedKeyGroups(new MixedKeyGrouper(), ['a' => new MixedKeyGroup()]));
