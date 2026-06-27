<?php

declare(strict_types=1);

namespace Foo;

interface ContainerInterface
{
    /**
     * @return int
     */
    public function get(): ?int;
}

function returns_narrowed(ContainerInterface $c): int
{
    return $c->get();
}

class Service
{
    /**
     * @param int $value
     */
    public function accept(?int $value): bool
    {
        return $value === null;
    }
}

function param_widened(Service $service): void
{
    $service->accept(null);
    $service->accept(1);
}
