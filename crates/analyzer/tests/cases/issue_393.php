<?php

/**
 * @param array<string, int> $asdf
 *
 */
function x(array $asdf): void
{
    if (isset($asdf['ayy'])) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        echo $asdf['qwer']; // error
    }
}

/**
 * @param array<string, int> $asdf
 */
function y(array $asdf): void
{
    if (isset($asdf['qwer'])) {
        echo $asdf['qwer']; // no error
    }
}
