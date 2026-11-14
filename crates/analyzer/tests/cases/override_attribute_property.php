<?php

declare(strict_types=1);

class OverrideParent
{
    public int $value = 1;
}

interface OverrideContract
{
    public int $contract { get; }
}

class ValidOverrides extends OverrideParent implements OverrideContract
{
    #[\Override]
    public int $value = 2;

    #[\Override]
    public int $contract = 2;
}

trait OverrideTrait
{
    #[\Override]
    public int $value = 3;

    #[NotOverride]
    public int $tagged = 3;
}

class TraitOverride extends OverrideParent
{
    use OverrideTrait;
}

// @mago-expect analysis:invalid-override-attribute
class TraitWithoutParent
{
    use OverrideTrait;
}

class InvalidOverrides extends OverrideParent
{
    // @mago-expect analysis:invalid-override-attribute(2)
    #[\Override]
    public int $first = 1, $second = 2;

    // @mago-expect analysis:invalid-override-attribute
    #[\Override]
    public int $hooked { get => 1; }

    public function __construct(
        // @mago-expect analysis:invalid-override-attribute
        #[\Override]
        public int $promoted,
    ) {}
}

#[\Attribute]
final class NotOverride {}

class OverrideFinalParent
{
    final public int $locked = 1;
}

class OverrideFinalChild extends OverrideFinalParent
{
    // @mago-expect analysis:override-final-property
    #[\Override]
    public int $locked = 2;
}

// @mago-expect analysis:non-existent-class-like
class OverrideUnknownParent extends MissingOverrideParent
{
    #[\Override]
    public int $value = 1;
}
