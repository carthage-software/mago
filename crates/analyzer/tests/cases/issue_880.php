<?php

declare(strict_types=1);

class X
{
    public function clone(): static
    {
        return clone $this;
    }
}

class XX extends X
{
    public function a_method(): string
    {
        return 'abc';
    }
}

$xx = new XX();
$a = $xx->clone()->a_method();
