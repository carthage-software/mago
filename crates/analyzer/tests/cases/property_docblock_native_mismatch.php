<?php

declare(strict_types=1);

final class PropertyDocblockMismatch
{
    /**
     * @var array<int, string>
     */
    // @mago-expect analysis:docblock-type-mismatch,invalid-property-default-value
    public string $s = 'a';

    /** @var list<int> */
    public array $list = [];

    /** @var (\Closure(string): string[])|null */
    public ?\Closure $callback = null;
}
