<?php

declare(strict_types=1);

/**
 * @consistent-constructor
 */
abstract class BaseClass
{
    final public function __construct() {}

    abstract public function getValue(): string;
}

final class ConcreteImpl extends BaseClass
{
    public function getValue(): string
    {
        return 'value';
    }
}

class Factory
{
    /**
     * @param class-string<BaseClass> $className
     */
    public function create(string $className): BaseClass
    {
        return new $className();
    }
}

/**
 * @param class-string<BaseClass> $class
 */
function createInstance(string $class): BaseClass
{
    return new $class();
}

function test(): void
{
    $factory = new Factory();
    $instance = $factory->create(ConcreteImpl::class);
    echo $instance->getValue();

    $instance2 = createInstance(ConcreteImpl::class);
    echo $instance2->getValue();
}
