<?php

/**
 * @return non-empty-string
 */
function test_non_empty_bin2hex(): string
{
    return bin2hex('a');
}

/**
 * @return ''
 */
function test_empty_bin2hex(): string
{
    return bin2hex('');
}
