<?php

declare(strict_types=1);

namespace Generics\Test\Diamond;

use Generics\Example;
use Generics\IBar;
use Generics\IFoo;

/** @extends Example<IFoo> */
interface FooExample extends Example {}

/** @extends Example<IBar> */
interface BarExample extends Example {}

class DiamondReturnViolation implements FooExample, BarExample
{
    /**
     * @param IFoo|IBar $v
     * @return IFoo
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
