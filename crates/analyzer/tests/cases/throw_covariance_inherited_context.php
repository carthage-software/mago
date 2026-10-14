<?php

declare(strict_types=1);

class BaseFailure extends \Exception
{
    /**
     * @throws self
     * @mago-expect analysis:incompatible-throw-type
     */
    public function run(): void
    {
        throw new self();
    }
}

interface Runner
{
    /** @throws ChildFailure */
    public function run(): void;
}

class ChildFailure extends BaseFailure implements Runner {}

interface BroadRunner
{
    /** @throws BaseFailure */
    public function run(): void;
}

final class CompatibleFailure extends BaseFailure implements BroadRunner {}

class StaticBaseFailure extends \Exception
{
    /** @throws static */
    public function run(): void {}
}

interface StaticRunner
{
    /** @throws StaticChildFailure */
    public function run(): void;
}

final class StaticChildFailure extends StaticBaseFailure implements StaticRunner {}

trait TraitFailureRunner
{
    /** @throws self */
    public function run(): void {}
}

interface TraitRunner
{
    /** @throws TraitFailure */
    public function run(): void;
}

final class TraitFailure extends \Exception implements TraitRunner
{
    use TraitFailureRunner;
}
