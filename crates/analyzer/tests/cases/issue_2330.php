<?php

declare(strict_types=1);

namespace Issue2330;

/** @consistent-constructor */
abstract class Base {}

final class Concrete extends Base {}

final class Probe
{
    /**
     * @template T of Base
     *
     * @param class-string<T> $class
     *
     * @return T
     */
    public function fromClassString(string $class): Base
    {
        return new $class();
    }

    /**
     * @template T of Base
     *
     * @param T $original
     *
     * @return T
     */
    public function fromInstance(Base $original): Base
    {
        $class = $original::class;

        return new $class();
    }

    /**
     * @template T of Base
     *
     * @param T $original
     *
     * @return class-string<T>
     */
    public function classStringFromInstance(Base $original): string
    {
        return $original::class;
    }
}

function takeConcrete(Concrete $_): void {}

/** @param class-string<Concrete> $_ */
function takeConcreteClassString(string $_): void {}

$probe = new Probe();
takeConcrete($probe->fromClassString(Concrete::class));
takeConcrete($probe->fromInstance(new Concrete()));
takeConcreteClassString($probe->classStringFromInstance(new Concrete()));
