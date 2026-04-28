<?php

declare(strict_types=1);

class GenAnimalCov
{
}

final class GenDogCov extends GenAnimalCov
{
}

/**
 * @template-covariant T
 */
final class GenReadOnlyBox
{
    /** @param T $value */
    public function __construct(private mixed $value)
    {
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param GenReadOnlyBox<GenAnimalCov> $box
 */
function read_animal(GenReadOnlyBox $box): GenAnimalCov
{
    return $box->get();
}

/**
 * @param GenReadOnlyBox<GenDogCov> $dogs
 */
function pass_dog_to_animal(GenReadOnlyBox $dogs): void
{
    read_animal($dogs);
}
