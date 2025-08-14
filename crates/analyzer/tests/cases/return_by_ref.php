<?php

function &get_ref(): string
{
    /** @var string $a */
    static $a = 'f';
    return $a;
}

function get_owned(): string
{
    return 'f';
}

function &get_ref_return_ref(): string
{
    return get_ref();
}

/**
 * @mago-expect analysis:invalid-return-statement
 */
function &get_ref_return_owned(): string
{
    return get_owned();
}

function get_owned_return_ref(): string
{
    return get_ref();
}

function get_owned_return_owned(): string
{
    return get_owned();
}
