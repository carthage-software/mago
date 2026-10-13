<?php

declare(strict_types=1);

use Deprecated as Old;

final class Settings
{
    /** @deprecated Use NEW_LIMIT instead. */
    public const OLD_LIMIT = 10;

    #[\Deprecated('Use NEW_LIMIT instead.')]
    public const OTHER_LIMIT = 20;

    public const NEW_LIMIT = 30;

    /** @deprecated Use $name instead. */
    public string $label = '';

    /** @deprecated */
    public static int $count = 0;

    public string $name = '';

    /** @deprecated */
    public function old(): void {}
}

function use_settings(Settings $settings): int
{
    // @mago-expect analysis:deprecated-method
    $settings->old();
    // @mago-expect analysis:deprecated-property
    $settings->label = 'x';
    // @mago-expect analysis:deprecated-property
    echo $settings->label;
    // @mago-expect analysis:deprecated-property
    Settings::$count++;

    // @mago-expect analysis:deprecated-class-constant(2)
    return Settings::OLD_LIMIT + Settings::OTHER_LIMIT + Settings::NEW_LIMIT;
}

function repeated_reads(Settings $settings): void
{
    // @mago-expect analysis:deprecated-property
    echo $settings->label;
    // @mago-expect analysis:deprecated-property
    echo $settings->label;
    // @mago-expect analysis:deprecated-property
    echo Settings::$count;
    // @mago-expect analysis:deprecated-property
    echo Settings::$count;
    // @mago-expect analysis:deprecated-property
    Settings::$count = 1;
    // @mago-expect analysis:deprecated-property
    echo Settings::$count;
    // @mago-expect analysis:deprecated-property
    $settings->label .= 'x';
    // @mago-expect analysis:deprecated-property
    echo $settings->label;

    $settings->name = 'new';
    echo $settings->name;
    echo $settings->name;
}

function nullable_read(?Settings $settings): ?string
{
    // @mago-expect analysis:deprecated-property
    return $settings?->label;
}

function invalid_assignment(Settings $settings): void
{
    // @mago-expect analysis:deprecated-property,invalid-property-assignment-value
    $settings->label = 1;
}

function class_string_reads(): void
{
    $class = Settings::class;
    // @mago-expect analysis:deprecated-property
    echo $class::$count;
    // @mago-expect analysis:deprecated-property
    echo $class::$count;
}

function dynamic_names(Settings $settings): void
{
    $property = 'label';
    // @mago-expect analysis:deprecated-property
    echo $settings->{$property};
    // @mago-expect analysis:deprecated-property
    echo $settings->{$property};

    $staticProperty = 'count';
    // @mago-expect analysis:deprecated-property
    echo Settings::${$staticProperty};
    // @mago-expect analysis:deprecated-property
    echo Settings::${$staticProperty};

    $constant = 'OLD_LIMIT';
    // @mago-expect analysis:deprecated-class-constant
    echo Settings::{$constant};
}

class LegacySettings
{
    /** @deprecated */
    public string $legacyName = '';

    /** @deprecated */
    public static string $legacyState = '';

    /** @deprecated */
    public const LEGACY_TOKEN = 1;

    #[Old]
    public const FIRST = 2, SECOND = 3;
}

final class InheritedSettings extends LegacySettings
{
    public function read(): string
    {
        // @mago-expect analysis:deprecated-property
        echo $this->legacyName;
        // @mago-expect analysis:deprecated-property
        echo $this->legacyName;
        // @mago-expect analysis:deprecated-property
        echo self::$legacyState;
        // @mago-expect analysis:deprecated-property
        echo static::$legacyState;
        // @mago-expect analysis:deprecated-property
        echo parent::$legacyState;

        // @mago-expect analysis:deprecated-class-constant(5)
        echo self::LEGACY_TOKEN, static::LEGACY_TOKEN, parent::LEGACY_TOKEN, self::FIRST, self::SECOND;

        // @mago-expect analysis:deprecated-property
        return $this->legacyName;
    }
}

function inherited_members(InheritedSettings $settings): void
{
    // @mago-expect analysis:deprecated-property
    $settings->legacyName = 'x';
    // @mago-expect analysis:deprecated-property
    echo $settings->legacyName;
    // @mago-expect analysis:deprecated-property
    InheritedSettings::$legacyState = 'x';
    // @mago-expect analysis:deprecated-property
    echo InheritedSettings::$legacyState;
    // @mago-expect analysis:deprecated-class-constant(3)
    echo InheritedSettings::LEGACY_TOKEN, InheritedSettings::FIRST, InheritedSettings::SECOND;
}

final class PromotedSettings
{
    public function __construct(
        /** @deprecated */
        public string $legacyName = '',
    ) {}
}

function promoted_member(PromotedSettings $settings): string
{
    // @mago-expect analysis:deprecated-property
    $settings->legacyName = 'x';
    // @mago-expect analysis:deprecated-property
    return $settings->legacyName;
}

final class HookedSettings
{
    /** @deprecated */
    public string $legacyName {
        get => 'legacy';
        set {}
    }
}

function hooked_member(HookedSettings $settings): string
{
    // @mago-expect analysis:deprecated-property
    $settings->legacyName = 'x';
    // @mago-expect analysis:deprecated-property
    return $settings->legacyName;
}

interface LegacyInterface
{
    /** @deprecated */
    public const OLD_TOKEN = 1;
}

trait LegacyTrait
{
    /** @deprecated */
    public string $legacyValue = '';

    #[Old]
    public const OLD_TOKEN = 1;
}

final class TraitSettings
{
    use LegacyTrait;
}

function trait_and_interface_members(TraitSettings $settings): void
{
    // @mago-expect analysis:deprecated-property
    echo $settings->legacyValue;
    // @mago-expect analysis:deprecated-class-constant
    echo TraitSettings::OLD_TOKEN;
    // @mago-expect analysis:deprecated-class-constant
    echo LegacyInterface::OLD_TOKEN;
}

enum LegacyEnum
{
    #[Old]
    public const OLD_TOKEN = 1;
}

function enum_constant(): int
{
    // @mago-expect analysis:deprecated-class-constant
    return LegacyEnum::OLD_TOKEN;
}

/** @property string $label */
final class MagicSettings
{
    /** @deprecated */
    private string $label = '';

    public function __get(string $_): string
    {
        return 'replacement';
    }

    public function internal_read(): string
    {
        // @mago-expect analysis:deprecated-property
        return $this->label;
    }
}

function magic_member(MagicSettings $settings): string
{
    echo $settings->label;

    return $settings->label;
}
