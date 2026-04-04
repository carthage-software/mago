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

/** @mago-expect analysis:experimental-usage */
new ExperimentalClass();

/** @mago-expect analysis:experimental-usage */
/** @mago-expect analysis:unused-statement */
ExperimentalClass::class;

/** @mago-expect analysis:experimental-usage */
experimental_function();

/** @mago-expect analysis:experimental-usage */
/** @mago-expect analysis:unused-statement */
EXPERIMENTAL_CONST;

/** @mago-expect analysis:experimental-usage */
class StableClass implements ExperimentalInterface {}

/** @mago-expect analysis:experimental-usage */
class AnotherStable
{
    use ExperimentalTrait;
}

/** @mago-expect analysis:experimental-usage */
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
        /** @mago-expect analysis:experimental-usage */
        new ExperimentalClass();
    }

    /** @experimental */
    public function experimentalMethod(): void
    {
        // OK — method is experimental
        new ExperimentalClass();
    }
}
