<?php

declare(strict_types=1);

namespace Issue2351;

interface Something
{
    /**
     * @template T
     *
     * @param T $arg
     *
     * @return T
     */
    public function mirror(mixed $arg): mixed;
}

final class NoDocs implements Something
{
    public function mirror(mixed $arg): mixed
    {
        return $arg;
    }
}

/** @return 'Fred' */
function mirrorWithoutDocs(NoDocs $mirror): string
{
    return $mirror->mirror('Fred');
}

final class InheritDocs implements Something
{
    /** @inheritDoc */
    public function mirror(mixed $arg): mixed
    {
        return $arg;
    }
}

/** @return 'Fred' */
function mirrorWithInheritedDocs(InheritDocs $mirror): string
{
    return $mirror->mirror('Fred');
}

final class RepeatedDocs implements Something
{
    /**
     * @template T
     *
     * @param T $arg
     *
     * @return T
     */
    public function mirror(mixed $arg): mixed
    {
        return $arg;
    }
}

/** @return 'Fred' */
function mirrorWithRepeatedDocs(RepeatedDocs $mirror): string
{
    return $mirror->mirror('Fred');
}
