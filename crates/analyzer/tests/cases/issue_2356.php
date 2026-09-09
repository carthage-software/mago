<?php

declare(strict_types=1);

/** @throws LogicException */
function chainCaughtThrowable(): void
{
    try {
    } catch (Throwable $exception) {
        // PDOException can supply a string code, so this can throw a TypeError at runtime.
        // @mago-expect analysis:possibly-invalid-argument
        throw new LogicException('Encountered unexpected exception', $exception->getCode(), $exception);
    }
}

function throwableCode(Throwable $exception): int|string
{
    return $exception->getCode();
}

function exceptionCode(Exception $exception): int|string
{
    return $exception->getCode();
}

function pdoExceptionCode(PDOException $exception): int|string
{
    return $exception->getCode();
}

function chainException(Exception $exception): LogicException
{
    // @mago-expect analysis:possibly-invalid-argument
    return new LogicException($exception->getMessage(), $exception->getCode(), $exception);
}

function chainPdoException(PDOException $exception): LogicException
{
    // @mago-expect analysis:possibly-invalid-argument
    return new LogicException($exception->getMessage(), $exception->getCode(), $exception);
}

function pdoExceptionStringCode(PDOException $exception): string
{
    // PDOException codes can also be integers.
    // @mago-expect analysis:invalid-return-statement
    return $exception->getCode();
}

function chainError(Error $error): LogicException
{
    return new LogicException($error->getMessage(), $error->getCode(), $error);
}

function chainTypeError(TypeError $error): LogicException
{
    return new LogicException($error->getMessage(), $error->getCode(), $error);
}

function chainWithIntegerCode(Throwable $exception): LogicException
{
    $code = $exception->getCode();
    if (is_int($code)) {
        return new LogicException($exception->getMessage(), $code, $exception);
    }

    return new LogicException($exception->getMessage() . ' (' . $code . ')', 0, $exception);
}

function chainWithCastCode(Throwable $exception): LogicException
{
    return new LogicException($exception->getMessage(), (int) $exception->getCode(), $exception);
}

function chainWithDefaultCode(Throwable $exception): LogicException
{
    return new LogicException($exception->getMessage(), previous: $exception);
}
