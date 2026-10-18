<?php

declare(strict_types=1);

/** @param array{} $_ */
function acceptEmptyArray(array $_): void {}

function acceptString(string $_): void {}

function acceptInt(int $_): void {}

/** @param array<int, string> $items */
function afterUnsettingIntegerKeys(array $items): void
{
    if (empty($items)) {
        return;
    }

    while (!empty($items)) {
        unset($items[rand(0, 10)]);
    }

    acceptEmptyArray($items);

    foreach ($items as $key => $value) {
        acceptInt($key);
        acceptString($value);
    }

    acceptEmptyArray($items);
}

/** @param list<string> $items */
function afterUnsettingListItems(array $items): void
{
    while (!empty($items)) {
        unset($items[0]);
    }

    acceptEmptyArray($items);

    foreach ($items as $value) {
        acceptString($value);
    }

    acceptEmptyArray($items);
}

/** @param array<string, int> $items */
function afterUnsettingStringKeys(array $items, string $key): void
{
    if (empty($items)) {
        return;
    }

    while (!empty($items)) {
        unset($items[$key]);
    }

    acceptEmptyArray($items);

    foreach ($items as $value) {
        acceptInt($value);
    }

    acceptEmptyArray($items);
}

/** @param array<string, int> $items */
function afterEmptyGuard(array $items): void
{
    if (!empty($items)) {
        return;
    }

    acceptEmptyArray($items);

    foreach ($items as $key => $value) {
        acceptString($key);
        acceptInt($value);
    }

    // @mago-expect analysis:impossible-condition
    if ($items) {
    }
}

/** @param list<string> $items */
function insideFalsyGuard(array $items): void
{
    if (!$items) {
        acceptEmptyArray($items);

        foreach ($items as $value) {
            acceptString($value);
        }

        // @mago-expect analysis:impossible-condition
        if ($items) {
        }
    }
}

/** @return list{'key', 'value', 'result'} */
function emptyLoopDoesNotAssign(): array
{
    $key = 'key';
    $value = 'value';
    $result = 'result';

    foreach ([] as $key => &$value) {
        $result = 42;
    }

    return [$key, $value, $result];
}

/** @return array{} */
function emptyLoopStillEvaluatesIterator(): array
{
    foreach ($items = [] as $value) {
        acceptString($value);
    }

    return $items;
}

function emptyLoopDoesNotReturn(): string
{
    foreach ([] as $_) {
        return 42;
    }

    return 'result';
}

/** @param array<string, int> $items */
function possiblyEmptyLoopStillChecksValues(array $items): void
{
    foreach ($items as $value) {
        // @mago-expect analysis:invalid-argument
        acceptString($value);
    }
}

/** @param array{}|array{value: int} $items */
function nonEmptyUnionMemberStillChecksValues(array $items): void
{
    foreach ($items as $value) {
        // @mago-expect analysis:invalid-argument
        acceptString($value);
    }
}

function arrayFilledOnEarlierIteration(): void
{
    $items = [];

    for ($i = 0; $i < 2; $i++) {
        foreach ($items as $value) {
            // @mago-expect analysis:invalid-argument
            acceptString($value);
        }

        $items = [42];
    }
}
