<?php

declare(strict_types=1);

/**
 * @consistent-constructor
 */
abstract class Handler
{
    final public function __construct() {}

    abstract public function handle(): void;
}

final class ConcreteHandlerA extends Handler
{
    #[Override]
    public function handle(): void
    {
        echo "Handler A\n";
    }
}

final class ConcreteHandlerB extends Handler
{
    #[Override]
    public function handle(): void
    {
        echo "Handler B\n";
    }
}

abstract class AbstractFactory
{
    /**
     * @return class-string<Handler>
     */
    abstract protected function getHandlerClass(): string;

    public function createHandler(): Handler
    {
        $handlerClass = $this->getHandlerClass();

        return new $handlerClass();
    }
}

class FactoryA extends AbstractFactory
{
    /**
     * @return class-string<Handler>
     */
    #[Override]
    protected function getHandlerClass(): string
    {
        return ConcreteHandlerA::class;
    }
}

class FactoryB extends AbstractFactory
{
    /**
     * @return class-string<Handler>
     */
    #[Override]
    protected function getHandlerClass(): string
    {
        return ConcreteHandlerB::class;
    }
}

function test(): void
{
    $factoryA = new FactoryA();
    $handlerA = $factoryA->createHandler();
    $handlerA->handle();

    $factoryB = new FactoryB();
    $handlerB = $factoryB->createHandler();
    $handlerB->handle();
}
