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

function take_ref(string &$str): void
{
    $str = 'b';
}

function ref_as_ref(): void
{
    take_ref(get_ref());
}

/**
 * @mago-expect analysis:invalid-pass-by-reference
 */
function owned_as_ref(): void
{
    take_ref(get_owned());
}

ref_as_ref();
owned_as_ref();
