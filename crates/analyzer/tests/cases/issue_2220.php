<?php

declare(strict_types=1);

/**
 * @api
 * @consistent-constructor
 */
class Base
{
    public function __construct(string $raw)
    {
        echo $raw;
    }

    public static function fromRaw(string $raw): static
    {
        return new static($raw);
    }
}

/**
 * @template T of Base
 */
abstract class Consumer
{
    /**
     * @return class-string<T>
     */
    abstract protected function getPHPType(): string;

    /**
     * @return T|null
     */
    public function convert(?string $value): ?Base
    {
        if (is_string($value)) {
            $class = $this->getPHPType();

            return $class::fromRaw($value);
        }

        return null;
    }
}
