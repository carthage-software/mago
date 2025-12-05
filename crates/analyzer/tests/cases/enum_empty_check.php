<?php

declare(strict_types=1);

enum TaskStatus: string
{
    case PENDING = 'pending';
    case ACTIVE = 'active';
    case COMPLETED = 'completed';
}

enum TaskPriority: int
{
    case LOW = 1;
    case MEDIUM = 2;
    case HIGH = 3;
}

/**
 * @return list<string>
 */
function getStatusValues(): array
{
    $cases = TaskStatus::cases();
    $values = [];

    foreach ($cases as $case) {
        $values[] = $case->value;
    }

    return $values;
}

/**
 * @return list<TaskPriority>
 */
function getHighPriorities(): array
{
    $cases = TaskPriority::cases();
    $filtered = [];

    foreach ($cases as $case) {
        if ($case->value >= 2) {
            $filtered[] = $case;
        }
    }

    // This check is NOT impossible - filtering could result in empty array
    if ([] === $filtered) {
        return [TaskPriority::LOW];
    }

    return $filtered;
}

/**
 * @param callable(TaskStatus): bool $filter
 * @return list<TaskStatus>
 */
function filterStatuses(callable $filter): array
{
    $cases = TaskStatus::cases();
    $result = [];

    foreach ($cases as $case) {
        if ($filter($case)) {
            $result[] = $case;
        }
    }

    if ([] === $result) {
        return TaskStatus::cases();
    }

    return $result;
}

function test(): void
{
    $values = getStatusValues();
    print_r($values);

    $high = getHighPriorities();
    print_r($high);

    $filtered = filterStatuses(fn(TaskStatus $s) => $s === TaskStatus::ACTIVE);
    print_r($filtered);
}
