<?php

declare(strict_types=1);

namespace Generics\Test\BoundExtendsViolation;

use Generics\AnimalBox;
use Generics\IFoo;

/**
 * @extends AnimalBox<IFoo>
 */
interface FooBox extends AnimalBox {}
