<?php

declare(strict_types=1);

interface Column
{
    public function name(): string;
}

final class FlatColumn implements Column
{
    public function __construct(
        private string $name,
    ) {}

    public function name(): string
    {
        return $this->name;
    }
}

final class NestedColumn implements Column
{
    public function __construct(
        private string $name,
    ) {}

    public function name(): string
    {
        return $this->name;
    }

    public function isList(): bool
    {
        return true;
    }
}

function processNestedColumn(NestedColumn $column): void
{
    echo 'Processing nested column: ' . $column->name() . "\n";
}

function process(Column $column): void
{
    if ($column instanceof FlatColumn) {
        echo "Flat column\n";
        return;
    }

    /**
     * @var NestedColumn $column
     */
    // Comment after docblock should not prevent docblock from being recognized
    // 
    // 
    // Some comment here
    // 
    /* another */
    # and this
    processNestedColumn($column);
}
