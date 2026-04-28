<?php

declare(strict_types=1);

class AnimalR
{
}

class StringContainerR
{
}

/**
 * @param class-string<AnimalR> $cls
 */
function takeAnimalR(string $cls): void
{
    echo $cls;
}

/** @mago-expect analysis:invalid-argument */
takeAnimalR(StringContainerR::class);
