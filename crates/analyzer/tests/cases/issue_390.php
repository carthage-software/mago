<?php

/**
 * @param array<string, int> $test
 *
 */
function x(array $test): void
{
    if (isset($test['test'])) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        echo $test['asdf'];
    }
}
