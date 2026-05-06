<?php

/**
 * @template T
 */
trait Holder
{
    /**
     * @var null|T
     */
    public mixed $value = null;

    /**
     * @return null|T
     */
    public function getValue(): mixed
    {
        return $this->value;
    }

    /**
     * @param null|T $value
     */
    public function setValue(mixed $value): void
    {
        $this->value = $value;
    }
}

/**
 * @template T
 */
class Box
{
    /**
     * @use Holder<T>
     */
    use Holder;

    /**
     * @param null|T $value
     */
    public function __construct(mixed $value = null)
    {
        $this->setValue($value);
    }
}

/**
 * @template T
 * @param Box<T> $box
 *
 * @return T|null
 */
function extract_value(Box $box): mixed
{
    return $box->getValue();
}

/**
 * @template T
 * @param Box<T> $box
 * @param T|null $value
 */
function insert_value(Box $box, mixed $value): void
{
    $box->setValue($value);
}
