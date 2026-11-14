<?php

declare(strict_types=1);

define('SOME_PATH', (string) getenv('SOME_PATH'));
define('NO_DEBUG', !getenv('DEBUG'));
define('IS_MODERN', PHP_VERSION_ID >= 80000);

function takes_string(string $s): void
{
    echo $s;
}

function takes_bool(bool $b): void
{
    echo $b ? 'y' : 'n';
}

takes_string(SOME_PATH);
takes_bool(NO_DEBUG);
takes_bool(IS_MODERN);
