<?php

declare(strict_types=1);

interface ACG
{
    public function a(): int;
}

interface BCG
{
    public function b(): int;
}

interface CCG
{
    public function c(): int;
}

/**
 * @param ACG&BCG&CCG $x
 */
function takeTripleCG(object $x): int
{
    return $x->a() + $x->b() + $x->c();
}

final class HostCG implements ACG, BCG, CCG
{
    public function a(): int
    {
        return 1;
    }

    public function b(): int
    {
        return 2;
    }

    public function c(): int
    {
        return 3;
    }
}

echo takeTripleCG(new HostCG());
