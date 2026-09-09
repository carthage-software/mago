<?php

declare(strict_types=1);

/** @throws LogicException */
function chainCaughtThrowable(): void
{
    try {
    } catch (Throwable $exception) {
        throw new LogicException('Encountered unexpected exception', $exception->getCode(), $exception);
    }
}

function throwableCode(Throwable $exception): int
{
    return $exception->getCode();
}

function exceptionCode(Exception $exception): int
{
    return $exception->getCode();
}

function pdoExceptionCode(PDOException $exception): int|string
{
    return $exception->getCode();
}

function chainException(Exception $exception): LogicException
{
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

function chainWithIntegerCode(PDOException $exception): LogicException
{
    $code = $exception->getCode();
    if (is_int($code)) {
        return new LogicException($exception->getMessage(), $code, $exception);
    }

    return new LogicException($exception->getMessage() . ' (' . $code . ')', 0, $exception);
}

function chainWithCastCode(PDOException $exception): LogicException
{
    return new LogicException($exception->getMessage(), (int) $exception->getCode(), $exception);
}

function chainWithDefaultCode(Throwable $exception): LogicException
{
    return new LogicException($exception->getMessage(), previous: $exception);
}

function runtimeExceptionCode(RuntimeException $exception): int
{
    return $exception->getCode();
}

final class CustomException extends Exception {}

function customExceptionCode(CustomException $exception): int
{
    return $exception->getCode();
}

final class CustomPdoException extends PDOException {}

function chainCustomPdoException(CustomPdoException $exception): LogicException
{
    // @mago-expect analysis:possibly-invalid-argument
    return new LogicException($exception->getMessage(), $exception->getCode(), $exception);
}

function nullableThrowableCode(?Throwable $exception): ?int
{
    return $exception?->getCode();
}

function nullablePdoExceptionCode(?PDOException $exception): int|string|null
{
    return $exception?->getCode();
}

/** @return Closure(): int */
function throwableCodeCallable(Throwable $exception): Closure
{
    return $exception->getCode(...);
}

/** @return Closure(): int */
function exceptionCodeCallable(Exception $exception): Closure
{
    return $exception->getCode(...);
}

/** @return Closure(): (int|string) */
function pdoExceptionCodeCallable(PDOException $exception): Closure
{
    return $exception->getCode(...);
}

/** @return Closure(): int */
function pdoExceptionIntegerCodeCallable(PDOException $exception): Closure
{
    // @mago-expect analysis:invalid-return-statement
    return $exception->getCode(...);
}

function pdoExceptionHasCode(PDOException $exception): bool
{
    if ($exception->getCode() === 42) {
        return true;
    }

    return false;
}

final class InvalidPdoExceptionOverride extends PDOException
{
    // @mago-expect analysis:override-final-method
    public function getCode(): int|string
    {
        return 0;
    }
}

function narrowPdoException(Throwable $exception): LogicException
{
    if ($exception instanceof PDOException) {
        // @mago-expect analysis:possibly-invalid-argument
        return new LogicException($exception->getMessage(), $exception->getCode(), $exception);
    }

    return new LogicException($exception->getMessage(), $exception->getCode(), $exception);
}
