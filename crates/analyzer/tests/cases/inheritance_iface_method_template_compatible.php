<?php

declare(strict_types=1);

/**
 * @template T
 */
interface InhIfaceTpl
{
    /** @return T */
    public function get(): mixed;
}

/**
 * @implements InhIfaceTpl<int>
 */
class InhIfaceTplIntImpl implements InhIfaceTpl
{
    public function get(): int
    {
        return 1;
    }
}

(new InhIfaceTplIntImpl())->get();
