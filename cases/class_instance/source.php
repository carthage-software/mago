<?php

/**
 * @template T
 */
interface Box
{
    /**
     * @return T
     */
    public function getValue(): mixed;
}

/**
 * @template T
 * @implements Box<T>
 */
class BoxImpl implements Box
{
    /**
     * @var T
     */
    private mixed $value;

    /**
     * @param T $value
     */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }

    /**
     * @return T
     */
    #[Override]
    public function getValue(): mixed
    {
        return $this->value;
    }
}

/**
 * @template T
 * @param class-string<T> $class
 *
 * @psalm-assert T $object
 * @throws InvalidArgumentException
 */
function assertInstanceOf(string $class, object $object): void
{
    if (!$object instanceof $class) {
        throw new InvalidArgumentException("Object is not an instance of $class");
    }
}

/**
 * @template T of object
 * @param class-string<T> $class
 * @return T
 * @throws InvalidArgumentException
 */
function getAs(string $class, object $object): object
{
    assertInstanceOf($class, $object);

    return $object;
}

function getBox(): object
{
    return new BoxImpl(42);
}

function useBox(Box $box): void
{
    var_dump($box);
}

$box = getAs(BoxImpl::class, getBox());
useBox($box);
