<?php

declare(strict_types=1);

/**
 * @template T
 */
trait GenStateTrait2
{
    /** @var list<T> */
    private array $state = [];

    /** @param T $item */
    public function record(mixed $item): void
    {
        $this->state[] = $item;
    }
}

final class GenIntState2
{
    /**
     * @use GenStateTrait2<int>
     */
    use GenStateTrait2;
}

$s = new GenIntState2();
/** @mago-expect analysis:invalid-argument */
$s->record('not int');
