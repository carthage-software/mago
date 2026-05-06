<?php

declare(strict_types=1);

/**
 * @throws \RuntimeException
 */
function test_total_length_range(string $data, int $length): void
{
    $totalLength = 0;
    $currentOffset = 0;

    while (true) {
        $lengthByte = ord($data[$currentOffset++]);

        if ($lengthByte === 0) {
            break;
        }

        if (($lengthByte & 0xC0) === 0xC0) {
            $currentOffset++;
        } elseif ($lengthByte > 63) {
            throw new \RuntimeException('too long');
        } else {
            $totalLength += $lengthByte + 1;
            if ($totalLength > 253) {
                throw new \RuntimeException('name too long');
            }
        }
    }
}
