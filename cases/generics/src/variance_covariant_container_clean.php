<?php

declare(strict_types=1);

namespace Generics\Test\VarianceCovariantClean;

use Generics\Animal;
use Generics\Dog;

/**
 * @template-covariant T of Animal
 */
interface CovariantContainer
{
    /**
     * @return T
     */
    public function get(): Animal;
}

/**
 * @implements CovariantContainer<Dog>
 */
final class DogContainer implements CovariantContainer
{
    public function __construct(
        private Dog $dog,
    ) {}

    public function get(): Animal
    {
        return $this->dog;
    }
}
