<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class GenIntCol
{
    /** @var list<T> */
    private array $data = [];

    /** @param T $v */
    public function add(int $v): void
    {
        $this->data[] = $v;
    }

    /** @return list<T> */
    public function all(): array
    {
        return $this->data;
    }
}

/** @var GenIntCol<int> $c */
$c = new GenIntCol();
$c->add(1);
$c->add(2);
foreach ($c->all() as $n) {
    echo $n + 1;
}
