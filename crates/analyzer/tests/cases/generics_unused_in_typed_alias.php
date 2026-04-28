<?php

declare(strict_types=1);

/**
 * @template T
 * @template U
 *
 * @mago-expect analysis:unused-template-parameter
 */
final class GenAliasUnused
{
    /** @var T */
    public mixed $val;

    public function __construct(mixed $v)
    {
        $this->val = $v;
    }
}
