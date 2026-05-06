<?php

declare(strict_types=1);

/** @experimental */
class ExperimentalClass
{
    public function method(): void {}
}

/** @experimental */
function experimental_function(): void {}

/** @experimental */
interface ExperimentalInterface {}

/** @experimental */
trait ExperimentalTrait {}

/** @experimental */
const EXPERIMENTAL_CONST = 42;

new ExperimentalClass();

ExperimentalClass::class;

experimental_function();

EXPERIMENTAL_CONST;

class StableClass implements ExperimentalInterface {}

class AnotherStable
{
    use ExperimentalTrait;
}

class StableChild extends ExperimentalClass {}

/** @experimental */
function uses_experimental_ok(): void
{
    new ExperimentalClass();
    experimental_function();
}

/** @experimental */
class ExperimentalConsumer
{
    public function method(): void
    {
        new ExperimentalClass();
        experimental_function();
    }
}

/** @experimental */
class ExperimentalChild extends ExperimentalClass implements ExperimentalInterface
{
    use ExperimentalTrait;
}

class StableClassWithExperimentalUsage
{
    public function stableMethod(): void
    {
        new ExperimentalClass();
    }

    /** @experimental */
    public function experimentalMethod(): void
    {
        // OK — method is experimental
        new ExperimentalClass();
    }
}
