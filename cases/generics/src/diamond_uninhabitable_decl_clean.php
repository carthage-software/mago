<?php

declare(strict_types=1);

namespace Generics\Test\DiamondUninhabitableDeclClean;

use Generics\IBar;
use Generics\IFoo;
use Generics\Producer;

/** @extends Producer<IFoo> */
interface FooProducer extends Producer {}

/** @extends Producer<IBar> */
interface BarProducer extends Producer {}

interface FlexObjectProducer extends FooProducer, BarProducer {}
