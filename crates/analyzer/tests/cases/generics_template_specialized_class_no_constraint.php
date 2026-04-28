<?php

declare(strict_types=1);

/**
 * @template T
 */
class GenAnyOf
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }
}

/**
 * @extends GenAnyOf<bool>
 */
final class GenBoolOf extends GenAnyOf
{
    public function flip(): bool
    {
        return !$this->value;
    }
}

$b = new GenBoolOf(true);
echo $b->flip();
