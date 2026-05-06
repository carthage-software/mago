<?php

declare(strict_types=1);

final class StaticHelper
{
    public static function help(string $s): string
    {
        return $s;
    }
}

$callable = [StaticHelper::class, 'help'];
$callable('hi');
