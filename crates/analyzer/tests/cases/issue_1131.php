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

returns_null()?->test; // @mago-expect analysis:unused-statement
returns_mixed()?->test; // @mago-expect analysis:unused-statement,mixed-property-access

returns_null()?->foo();
returns_mixed()?->foo(); // @mago-expect analysis:mixed-method-access
