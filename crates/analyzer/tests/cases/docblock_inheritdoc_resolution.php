<?php

declare(strict_types=1);

class ParentBK
{
    /**
     * @return non-empty-string
     */
    public function name(): string
    {
        return 'parent';
    }
}

class ChildBK extends ParentBK
{
    /**
     * @inheritDoc
     */
    public function name(): string
    {
        return 'child';
    }
}

/** @param non-empty-string $s */
function takeNeBK(string $s): string
{
    return $s;
}

$c = new ChildBK();
takeNeBK($c->name());
