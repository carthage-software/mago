<?php

declare(strict_types=1);

interface ContainerInterface
{
    public const RUNTIME_EXCEPTION_ON_INVALID_REFERENCE = 0;
    public const EXCEPTION_ON_INVALID_REFERENCE = 1;
    public const NULL_ON_INVALID_REFERENCE = 2;
    public const IGNORE_ON_INVALID_REFERENCE = 3;
    public const IGNORE_ON_UNINITIALIZED_REFERENCE = 4;

    public function set(string $id, ?object $service): void;

    /**
     * @template C of object
     * @template B of self::*_REFERENCE
     *
     * @param string|class-string<C> $id
     * @param B                      $invalidBehavior
     *
     * @return ($id is class-string<C> ? (B is 0|1 ? C : C|null) : (B is 0|1 ? object : object|null))
     *
     * @throws ServiceCircularReferenceException When a circular reference is detected
     * @throws ServiceNotFoundException          When the service is not defined
     *
     * @see Reference
     */
    public function get(string $id, int $invalidBehavior = self::EXCEPTION_ON_INVALID_REFERENCE): ?object;
}

class X {}

function run1(ContainerInterface $container): X
{
    return $container->get(X::class, ContainerInterface::RUNTIME_EXCEPTION_ON_INVALID_REFERENCE);
}

function run2(ContainerInterface $container): X
{
    return $container->get(X::class, ContainerInterface::EXCEPTION_ON_INVALID_REFERENCE);
}

function run3(ContainerInterface $container): ?X
{
    return $container->get(X::class, ContainerInterface::NULL_ON_INVALID_REFERENCE);
}
