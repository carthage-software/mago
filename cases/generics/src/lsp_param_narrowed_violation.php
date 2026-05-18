<?php

declare(strict_types=1);

namespace Generics\Test\LspParamNarrowed;

use Generics\Animal;
use Generics\Dog;
use Generics\Example;

/**
 * @implements Example<Animal>
 */
class LspParamNarrowedViolation implements Example
{
    /**
     * @param Dog $v
     * @return Animal
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
