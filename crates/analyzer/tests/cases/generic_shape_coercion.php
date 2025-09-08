<?php

/**
 * @template T
 */
interface TypeInterface
{
    /** @assert-if-true T $value */
    public function matches(mixed $value): bool;

    /**
     * @return T
     */
    public function coerce(mixed $value): mixed;

    public function isOptional(): bool;
}

/** @return TypeInterface<array-key> */
function array_key(): TypeInterface
{
    return array_key();
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @mago-expect analysis:mixed-assignment
 */
final class ShapeType
{
    /**
     * @param array<Tk, TypeInterface<Tv>> $elements_types
     */
    public function __construct(
        private array $elements_types,
        private bool $allow_unknown_fields = false,
    ) {}

    /**
     * @throws Throwable
     *
     * @return array<Tk, Tv>
     */
    public function coerceIterable(mixed $value): array
    {
        if (!is_iterable($value)) {
            throw new Exception();
        }

        $arrayKeyType = array_key();
        $array = [];
        try {
            foreach ($value as $k => $v) {
                if ($arrayKeyType->matches($k)) {
                    $array[$k] = $v;
                }
            }
        } catch (Throwable $e) {
            throw $e;
        }

        $result = [];

        try {
            foreach ($this->elements_types as $element => $type) {
                if (isset($array[$element])) {
                    $result[$element] = $type->coerce($array[$element]);

                    continue;
                }

                if ($type->isOptional()) {
                    continue;
                }

                throw new Exception();
            }
        } catch (Exception $e) {
            throw $e;
        }

        if ($this->allow_unknown_fields) {
            foreach ($array as $k => $v) {
                if (!isset($result[$k])) {
                    $result[$k] = $v;
                }
            }
        }

        /** @var array<Tk, Tv> */
        return $result;
    }
}
