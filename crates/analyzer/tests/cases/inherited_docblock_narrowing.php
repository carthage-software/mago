<?php

declare(strict_types=1);

/**
 * @template TEntity of object
 */
abstract class Factory
{
    /**
     * @return TEntity|null
     */
    abstract public function createEntityFromDto(): ?object;
}

/**
 * @extends Factory<stdClass>
 */
class UserFactory extends Factory
{
    public function createEntityFromDto(): object
    {
        return new stdClass();
    }
}

/**
 * @template T
 */
abstract class NullableParent
{
    /** @return T|null */
    abstract public function getValue(): ?object;
}

/** @extends NullableParent<stdClass> */
class NonNullableChild extends NullableParent
{
    public function getValue(): object
    {
        return new stdClass();
    }
}

/** @extends NullableParent<stdClass> */
class StillNullableChild extends NullableParent
{
    public function getValue(): ?object
    {
        return null;
    }
}

abstract class GrandParentClass
{
    /** @return object|null */
    abstract public function create(): ?object;
}

abstract class ParentLevel extends GrandParentClass {}

class GrandChild extends ParentLevel
{
    public function create(): object
    {
        return new stdClass();
    }
}

function use_stdclass(stdClass $obj): void
{
    use_stdclass($obj);
}

$factory = new UserFactory();
use_stdclass($factory->createEntityFromDto());
