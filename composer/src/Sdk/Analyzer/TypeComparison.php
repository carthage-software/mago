<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use function pack;

/**
 * One native type comparison, suitable for batched evaluation.
 *
 * @api
 */
final class TypeComparison
{
    private ?string $leftEncoding = null;

    private ?string $rightEncoding = null;

    private ?string $cacheKey = null;

    public function __construct(
        public readonly TypeComparisonKind $kind,
        public readonly Type $left,
        public readonly Type $right,
    ) {}

    public static function equal(Type $left, Type $right): self
    {
        return new self(TypeComparisonKind::Equal, $left, $right);
    }

    public static function containedBy(Type $input, Type $container): self
    {
        return new self(TypeComparisonKind::ContainedBy, $input, $container);
    }

    public static function canBeIdentical(Type $left, Type $right): self
    {
        return new self(TypeComparisonKind::CanBeIdentical, $left, $right);
    }

    /** @internal */
    public function encodeLeft(): string
    {
        return $this->leftEncoding ??= $this->left->encode();
    }

    /** @internal */
    public function encodeRight(): string
    {
        return $this->rightEncoding ??= $this->right->encode();
    }

    /** @internal */
    public function cacheKey(): string
    {
        return $this->cacheKey ??= pack('C', $this->kind->value) . $this->encodeLeft() . $this->encodeRight();
    }
}
