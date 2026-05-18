<?php

declare(strict_types=1);

namespace Generics\Test\VarianceContravariantClean;

use Generics\Animal;

/**
 * @template-contravariant T of Animal
 */
interface Consumer
{
    /**
     * @param T $value
     */
    public function consume(Animal $value): void;
}

/**
 * @implements Consumer<Animal>
 */
final class AnimalConsumer implements Consumer
{
    public function consume(Animal $value): void {}
}
