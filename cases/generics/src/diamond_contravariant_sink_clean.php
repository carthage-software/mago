<?php

declare(strict_types=1);

namespace Generics\Test\DiamondContravariantSinkClean;

use Generics\IBar;
use Generics\IFoo;
use Generics\Sink;

/** @extends Sink<IFoo> */
interface FooSinkClean extends Sink {}

/** @extends Sink<IBar> */
interface BarSinkClean extends Sink {}

class DiamondContravariantSinkClean implements FooSinkClean, BarSinkClean
{
    /**
     * @param IFoo|IBar $v
     */
    public function sink(mixed $v): void {}
}
