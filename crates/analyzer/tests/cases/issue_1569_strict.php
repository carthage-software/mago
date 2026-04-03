<?php

declare(strict_types=1);

/**
 * @param array<string, int> $o
 */
function test_consistent_warn_expected(array $o): void
{
    /** @mago-expect analysis:possibly-undefined-string-array-index,possibly-undefined-string-array-index */
    if ($o['x_1'] || $o['x_2']) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        if ($o['x_1']) {
            echo 'x_1';
        }

        /** @mago-expect analysis:possibly-undefined-string-array-index */
        if ($o['x_2']) {
            echo 'x_2';
        }
    }
}
