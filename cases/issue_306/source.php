<?php

declare(strict_types=1);

class Sub
{
    public int $a = 0;
    public string $b = '';
}

class Worker
{
    /** @var list<int> */
    private array $list = [];

    /**
     * @param list<int> $subs
     */
    function test(array $subs): void
    {
        $this->list += $subs;
    }
}
