<?php

declare(strict_types=1);

namespace Generics\Test\DiamondUninhabitable;

use Generics\Producer;

/** @extends Producer<int> */
interface IntProducer extends Producer {}

/** @extends Producer<string> */
interface StringProducer extends Producer {}

class DiamondUninhabitableViolation implements IntProducer, StringProducer
{
    /**
     * @return int
     */
    public function produce(): mixed
    {
        throw new \LogicException('stub');
    }
}
