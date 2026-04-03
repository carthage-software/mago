<?php

/**
 * @param array<string, int> $test
 *
 */
function x(array $test): void
{
    if (isset($test['test'])) {
        echo $test['asdf'];
    }
}
