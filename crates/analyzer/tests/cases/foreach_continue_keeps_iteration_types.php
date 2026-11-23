<?php

declare(strict_types=1);

final class Flag
{
    public function equals(Flag $other): bool
    {
        return $this === $other;
    }
}

/**
 * @param non-empty-list<Flag> $items
 */
function last_flag(array $items): Flag
{
    $last = null;
    foreach ($items as $item) {
        if ($last === null) {
            $last = $item;
            continue;
        }

        if (!$last->equals($item)) {
            return new Flag();
        }
    }

    return $last;
}

/**
 * @param list<Flag> $items
 */
function maybe_last_flag(array $items): Flag
{
    $last = null;
    foreach ($items as $item) {
        $last = $item;
        continue;
    }

    /** @mago-expect analysis:nullable-return-statement,invalid-return-statement */
    return $last;
}

/**
 * @param non-empty-list<int> $items
 */
function skipped_assignment(array $items): int
{
    $last = null;
    foreach ($items as $item) {
        if ($item > 1) {
            continue;
        }

        $last = $item;
    }

    /** @mago-expect analysis:nullable-return-statement,invalid-return-statement */
    return $last;
}
