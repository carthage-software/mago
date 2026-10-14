<?php

declare(strict_types=1);

class AnimalBW
{
}

class DogBW extends AnimalBW
{
}

/** @param class-string<AnimalBW> $cls */
function takeAnimalBW(string $cls): void
{
    echo $cls;
}

takeAnimalBW(DogBW::class);
takeAnimalBW('DogBW');
takeAnimalBW('\DogBW');

/** @mago-expect analysis:invalid-argument */
takeAnimalBW(stdClass::class);
