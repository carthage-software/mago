<?php declare(strict_types=1);

/**
 * @template T
 */
interface TypeInterface
{
    /**
     * @psalm-assert T $value
     * @return T
     */
    public function assert(mixed $value): mixed;
}

/**
 * @return TypeInterface<string>
 */
function str(): TypeInterface
{
    return str();
}

/**
 * @template T
 * @param TypeInterface<T> $t
 * @return TypeInterface<list<T>>
 */
function vec(TypeInterface $t): TypeInterface
{
    return vec($t);
}

/**
 * @param array<mixed> $arr
 * @return list<string>
 */
function as_str_vec(array $arr): array
{
    vec(str())->assert($arr);

    return $arr;
}
