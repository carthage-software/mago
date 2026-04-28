<?php

declare(strict_types=1);

final class Config
{
    public const string NAME = 'mago';
}

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

takes_non_empty(Config::NAME);
