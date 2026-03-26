<?php

declare(strict_types=1);

interface FactoryInterface
{
    public function create(): object;
}

class MyFactory implements FactoryInterface
{
    public function create(): object
    {
        return new \stdClass();
    }
}

class Service
{
    public function __construct(
        private readonly MyFactory $factory,
    ) {}

    public function doSomething(): object
    {
        $handler = new class($this->factory) {
            public function __construct(
                private readonly MyFactory $factory,
            ) {}

            public function handle(): object
            {
                return $this->factory->create();
            }
        };

        return $handler->handle();
    }
}
