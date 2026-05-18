<?php

declare(strict_types=1);

namespace Generics\Test\DiamondContravariantSink;

use Generics\IBar;
use Generics\IFoo;
use Generics\Sink;

/** @extends Sink<IFoo> */
interface FooSink extends Sink {}

/** @extends Sink<IBar> */
interface BarSink extends Sink {}

class DiamondContravariantSinkViolation implements FooSink, BarSink
{
    /**
     * @param IFoo $v
     */
    public function sink(mixed $v): void {}
}
