<?php

declare(strict_types=1);

final class TernaryClauseValue
{
    public function get(): int
    {
        return 1;
    }
}

function ternary_keeps_or_clause(?TernaryClauseValue $a, ?TernaryClauseValue $b): int
{
    return $a !== null || $b !== null ? ($a === null ? $b->get() : 1) : 0;
}

function match_arm_keeps_or_clause(?TernaryClauseValue $a, ?TernaryClauseValue $b): int
{
    return match (true) {
        $a !== null || $b !== null => $a === null ? $b->get() : 1,
        default => 0,
    };
}
