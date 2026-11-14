<?php

declare(strict_types=1);

final class Config
{
    public function __construct(public int $n = 1) {}
}

define('DEFINED', new Config());

/** @var Config */
const DECLARED = new Config(2);

function total(): int
{
    return DEFINED->n + DECLARED->n;
}
