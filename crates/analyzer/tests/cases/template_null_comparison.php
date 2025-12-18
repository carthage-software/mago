<?php

interface TestInterface
{
    public function test(): void;
}

/**
 * @template T of TestInterface|null
 */
class Test
{
    /**
     * @psalm-param T $value
     */
    public function __construct(
        public TestInterface|null $value,
    ) {}

    public function getValue(): null|TestInterface
    {
        return $this->value;
    }

    public function getValue2(): TestInterface
    {
        if ($this->value === null) {
            exit(1);
        }

        return $this->value;
    }

    public function getValue3(): TestInterface
    {
        return $this->value ?? exit(1);
    }
}

/**
 * @template T of string|null
 */
class ScalarTest
{
    /**
     * @param T $value
     */
    public function __construct(
        public string|null $value,
    ) {}

    public function getValue(): string
    {
        if ($this->value === null) {
            return 'default';
        }

        return $this->value;
    }
}
