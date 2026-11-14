<?php

declare(strict_types=1);

if (PHP_OS_FAMILY === 'Windows') {
    define('SEP', ';');
} else {
    define('SEP', ':');
}

function family(): string
{
    return SEP === ':' ? 'unix' : 'windows';
}

echo family(), "\n";
