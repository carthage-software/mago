<?php

declare(strict_types=1);

function returns_null(): null
{
    return null;
}

function returns_mixed(): mixed
{
    return 1;
}

returns_null()?->test;
returns_mixed()?->test;

returns_null()?->foo();
returns_mixed()?->foo();
