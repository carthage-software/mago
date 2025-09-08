<?php

/**
 * @param array<string, int> $test
 *
 * @mago-expect analysis:mixed-argument
 * @mago-expect analysis:undefined-string-array-index
 */
function x(array $test): void
{
    if (isset($test['test'])) {
        echo $test['asdf'];
    }
}
