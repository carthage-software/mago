<?php

declare(strict_types=1);

class AnimalQ
{
}

class DogQ extends AnimalQ
{
}

/**
 * @param class-string<AnimalQ> $cls
 *
 * @return class-string<AnimalQ>
 */
function createQ(string $cls): string
{
    return $cls;
}

echo createQ(DogQ::class);

/** @mago-expect analysis:invalid-argument */
echo createQ(stdClass::class);
