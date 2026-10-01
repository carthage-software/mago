<?php

declare(strict_types=1);

namespace Issue2379;

/** @template TChild of Child */
interface Owner
{
    /** @return list<TChild> */
    public function getChildren(): array;
}

/** @template TOwner of Owner */
interface Child
{
    /** @return TOwner */
    public function getOwner(): Owner;
}

final class Probe
{
    /**
     * @template T of Child
     *
     * @param T $original
     *
     * @return T
     */
    public function flat(Owner $owner, Child $original): Child
    {
        $class = $original::class;

        /** @mago-expect analysis:too-many-arguments,unsafe-instantiation */
        return new $class($owner);
    }

    /**
     * @template TOwner of Owner
     * @template T of Child<TOwner>
     *
     * @param TOwner $owner
     * @param T $original
     *
     * @return T
     */
    public function nested(Owner $owner, Child $original): Child
    {
        $class = $original::class;

        /** @mago-expect analysis:too-many-arguments,unsafe-instantiation */
        return new $class($owner);
    }
}
