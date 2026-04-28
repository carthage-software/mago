<?php

declare(strict_types=1);

/**
 * @implements Iterator<int, string>
 */
final class GenStringIter implements Iterator
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

$iter = new GenStringIter(['a', 'b']);
foreach ($iter as $k => $v) {
    echo $k . '=' . $v;
}
