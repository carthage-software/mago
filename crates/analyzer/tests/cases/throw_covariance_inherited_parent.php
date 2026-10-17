<?php

declare(strict_types=1);

class ParentFailure extends \Exception {}

class BaseFailure extends ParentFailure
{
    /**
     * @throws parent
     * @mago-expect analysis:incompatible-throw-type
     */
    public function run(): void
    {
        throw new ParentFailure();
    }
}

interface Runner
{
    /** @throws BaseFailure */
    public function run(): void;
}

class ChildFailure extends BaseFailure implements Runner {}

interface BroadRunner
{
    /** @throws ParentFailure */
    public function run(): void;
}

class CompatibleFailure extends BaseFailure implements BroadRunner {}

class DirectFailure extends BaseFailure implements Runner
{
    /** @throws parent */
    public function run(): void
    {
        throw new BaseFailure();
    }
}

trait ParentFailureRunner
{
    /**
     * @throws parent
     * @mago-expect analysis:incompatible-throw-type
     */
    public function run(): void {}
}

class TraitBaseFailure extends ParentFailure
{
    use ParentFailureRunner;
}

interface TraitRunner
{
    /** @throws TraitBaseFailure */
    public function run(): void;
}

class TraitChildFailure extends TraitBaseFailure implements TraitRunner {}

class TraitOverrideFailure extends TraitBaseFailure
{
    /** @throws ParentFailure */
    public function run(): void
    {
        throw new ParentFailure();
    }
}

trait StaticFailureRunner
{
    /** @throws static */
    public function run(): void {}
}

class StaticBaseFailure extends ParentFailure
{
    use StaticFailureRunner;
}

class StaticChildFailure extends StaticBaseFailure
{
    /** @throws static */
    public function run(): void {}
}
