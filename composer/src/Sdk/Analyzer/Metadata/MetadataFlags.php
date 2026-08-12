<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/** Immutable semantic flags attached to scanned metadata. @api */
final class MetadataFlags
{
    public const ABSTRACT = 1;
    public const FINAL = 1 << 1;
    public const READONLY = 1 << 3;
    public const DEPRECATED = 1 << 4;
    public const INTERNAL = 1 << 7;
    public const USER_DEFINED = 1 << 14;
    public const BUILTIN = 1 << 15;
    public const MUST_USE = 1 << 17;
    public const PURE = 1 << 19;
    public const BY_REFERENCE = 1 << 26;
    public const VARIADIC = 1 << 27;
    public const PROMOTED_PROPERTY = 1 << 28;
    public const HAS_DEFAULT = 1 << 29;
    public const VIRTUAL_PROPERTY = 1 << 30;
    public const ASYMMETRIC_PROPERTY = 1 << 31;
    public const STATIC = 1 << 32;
    public const WRITEONLY = 1 << 33;
    public const MAGIC_METHOD = 1 << 35;
    public const API = 1 << 36;
    public const MUTATION_FREE = 1 << 37;
    public const EXTERNAL_MUTATION_FREE = 1 << 38;
    public const SUSPENDS_FIBER = 1 << 39;
    public const EXPERIMENTAL = 1 << 40;
    public const POLYFILL = 1 << 41;
    public const PATCH = 1 << 42;

    public function __construct(
        public readonly int $bits,
    ) {}

    public function contains(int $flag): bool
    {
        return ($this->bits & $flag) === $flag;
    }
}
