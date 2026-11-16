<?php

declare(strict_types=1);

/** @template T of int */
final class Filler
{
    /**
     * @param-out T $s
     *
     * @throws Exception
     */
    public function fill(mixed &$s): void
    {
        throw new Exception();
    }
}

/** @template T */
final class Checker
{
    /**
     * @psalm-assert T $value
     *
     * @throws Exception
     */
    public function check(mixed $value): void
    {
        throw new Exception();
    }
}

/** @template T of Throwable */
final class Thrower
{
    /**
     * @throws T
     * @throws Exception
     */
    public function run(): void
    {
        throw new Exception();
    }
}

/**
 * @template T of object
 *
 * @mixin T
 */
final class Proxy
{
}

/**
 * @template T
 *
 * @property T $value
 */
final class Bag
{
    public function __get(string $name): mixed
    {
        return null;
    }
}
