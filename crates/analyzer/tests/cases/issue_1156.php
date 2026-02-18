<?php

declare(strict_types=1);

/**
 * @template T
 * @psalm-inheritors None<T>|Some<T>
 */
abstract class Option
{
    /**
     * @return T
     */
    abstract public function unwrap(): mixed;
}

/**
 * @template T
 * @extends Option<T>
 */
final class None extends Option
{
    public function unwrap(): never
    {
        throw new Exception('this is none!');
    }
}

/**
 * @template T
 * @extends Option<T>
 */
final class Some extends Option
{
    public function __construct(
        /** @var T $value */
        public mixed $value,
    ) {}

    /** @return T */
    public function unwrap(): mixed
    {
        return $this->value;
    }
}

/** @param Some<'hello'> $some */
function use_some(Some $some): void
{
    use_some($some);
}

/** @param None<'hello'> $none */
function use_none(None $none): void
{
    use_none($none);
}

/**
 * @return Option<'hello'>
 */
function get_some_or_none(): Option
{
    if (rand(0, 1) == 0) {
        /** @var None<'hello'> */
        return new None();
    } else {
        return new Some('hello');
    }
}

$option = get_some_or_none();

if ($option instanceof Some) {
    use_some($option);
} else {
    use_none($option);
}

echo $option->unwrap();
