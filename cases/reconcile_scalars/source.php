<?php

/**
 * Represents a value in the CEL runtime.
 */
abstract readonly class Value
{
    /**
     * Returns the native PHP value.
     */
    abstract public function getNativeValue(): mixed;

    /**
     * Returns the CEL type name of the value.
     *
     * @return non-empty-string
     */
    abstract public function getType(): string;

    /**
     * Creates a Value object from a native PHP value.
     *
     * @throws Exception if the type is unsupported.
     */
    public static function from(mixed $value): Value
    {
        if ($value instanceof Value) {
            return $value;
        }

        if (null === $value) {
            return new NullValue();
        }

        if (is_bool($value)) {
            return new BooleanValue($value);
        }

        if (is_float($value)) {
            return new FloatValue($value);
        }

        if (is_int($value)) {
            return new IntegerValue($value);
        }

        if (is_string($value)) {
            return new StringValue($value);
        }

        throw new Exception(sprintf('Unsupported PHP type "%s"', gettype($value)));
    }
}

final readonly class IntegerValue extends Value
{
    public function __construct(
        private int $value,
    ) {}

    public function getNativeValue(): int
    {
        return $this->value;
    }

    public function getType(): string
    {
        return 'int';
    }
}

final readonly class FloatValue extends Value
{
    public function __construct(
        private float $value,
    ) {}

    public function getNativeValue(): float
    {
        return $this->value;
    }

    public function getType(): string
    {
        return 'double';
    }
}

final readonly class StringValue extends Value
{
    public function __construct(
        private string $value,
    ) {}

    public function getNativeValue(): string
    {
        return $this->value;
    }

    public function getType(): string
    {
        return 'string';
    }
}

final readonly class BooleanValue extends Value
{
    public function __construct(
        private bool $value,
    ) {}

    public function getNativeValue(): bool
    {
        return $this->value;
    }

    public function getType(): string
    {
        return 'bool';
    }
}

final readonly class NullValue extends Value
{
    public function getNativeValue(): null
    {
        return null;
    }

    public function getType(): string
    {
        return 'null';
    }
}
