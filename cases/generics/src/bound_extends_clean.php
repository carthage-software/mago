<?php

declare(strict_types=1);

namespace Generics\Test\BoundExtendsClean;

use Generics\AnimalBox;
use Generics\Dog;

/**
 * @extends AnimalBox<Dog>
 */
interface DogBox extends AnimalBox {}
