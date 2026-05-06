<?php

declare(strict_types=1);

trait HelloU
{
    public function hello(): string
    {
        return 'hello';
    }
}

/**
 * @param trait-string<HelloU> $cls
 */
function takeTraitStringU(string $cls): void
{
    echo $cls;
}

takeTraitStringU(HelloU::class);
