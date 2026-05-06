<?php

/**
 * @psalm-assert-if-true numeric-string $a
 */
function is_numeric_string(string $a): bool
{
    return is_numeric_string($a);
}

class SomeStringValue
{
    private string $value = '';

    public function __construct(string $value)
    {
        $this->value = $value;
    }

    public function isNonZeroNumericString(): bool
    {
        if (!is_numeric_string($this->value)) {
            return false;
        }

        if ($this->value === '0') {
            return false;
        }

        return true;
    }
}

/**
 * @param numeric-string $f
 */
function s(string $f): void
{
    echo "s: $f\n";
}

s('0');
