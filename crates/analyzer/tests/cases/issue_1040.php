<?php

declare(strict_types=1);

namespace App;

use Override;

/**
 * @template TConfiguration of array<array-key, mixed>
 */
interface ContainerInterface
{
    /** @param TConfiguration $configuration */
    public function get(array $configuration): string;
}

/**
 * @phpstan-type MyConfiguration array{name: string, value: int}
 */
class Calculator
{
    /** @param MyConfiguration $configuration */
    public function calculate(array $configuration): string
    {
        return $configuration['name'];
    }
}

/**
 * @phpstan-import-type MyConfiguration from Calculator
 *
 * @template-implements ContainerInterface<MyConfiguration>
 */
class MyContainer implements ContainerInterface
{
    #[Override]
    public function get(array $configuration): string
    {
        return $configuration['name'];
    }
}
