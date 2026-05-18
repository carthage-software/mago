<?php

declare(strict_types=1);

namespace Generics\Test\ArityInterfaceExtendsClean;

use Generics\Example;

/**
 * @extends Example<int>
 */
interface ParameterizedExample extends Example {}
