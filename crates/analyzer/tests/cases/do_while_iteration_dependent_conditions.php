<?php

declare(strict_types=1);

function decode_integer(bool $has_more): void
{
    $shift = 0;

    do {
        if ($shift > 56) {
            echo 'Integer overflow.';
        }

        $shift += 7;
    } while ($has_more);
}

/**
 * @param null|positive-int $max_bytes
 */
function read_all(?int $max_bytes, string $chunk, bool $reached_end): void
{
    $to_read = $max_bytes;

    do {
        if (null !== $to_read) {
            $to_read -= strlen($chunk);
        }
    } while ((null === $to_read || $to_read > 0) && !$reached_end);
}
