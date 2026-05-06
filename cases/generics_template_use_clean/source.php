<?php

declare(strict_types=1);

/**
 * @template T
 */
trait GenHolderTr
{
    /** @var T */
    public mixed $val;

    /** @return T */
    public function getVal(): mixed
    {
        return $this->val;
    }
}

/**
 * @template T
 */
final class GenWrap
{
    /**
     * @use GenHolderTr<T>
     */
    use GenHolderTr;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->val = $v;
    }
}

/**
 * @param GenWrap<int> $w
 */
function consume_wrap(GenWrap $w): int
{
    return $w->getVal();
}

consume_wrap(new GenWrap(1));
