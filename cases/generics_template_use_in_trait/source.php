<?php

declare(strict_types=1);

/**
 * @template T
 */
trait GenStateTrait
{
    /** @var list<T> */
    private array $state = [];

    /** @param T $item */
    public function record(mixed $item): void
    {
        $this->state[] = $item;
    }

    /** @return list<T> */
    public function snapshot(): array
    {
        return $this->state;
    }
}

final class GenIntState
{
    /**
     * @use GenStateTrait<int>
     */
    use GenStateTrait;
}

$s = new GenIntState();
$s->record(1);
$s->record(2);
foreach ($s->snapshot() as $n) {
    echo $n + 1;
}
