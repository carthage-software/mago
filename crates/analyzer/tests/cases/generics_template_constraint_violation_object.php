<?php

declare(strict_types=1);

interface GenSized2
{
    public function size(): int;
}

/**
 * @template T of GenSized2
 */
final class GenSizedHolder2
{
    /** @param T $item */
    public function __construct(public GenSized2 $item)
    {
    }
}

final class GenNotSized
{
    public int $x = 0;
}

/** @mago-expect analysis:template-constraint-violation,possibly-invalid-argument */
new GenSizedHolder2(new GenNotSized());
