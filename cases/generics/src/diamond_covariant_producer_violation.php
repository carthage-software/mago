<?php

declare(strict_types=1);

namespace Generics\Test\DiamondCovariantProducer;

use Generics\IBar;
use Generics\IFoo;
use Generics\Producer;

/** @extends Producer<IFoo> */
interface FooProducer extends Producer {}

/** @extends Producer<IBar> */
interface BarProducer extends Producer {}

class DiamondCovariantProducerViolation implements FooProducer, BarProducer
{
    /**
     * @return IFoo
     */
    public function produce(): mixed
    {
        throw new \LogicException('stub');
    }
}
