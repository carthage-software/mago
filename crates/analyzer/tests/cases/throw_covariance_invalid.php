<?php

declare(strict_types=1);

final class CovarianceNotFound extends \RuntimeException {}

interface CovarianceNotFoundContract
{
    /** @throws CovarianceNotFound */
    public function load(): object;
}

final class CovarianceMissingTagImplementation implements CovarianceNotFoundContract
{
    /** @mago-expect analysis:incompatible-throw-type */
    public function load(): object
    {
        throw new CovarianceNotFound();
    }
}

final class CovarianceWidenedImplementation implements CovarianceNotFoundContract
{
    /**
     * @throws \Exception
     * @mago-expect analysis:incompatible-throw-type
     */
    public function load(): object
    {
        return new \stdClass();
    }
}

class GenericCovarianceFailure extends \RuntimeException {}

/**
 * @template-covariant TFailure of \Throwable
 * @mago-expect analysis:unused-template-parameter
 */
interface GenericCovarianceHandler
{
    /** @throws TFailure */
    public function handle(): void;
}

/** @implements GenericCovarianceHandler<GenericCovarianceFailure> */
final class GenericCovarianceWidenedImplementation implements GenericCovarianceHandler
{
    /**
     * @throws \RuntimeException
     * @mago-expect analysis:incompatible-throw-type
     */
    public function handle(): void
    {
    }
}

/** @extends GenericCovarianceHandler<\RuntimeException> */
interface GenericCovarianceBroadPath extends GenericCovarianceHandler {}

/** @extends GenericCovarianceHandler<GenericCovarianceFailure> */
interface GenericCovarianceSpecificPath extends GenericCovarianceHandler {}

interface GenericCovarianceDiamond extends GenericCovarianceBroadPath, GenericCovarianceSpecificPath {}

final class GenericCovarianceDiamondImplementation implements GenericCovarianceDiamond
{
    /**
     * @throws \RuntimeException
     * @mago-expect analysis:incompatible-throw-type
     */
    public function handle(): void
    {
    }
}

class CovarianceThrowParent
{
    /** @throws CovarianceNotFound */
    public function save(): void
    {
    }
}

final class CovarianceThrowChild extends CovarianceThrowParent
{
    /**
     * @throws \Exception
     * @mago-expect analysis:incompatible-throw-type
     */
    public function save(): void
    {
    }
}
