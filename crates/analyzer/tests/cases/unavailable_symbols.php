<?php

#[Mago\AvailableSince(80600)]
class FutureClass
{
    #[Mago\AvailableSince(80600)]
    public const string NEW_CONST = 'x';

    #[Mago\AvailableSince(80600)]
    public string $shinyProp = 'x';

    #[Mago\AvailableSince(80600)]
    public function shinyMethod(): void {}
}

#[Mago\AvailableSince(80600)]
function shiny_fn(): void {}

#[Mago\AvailableSince(80600)]
const FUTURE_CONST = 1;

#[Mago\AvailableSince(80600)]
enum FutureEnum
{
    #[Mago\AvailableSince(80600)]
    case Alpha;
}

#[Mago\AvailableUntil(80000)]
class LegacyClass
{
    #[Mago\AvailableUntil(80000)]
    public const string OLD_CONST = 'x';

    #[Mago\AvailableUntil(80000)]
    public string $oldProp = 'x';

    #[Mago\AvailableUntil(80000)]
    public function oldMethod(): void {}
}

#[Mago\AvailableUntil(80000)]
function legacy_fn(): void {}

#[Mago\AvailableUntil(80000)]
const LEGACY_CONST = 1;

#[Mago\AvailableUntil(80000)]
enum LegacyEnum
{
    #[Mago\AvailableUntil(80000)]
    case Old;
}

/**
 * @mago-expect analysis:unavailable-class-like
 * @mago-expect analysis:unavailable-function
 * @mago-expect analysis:unavailable-method
 * @mago-expect analysis:unavailable-property
 * @mago-expect analysis:unavailable-class-constant
 * @mago-expect analysis:unavailable-constant
 * @mago-expect analysis:unavailable-enum-case
 */
function uses_future(): void
{
    $obj = new FutureClass();
    $obj->shinyMethod();
    $_ = $obj->shinyProp;
    $_ = FutureClass::NEW_CONST;
    $_ = FUTURE_CONST;
    $_ = FutureEnum::Alpha;
    shiny_fn();
}

/**
 * @mago-expect analysis:unavailable-class-like
 * @mago-expect analysis:unavailable-function
 * @mago-expect analysis:unavailable-method
 * @mago-expect analysis:unavailable-property
 * @mago-expect analysis:unavailable-class-constant
 * @mago-expect analysis:unavailable-constant
 * @mago-expect analysis:unavailable-enum-case
 */
function uses_legacy(): void
{
    $obj = new LegacyClass();
    $obj->oldMethod();
    $_ = $obj->oldProp;
    $_ = LegacyClass::OLD_CONST;
    $_ = LEGACY_CONST;
    $_ = LegacyEnum::Old;
    legacy_fn();
}

// Disjoint ranges: present in 8.1-8.3, removed in 8.4, brought back in 8.6.
// Configured PHP 8.5 should still flag this as unavailable.
#[Mago\AvailableSince(80100)]
#[Mago\AvailableUntil(80300)]
#[Mago\AvailableSince(80600)]
function gap_fn(): void {}

/**
 * @mago-expect analysis:unavailable-function
 */
function uses_gap(): void
{
    gap_fn();
}
