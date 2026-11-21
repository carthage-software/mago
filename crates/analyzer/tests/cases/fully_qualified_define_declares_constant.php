<?php

declare(strict_types=1);

\define('FULLY_QUALIFIED_DEFINE_PATH', '/var/www');
DEFINE('UPPERCASE_DEFINE_LIMIT', 10);

function fully_qualified_define_declares_constant(): string
{
    return FULLY_QUALIFIED_DEFINE_PATH . '/' . UPPERCASE_DEFINE_LIMIT;
}
