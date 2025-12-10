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

    public function convert(): string|null
    {
        if ($this->value === null || $this->value === '') {
            return null;
        }

        return 'some string';
    }
}
