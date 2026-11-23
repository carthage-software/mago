<?php

declare(strict_types=1);

final class LoopNullsafeCoalesce
{
    public function ready(): bool
    {
        return true;
    }
}

function loop_nullsafe_coalesce_while(?LoopNullsafeCoalesce $value): LoopNullsafeCoalesce
{
    while ($value?->ready() ?? false) {
        return $value;
    }

    return new LoopNullsafeCoalesce();
}

function loop_nullsafe_coalesce_for(?LoopNullsafeCoalesce $value): LoopNullsafeCoalesce
{
    for (; $value?->ready() ?? false;) {
        return $value;
    }

    return new LoopNullsafeCoalesce();
}
