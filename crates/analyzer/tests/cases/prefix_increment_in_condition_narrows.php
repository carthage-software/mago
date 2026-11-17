<?php

declare(strict_types=1);

/**
 * @param positive-int $value
 */
function take_positive(int $value): void
{
}

/**
 * @param int<10, max> $value
 */
function take_ten_or_more(int $value): void
{
}

function prefix_decrement(int $value): void
{
    if (--$value > 0) {
        take_positive($value);
    }
}

final class PrefixIncrementCounter
{
    public int $count = 0;

    public function increment(): void
    {
        if (++$this->count > 0) {
            take_positive($this->count);
        }
    }
}

function prefix_increment_loop(): void
{
    $try = 8;
    while (++$try < 10) {
        echo $try;
    }

    take_ten_or_more($try);
}
