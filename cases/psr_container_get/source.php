<?php

declare(strict_types=1);

namespace Psr\Container;

/**
 * PSR-11 ContainerInterface stub for testing.
 */
interface ContainerInterface
{
    /**
     * @param string $id
     * @return mixed
     */
    public function get(string $id): mixed;

    public function has(string $id): bool;
}

class Example
{
    public function doWork(): string
    {
        return 'working';
    }
}

function accepts_example(Example $service): void
{
    $service->doWork();
}

function test_container_get(ContainerInterface $container): void
{
    $container->get(Example::class)->doWork();

    $example = $container->get(Example::class);
    accepts_example($example);
}

function test_non_class_string(ContainerInterface $container, string $serviceId): void
{
    accepts_example($container->get('some.service.id'));

    $container->get('some.service.id')->doWork();

    accepts_example($container->get($serviceId));

    $container->get($serviceId)->doWork();
}
