<?php

declare(strict_types=1);

namespace Generics\Test\DiamondParamNarrowed;

use Generics\Example;
use Generics\IBar;
use Generics\IFoo;

/** @extends Example<IFoo> */
interface FooExample extends Example {}

/** @extends Example<IBar> */
interface BarExample extends Example {}

class DiamondParamNarrowedViolation implements FooExample, BarExample
{
    /**
     * @param IFoo $v
     * @return IFoo&IBar
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
