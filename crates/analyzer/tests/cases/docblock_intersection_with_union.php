<?php

declare(strict_types=1);

interface AlphaBN
{
    public function alpha(): int;
}

interface BetaBN
{
    public function beta(): int;
}

interface GammaBN
{
    public function gamma(): int;
}

/**
 * @param (AlphaBN|BetaBN)&GammaBN $x
 */
function takeMixedBN(object $x): int
{
    return $x->gamma();
}

final class HostBN implements AlphaBN, GammaBN
{
    public function alpha(): int
    {
        return 1;
    }

    public function gamma(): int
    {
        return 3;
    }
}

echo takeMixedBN(new HostBN());
