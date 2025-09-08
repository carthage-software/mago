<?php

/**
 * @param array<string, int> $asdf
 *
 * @mago-expect analysis:undefined-string-array-index
 * @mago-expect analysis:mixed-argument
 */
function x(array $asdf): void
{
    if (isset($asdf['ayy'])) {
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
