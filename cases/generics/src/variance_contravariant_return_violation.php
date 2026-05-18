<?php

declare(strict_types=1);

namespace Generics\Test\VarianceContravariantReturn;

/**
 * @template-contravariant T
 */
interface BadContravariantReturn
{
    /**
     * @return T
     */
    public function produce(): mixed;
}
