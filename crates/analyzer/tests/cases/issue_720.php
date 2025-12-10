<?php

declare(strict_types=1);

/**
 * @template T
 */
class Transformer
{
    /**
     * @var T $value
     */
    private mixed $value;

    /**
     * @param T $value
     */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }

    public function convertString(): string|null
    {
        if ($this->value === null || $this->value === '') {
            return null;
        }

        return 'some string';
    }

    public function convertInt(): int|null
    {
        if ($this->value === null || $this->value === 0) {
            return null;
        }

        return 42;
    }

    public function convertFloat(): float|null
    {
        if ($this->value === null || $this->value === 0.0) {
            return null;
        }

        return 3.14;
    }

    public function convertTrue(): bool|null
    {
        if ($this->value === null || $this->value === true) {
            return null;
        }

        return false;
    }

    public function convertFalse(): bool|null
    {
        if ($this->value === null || $this->value === false) {
            return null;
        }

        return true;
    }
}
