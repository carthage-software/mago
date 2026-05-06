<?php

class AppException extends \Exception
{
    /**
     * @throws AppException
     */
    public static function create(string $message): never
    {
        throw new self($message);
    }
}

/**
 * @throws \RuntimeException
 */
function always_throws(): never
{
    throw new \RuntimeException('always');
}

/**
 * @throws AppException
 */
function test_throw_never_static_method(): void
{
    throw AppException::create('error');
}

/**
 * @throws \RuntimeException
 */
function test_throw_never_function(): void
{
    throw always_throws();
}
