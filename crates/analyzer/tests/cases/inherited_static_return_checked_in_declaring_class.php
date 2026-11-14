<?php

declare(strict_types=1);

interface InheritedStaticModelInterface
{
    public function asModel(): InheritedStaticModelInterface;
}

abstract class InheritedStaticBaseModel
{
    /** @mago-expect analysis:incompatible-return-type */
    public function asModel(): static
    {
        return $this;
    }
}

class InheritedStaticConcreteModel extends InheritedStaticBaseModel implements InheritedStaticModelInterface {}

interface InheritedStaticSelfInterface
{
    public function copy(): static;
}

abstract class InheritedStaticSelfBase
{
    public function copy(): static
    {
        return clone $this;
    }
}

final class InheritedStaticSelfConcrete extends InheritedStaticSelfBase implements InheritedStaticSelfInterface {}
