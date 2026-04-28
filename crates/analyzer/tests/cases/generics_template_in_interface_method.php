<?php

declare(strict_types=1);

interface GenRepo
{
    /**
     * @template T of object
     *
     * @param class-string<T> $cls
     *
     * @return T|null
     */
    public function find(string $cls): ?object;
}

final class GenSomeRow
{
    public int $id = 0;
}

final class GenRepoImpl implements GenRepo
{
    public function find(string $cls): ?object
    {
        return null;
    }
}

function lookup(GenRepo $repo): ?GenSomeRow
{
    return $repo->find(GenSomeRow::class);
}

$r = lookup(new GenRepoImpl());
if (null !== $r) {
    echo $r->id;
}
