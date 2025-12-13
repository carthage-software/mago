<?php

declare(strict_types=1);

/**
 * @template-covariant T
 */
interface TypeInterface
{
    /**
     * @return T
     *
     * @psalm-assert T $value
     */
    public function assert(mixed $value): mixed;

    /**
     * @psalm-assert-if-true T $value
     */
    public function isValid(mixed $value): bool;
}

/**
 * @implements TypeInterface<string>
 */
final class StringType implements TypeInterface
{
    #[Override]
    public function assert(mixed $value): mixed
    {
        if (!is_string($value)) {
            die('invalid');
        }

        return $value;
    }

    #[Override]
    public function isValid(mixed $value): bool
    {
        return is_string($value);
    }
}

/**
 * @return TypeInterface<mixed>
 */
function detect_type(mixed $value): TypeInterface
{
    if (is_string($value)) {
        $stringType = new StringType();

        if (!$stringType->isValid($value)) { // @mago-expect analysis:redundant-type-comparison
            return $stringType;
        }

        if ($stringType->isValid($value)) { // @mago-expect analysis:redundant-type-comparison
            return $stringType;
        }

        return new StringType();
    }

    die('unknown type');
}
