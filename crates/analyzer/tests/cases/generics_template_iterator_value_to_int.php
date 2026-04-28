<?php

declare(strict_types=1);

/**
 * @implements Iterator<int, string>
 */
final class GenStringIter2 implements Iterator
{
    private int $pos = 0;
    /** @var list<string> */
    private array $data;

    /** @param list<string> $data */
    public function __construct(array $data)
    {
        $this->data = $data;
    }

    public function current(): string
    {
        return $this->data[$this->pos];
    }

    public function key(): int
    {
        return $this->pos;
    }

    public function next(): void
    {
        ++$this->pos;
    }

    public function rewind(): void
    {
        $this->pos = 0;
    }

    public function valid(): bool
    {
        return isset($this->data[$this->pos]);
    }
}

function takes_int_str_iter(int $n): void
{
}

$iter = new GenStringIter2(['a']);
foreach ($iter as $k => $v) {
    /** @mago-expect analysis:invalid-argument */
    takes_int_str_iter($v);
}
