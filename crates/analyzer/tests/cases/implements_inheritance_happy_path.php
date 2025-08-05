<?php

interface Loggable
{
}

class User implements Loggable
{
}

/** @template T */
interface Repository
{
}

/**
 * @implements Repository<User>
 */
class UserRepository implements Repository
{
}

/**
 * @template TEntity
 * @implements Repository<TEntity>
 */
class GenericRepository implements Repository
{
}

/**
 * @inheritors PermittedImplementor
 */
interface RestrictedInterface
{
}

class PermittedImplementor implements RestrictedInterface
{
}

interface RequiredForB
{
}

/**
 * @require-implements RequiredForB
 */
interface InterfaceB
{
}

class FulfillsRequirement implements InterfaceB, RequiredForB
{
}

/**
 * @template T
 *
 * @enum-interface
 */
interface HasValue
{
    /**
     * @return T
     */
    public function getValue(): mixed;
}

/**
 * @implements HasValue<string>
 */
enum Status implements HasValue
{
    case PENDING;
    case COMPLETED;

    public function getValue(): string
    {
        return $this->name;
    }
}
