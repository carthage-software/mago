<?php

declare(strict_types=1);

namespace Generics\Test\LspSingleClean;

use Generics\Animal;
use Generics\Dog;
use Generics\Example;

/**
 * @implements Example<Animal>
 */
class LspSingleClean implements Example
{
    /**
     * @param Animal $v
     * @return Dog
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
