<?php

declare(strict_types=1);

/** @param list<int> $items */
function counts(array $items): int
{
    $counter = ['seen' => 0];
    $first = 0;
    foreach ($items as $_item) {
        $previous = $counter['seen']++;
        if ($previous === 0) {
            $first++;
        }
    }

    return $first;
}

final class Counter
{
    public int $value = 0;

    public function take(): int
    {
        $previous = $this->value--;

        return $previous === 0 ? 0 : $previous;
    }
}
