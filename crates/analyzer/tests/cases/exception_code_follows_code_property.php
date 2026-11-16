<?php

declare(strict_types=1);

class ServiceException extends LogicException
{
}

final class ServiceUnavailable extends ServiceException
{
}

/** @mago-expect analysis:incompatible-property-type */
final class StatusException extends LogicException
{
    /** @var int|string */
    protected $code;
}

/** @throws ServiceUnavailable */
function rethrow(ServiceUnavailable $exception): never
{
    throw new ServiceUnavailable('Service unavailable.', $exception->getCode(), $exception);
}

function logic_code(LogicException $exception): int|string
{
    return $exception->getCode();
}

/** @throws ServiceUnavailable */
function rethrow_logic(LogicException $exception): never
{
    // @mago-expect analysis:possibly-invalid-argument
    throw new ServiceUnavailable('Logic failure.', $exception->getCode(), $exception);
}

function service_code(ServiceException $exception): int
{
    return $exception->getCode();
}

/** @throws ServiceUnavailable */
function runtime_rethrow(RuntimeException $exception): never
{
    // @mago-expect analysis:possibly-invalid-argument
    throw new ServiceUnavailable('Runtime failure.', $exception->getCode(), $exception);
}

function database_code(PDOException $exception): int|string
{
    return $exception->getCode();
}
