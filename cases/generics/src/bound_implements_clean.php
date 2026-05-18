<?php

declare(strict_types=1);

namespace Generics\Test\BoundImplementsClean;

use Generics\Animal;
use Generics\AnimalBox;
use Generics\Dog;

/**
 * @implements AnimalBox<Dog>
 */
class BoundImplementsClean implements AnimalBox
{
    /**
     * @return Dog
     */
    public function unwrap(): Animal
    {
        throw new \LogicException('stub');
    }
}
