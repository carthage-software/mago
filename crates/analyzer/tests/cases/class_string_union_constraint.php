<?php

interface Initializer
{
}

interface DynamicInitializer
{
}

class Container
{
    /**
     * @template T of Initializer
     * @template U of DynamicInitializer
     *
     * @param class-string<T>|class-string<U> $initializerClass
     */
    public function addInitializer(string $initializerClass): self
    {
        return $this;
    }
}

class MyInitializer implements Initializer
{
}

class MyDynamicInitializer implements DynamicInitializer
{
}

$container = new Container();

$container->addInitializer(MyInitializer::class);
$container->addInitializer(MyDynamicInitializer::class);
