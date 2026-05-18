<?php

declare(strict_types=1);

namespace Generics\Test\DiamondCovariantProducerClean;

use Generics\IBar;
use Generics\IFoo;
use Generics\Producer;

/** @extends Producer<IFoo> */
interface FooProducerClean extends Producer {}

/** @extends Producer<IBar> */
interface BarProducerClean extends Producer {}

class DiamondCovariantProducerClean implements FooProducerClean, BarProducerClean
{
    /**
     * @return IFoo&IBar
     */
    public function produce(): mixed
    {
        throw new \LogicException('stub');
    }
}
