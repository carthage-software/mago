<?php

declare(strict_types=1);

namespace Generics\Test\Diamond;

use Generics\Example;
use Generics\IBar;
use Generics\IFoo;

/** @extends Example<IFoo> */
interface FooExampleClean extends Example {}

/** @extends Example<IBar> */
interface BarExampleClean extends Example {}

class DiamondReturnClean implements FooExampleClean, BarExampleClean
{
    /**
     * @param IFoo|IBar $v
     * @return IFoo&IBar
     */
    public function produce(mixed $v): mixed
    {
        throw new \LogicException('stub');
    }
}
