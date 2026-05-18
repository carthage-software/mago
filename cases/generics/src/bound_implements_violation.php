<?php

declare(strict_types=1);

namespace Generics\Test\BoundImplementsViolation;

use Generics\Animal;
use Generics\AnimalBox;
use Generics\IFoo;

/**
 * @implements AnimalBox<IFoo>
 */
class BoundImplementsViolation implements AnimalBox
{
    public function unwrap(): Animal
    {
        throw new \LogicException('stub');
    }
}
