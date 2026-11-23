<?php

declare(strict_types=1);

function unreachable_else_that_returns(int $value): int
{
    /** @mago-expect analysis:redundant-type-comparison,redundant-condition */
    if (is_int($value)) {
        return 1;
    } else {
        /** @mago-expect analysis:unreachable-else-clause */
        return 2;
    }
}

final class UnreachableElseHolder
{
    /**
     * @var array<string, int>
     */
    public array $values = [];

    /**
     * @return array<string, int>|false
     */
    public function values(): array|false
    {
        /** @mago-expect analysis:redundant-condition */
        if (isset($this->values)) {
            return $this->values;
        } else {
            /** @mago-expect analysis:unreachable-else-clause */
            return false;
        }
    }
}
