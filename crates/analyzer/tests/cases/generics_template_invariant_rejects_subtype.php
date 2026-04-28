<?php

declare(strict_types=1);

class GenAnimalInv
{
}

final class GenDogInv extends GenAnimalInv
{
}

/**
 * @template T
 */
final class GenInvBox
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @param T $v */
    public function set(mixed $v): void
    {
        $this->value = $v;
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param GenInvBox<GenAnimalInv> $box
 */
function take_animal_box(GenInvBox $box): void
{
}

/**
 * @param GenInvBox<GenDogInv> $dogs
 */
function flow_dogs(GenInvBox $dogs): void
{
    /** @mago-expect analysis:less-specific-argument */
    take_animal_box($dogs);
}
