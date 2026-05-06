<?php

/**
 */
function &get_str(): string
{
    return 'hello';
}

function take_ref(mixed &$_): void {}

/**
 */
function test(): void
{
    take_ref('hello');
}
