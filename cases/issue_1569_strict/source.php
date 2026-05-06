<?php

declare(strict_types=1);

/**
 * @param array<string, int> $o
 */
function test_consistent_warn_expected(array $o): void
{
    if ($o['x_1'] || $o['x_2']) {
        if ($o['x_1']) {
            echo 'x_1';
        }

        if ($o['x_2']) {
            echo 'x_2';
        }
    }
}
