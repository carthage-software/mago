<?php

declare(strict_types=1);

final class Pipeline
{
    /**
     * @template T = int
     *
     * @param T $seed
     *
     * @return T
     */
    public function run(mixed $seed): mixed
    {
        return $seed;
    }
}

$p = new Pipeline();
echo $p->run(7) + 1;
echo strlen($p->run('hi'));
