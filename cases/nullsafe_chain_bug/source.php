<?php

declare(strict_types=1);

class X
{
    public function nullable(): ?X
    {
        return null;
    }

    public function nonNullable(): X
    {
        return $this;
    }

    public X $prop;

    public function __construct()
    {
        $this->prop = $this;
    }

    /** @var list<int> */
    public array $items = [];
}

// Direct nullable access - ERROR
function test_direct(?X $a): void
{
    $a->nullable();
}

// Nullsafe with nullable return - ERROR
function test_nullsafe_nullable_return(?X $a): void
{
    $a?->nullable()->nullable();
}

// Nullsafe with non-nullable return - NO ERROR (short-circuits)
function test_nullsafe_non_nullable_return(?X $a): void
{
    $a?->nonNullable()->nonNullable(); // OK - entire chain short-circuits when $a is null
}

// Nullsafe with non-nullable property - NO ERROR (short-circuits)
function test_nullsafe_non_nullable_prop(?X $a): void
{
    $a?->prop->nonNullable(); // OK - entire chain short-circuits when $a is null
}

// Array access - NO ERROR (already handled)
function test_nullsafe_array(?X $a): void
{
    $a?->items[0]; // OK - array access protected
}

// Direct call on nullable without nullsafe - ERROR
function test_nullable_without_nullsafe(?X $a, ?X $b): void
{
    $a->nullable();
    $b->nullable();
}

// Nullsafe does not affect subsequent statements
function test_nullsafe_does_not_affect_subsequent(?X $a): void
{
    $a?->nullable();
    $a->nullable();
}

// Nullsafe does not affect different variables
function test_nullsafe_does_not_affect_different_var(?X $a, ?X $b): void
{
    $a?->nullable();
    $b->nullable();
}
