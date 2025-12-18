<?php

declare(strict_types=1);

class Entity
{
    public function __construct(
        public int $id,
    ) {}
}

/**
 * @template TEntity of object
 */
abstract class AbstractVoter
{
    /** @param TEntity $object */
    abstract protected function voteOnEntity(object $object): bool;
}

/**
 * @extends AbstractVoter<Entity>
 */
abstract class AbstractEntityVoter extends AbstractVoter
{
}

final class Voter1 extends AbstractEntityVoter
{
    #[Override]
    protected function voteOnEntity(object $object): bool
    {
        return $object->id === 1;
    }
}

final class Voter1Inherit extends AbstractEntityVoter
{
    /**
     * @inheritDoc
     */
    #[Override]
    protected function voteOnEntity(object $object): bool
    {
        return $object->id === 1;
    }
}

/**
 * @extends AbstractVoter<Entity>
 */
final class Voter2 extends AbstractVoter
{
    #[Override]
    protected function voteOnEntity(object $object): bool
    {
        return $object->id === 2;
    }
}

/**
 * @extends AbstractVoter<Entity>
 */
final class Voter2Inherit extends AbstractVoter
{
    /**
     * @inheritDoc
     */
    #[Override]
    protected function voteOnEntity(object $object): bool
    {
        return $object->id === 2;
    }
}
