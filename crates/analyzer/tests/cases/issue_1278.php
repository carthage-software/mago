<?php

declare(strict_types=1);

class SomeOtherClass
{
    public string $varWithValue = '456';
}

class TestClass
{
    private ?SomeOtherClass $test = null;

    public function __construct()
    {
        $this->test = new SomeOtherClass();
    }

    public function test(): void
    {
        // In `A || B`, when B is evaluated, A must be false (short-circuit).
        // `($this->test ?? null) === null` being false means `$this->test` is not null.
        // So `$this->test->varWithValue` in both the second operand and the else branch should be safe.
        if (($this->test ?? null) === null || $this->test->varWithValue !== '123') {
            var_dump('test');
        } else {
            var_dump($this->test->varWithValue);
        }
    }

    public function testNullCoalesceNotNull(): void
    {
        // `($this->test ?? null) !== null` directly narrows $this->test to non-null.
        if (($this->test ?? null) !== null) {
            var_dump($this->test->varWithValue);
        }
    }

    public function testNullCoalesceIsNull(): void
    {
        // `($this->test ?? null) === null` means $this->test is null.
        // In the else branch, $this->test is narrowed to non-null.
        if (($this->test ?? null) === null) {
            var_dump('is null');
        } else {
            var_dump($this->test->varWithValue);
        }
    }

    public function testAndShortCircuit(): void
    {
        // In `A && B`, when B is evaluated, A must be true.
        // `($this->test ?? null) !== null` being true means `$this->test` is not null.
        if (($this->test ?? null) !== null && $this->test->varWithValue === '123') {
            var_dump($this->test->varWithValue);
        }
    }
}
