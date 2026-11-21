<?php

declare(strict_types=1);

/**
 * @param int<0, 9> $value
 */
function increment_counts_as_assignment(int $value): void
{
    $changed = false;
    if ($value < 9) {
        $changed = true;
        $value++;
    }

    if (!$changed) {
        return;
    }

    if ($value === 9) {
        echo 'nine';
    }
}

final class IncrementCountsAsAssignmentLimit
{
    private const int MAX = 5;

    /**
     * @param list<string> $values
     */
    public static function count(array $values): int
    {
        $count = 0;
        foreach ($values as $_) {
            if ($count < self::MAX) {
                $count++;
            }
        }

        return $count;
    }
}

/**
 * @param callable(): bool $done
 */
function increment_counts_as_assignment_in_condition(callable $done): void
{
    $attempts = 0;
    while (true) {
        if ($done() && $attempts++ > 100) {
            break;
        }
    }
}
