<?php

#[Attribute(Attribute::TARGET_PARAMETER)]
final class DefaultFlagsAttribute
{
    /** @param int-mask<1, 2, 4> $flags */
    public function __construct(int $flags = 0) {}
}

function mapped_default(#[DefaultFlagsAttribute] int $page = 1): void {}

new DefaultFlagsAttribute();
new DefaultFlagsAttribute(flags: 3);
// @mago-expect analysis:invalid-argument
new DefaultFlagsAttribute(flags: 8);

final class LargeFlags
{
    public const READ = 1;
    public const WRITE = 2;
    public const EXECUTE = 4;
    public const DELETE = 8;
    public const CREATE = 16;
    public const UPDATE = 32;
    public const ARCHIVE = 64;
    public const RESTORE = 128;
    public const PUBLISH = 256;
    public const UNPUBLISH = 512;
    public const MODERATE = 1024;
    public const APPROVE = 2048;
    public const REJECT = 4096;
    public const IMPORT = 8192;
    public const EXPORT = 16384;
    public const SHARE = 32768;
    public const REVOKE = 65536;
    public const AUDIT = 131072;
    public const CONFIGURE = 262144;
    public const ADMINISTER = 524288;
}

#[Attribute(Attribute::TARGET_PARAMETER)]
final class LargeWildcardFlagsAttribute
{
    /** @param int-mask-of<LargeFlags::*> $flags */
    public function __construct(int $flags = 0) {}
}

function mapped_large_wildcard_default(#[LargeWildcardFlagsAttribute] int $page = 1): void {}

new LargeWildcardFlagsAttribute();

final class WildcardFlags
{
    public const READ = 1;
    public const WRITE = 2;
    public const EXECUTE = 4;
}

#[Attribute(Attribute::TARGET_PARAMETER)]
final class WildcardFlagsAttribute
{
    /** @param int-mask-of<WildcardFlags::*> $flags */
    public function __construct(int $flags = 0) {}
}

new WildcardFlagsAttribute(flags: 3);
// @mago-expect analysis:invalid-argument
new WildcardFlagsAttribute(flags: 8);

const DEFAULT_FLAGS = 1;

/**
 * @param int-mask<1, 2, 4> $flags
 * @return $flags
 */
function default_flags(int $flags = DEFAULT_FLAGS): int
{
    return $flags;
}

/** @param 1 $flags */
function requires_one(int $flags): void {}

requires_one(default_flags());
requires_one(default_flags(flags: 2)); // @mago-expect analysis:invalid-argument

/**
 * @template T
 * @param T $value
 * @return T
 */
function generic_default(mixed $value = 'default'): mixed
{
    return $value;
}

/** @template T */
final class DefaultBox
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value = 'default')
    {
        $this->value = $value;
    }

    /** @return T */
    public function value(): mixed
    {
        return $this->value;
    }
}

function requires_string(string $value): void {}

requires_string(generic_default());
requires_string((new DefaultBox())->value());

$callable = generic_default(...);
requires_string($callable());
