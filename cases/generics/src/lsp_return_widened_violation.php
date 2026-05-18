<?php

declare(strict_types=1);

namespace Generics\Test\LspReturnWidened;

use Generics\Animal;
use Generics\Example;

/**
 * @implements Example<Animal>
 */
class LspReturnWidenedViolation implements Example
{
    /**
     * @param Animal $v
     * @return object
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
