<?php

/**
 * @mago-expect analysis:invalid-return-statement
 */
function &get_str(): string
{
    return 'hello';
}

function take_ref(mixed &$_): void
{
}

/**
 * @mago-expect analysis:invalid-pass-by-reference
 */
function test(): void
{
    take_ref('hello');
}
