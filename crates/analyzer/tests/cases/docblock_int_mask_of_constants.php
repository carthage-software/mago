<?php

declare(strict_types=1);

final class FlagsP
{
    public const READ = 1;
    public const WRITE = 2;
    public const EXECUTE = 4;
}

/**
 * @param int-mask-of<FlagsP::*> $f
 */
function takeFlagsP(int $f): void
{
    echo $f;
}

takeFlagsP(FlagsP::READ | FlagsP::WRITE);
takeFlagsP(0);
takeFlagsP(7);
/** @mago-expect analysis:invalid-argument */
takeFlagsP(8);
