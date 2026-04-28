<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenStack
{
    /** @var list<T> */
    private array $data = [];

    /**
     * @param T $value
     *
     * @return GenStack<T>
     */
    public function push(mixed $value): GenStack
    {
        $this->data[] = $value;
        return $this;
    }

    /** @return T|null */
    public function pop(): mixed
    {
        return array_pop($this->data);
    }
}

/** @var GenStack<int> $s */
$s = new GenStack();
$s->push(1)->push(2)->push(3);
$top = $s->pop();
if (null !== $top) {
    echo $top + 1;
}
