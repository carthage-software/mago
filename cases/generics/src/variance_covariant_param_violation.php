<?php

declare(strict_types=1);

namespace Generics\Test\VarianceCovariantParam;

/**
 * @template-covariant T
 */
interface BadCovariantParam
{
    /**
     * @param T $value
     */
    public function consume(mixed $value): void;
}
