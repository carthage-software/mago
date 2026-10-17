<?php

declare(strict_types=1);

class CovarianceBaseException extends \RuntimeException {}

class CovarianceChildException extends CovarianceBaseException {}

class CovarianceOtherException extends \RuntimeException {}

interface CovarianceLoader
{
    /** @throws \RuntimeException */
    public function load(): object;
}

final class CovarianceLoaderImplementation implements CovarianceLoader
{
    /** @throws CovarianceChildException */
    public function load(): object
    {
        return new \stdClass();
    }
}

interface CovarianceUnionLoader
{
    /** @throws CovarianceBaseException|CovarianceOtherException */
    public function load(): object;
}

final class CovarianceUnionLoaderImplementation implements CovarianceUnionLoader
{
    /** @throws CovarianceBaseException */
    public function load(): object
    {
        return new \stdClass();
    }
}

class CovarianceParent
{
    /** @throws \RuntimeException */
    public function execute(): void
    {
    }
}

final class CovarianceNoThrowChild extends CovarianceParent
{
    /** @throws void */
    public function execute(): void
    {
    }
}

interface CovarianceUnspecifiedContract
{
    public function execute(): void;
}

final class CovarianceSpecifiedImplementation implements CovarianceUnspecifiedContract
{
    /** @throws CovarianceChildException */
    public function execute(): void
    {
    }
}

class CovarianceSelfThrowsParent
{
    /** @throws self */
    public function load(): void
    {
    }
}

final class CovarianceSelfThrowsChild extends CovarianceSelfThrowsParent
{
    /** @throws CovarianceSelfThrowsParent */
    public function load(): void
    {
    }
}

class CovarianceInheritDocContract
{
    /** @throws CovarianceNotFound */
    public function load(): void
    {
    }
}

final class CovarianceInheritDocImplementation extends CovarianceInheritDocContract
{
    /** @inheritDoc */
    public function load(): void
    {
    }
}
