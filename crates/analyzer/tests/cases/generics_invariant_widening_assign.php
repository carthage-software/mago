<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenAssignInv
{
    /** @var T */
    public mixed $value;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->value = $v;
    }
}

/** @param GenAssignInv<int> $b */
function set_string_inv(GenAssignInv $b): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $b->value = 'string';
}
