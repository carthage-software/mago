<?php

declare(strict_types=1);

final class Budget
{
    public int $limit = 0;
}

final class Dto
{
    public ?Budget $budget = null;
}

function limit_when_not_null(?Dto $meta): int
{
    if (!is_null($meta?->budget)) {
        return $meta->budget->limit;
    }

    return 0;
}

function limit_after_null_return(?Dto $meta): int
{
    if (is_null($meta?->budget)) {
        return 0;
    }

    return $meta->budget->limit;
}

function limit_in_else(?Dto $meta): int
{
    if (is_null($meta?->budget)) {
        return 0;
    } else {
        return $meta->budget->limit;
    }
}
