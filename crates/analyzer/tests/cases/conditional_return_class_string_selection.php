<?php

declare(strict_types=1);

class Foo {}
class Baz {}

interface Container
{
    /**
     * @template T
     * @template C
     *
     * @param T|class-string<C> $service
     *
     * @return (
     *   T is "foo" ? Foo :
     *   T is class-string<C> ? C :
     *   never-return
     * )
     */
    public function get($service);
}

function takes_baz(Baz $v): void {}

function uses_container(Container $container): void
{
    takes_baz($container->get(Baz::class));
}
