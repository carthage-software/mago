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
     * @phpstan-assert T $value
     */
    public function assert(mixed $value): mixed;

    /**
     * @phpstan-assert-if-true T $value
     */
    public function isValid(mixed $value): bool;
}

/**
 * @template TLeft
 * @template TRight
 *
 * @implements TypeInterface<TLeft|TRight>
 */
final class UnionType implements TypeInterface
{
    /**
     * @param TypeInterface<TLeft> $left
     * @param TypeInterface<TRight> $right
     */
    public function __construct(
        private TypeInterface $left,
        private TypeInterface $right,
    ) {}

    #[Override]
    public function assert(mixed $value): mixed
    {
        if ($this->left->isValid($value)) {
            return $value;
        }

        if ($this->right->isValid($value)) {
            return $value;
        }

        die('invalid');
    }

    #[Override]
    public function isValid(mixed $value): bool
    {
        return $this->left->isValid($value) || $this->right->isValid($value);
    }
}
