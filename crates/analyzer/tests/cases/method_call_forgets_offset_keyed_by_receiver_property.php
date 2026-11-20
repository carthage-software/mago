<?php

declare(strict_types=1);

final class OffsetKeyHolder
{
    public string $key = 'a';

    public function changeKey(): void
    {
        $this->key = 'b';
    }
}

/**
 * @param array<string, int|null> $values
 */
function method_call_forgets_offset_narrowing(OffsetKeyHolder $holder, array $values): int
{
    if ($values[$holder->key] !== null) {
        $holder->changeKey();
        if ($values[$holder->key] !== null) {
            return 1;
        }
    }

    return 0;
}
