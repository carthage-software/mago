<?php

declare(strict_types=1);

final class MapperCE
{
    /**
     * @template T
     *
     * @param T $value
     *
     * @return T
     */
    public function id(mixed $value): mixed
    {
        return $value;
    }
}

$m = new MapperCE();
echo $m->id(7) + 1;
echo $m->id('hello');
