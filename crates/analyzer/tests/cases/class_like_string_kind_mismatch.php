<?php

declare(strict_types=1);

trait KindUuid {}

interface KindShape {}

final class KindCircle implements KindShape {}

enum KindStatus {}

/** @param trait-string $name */
function kind_for_trait(string $name): void
{
    echo $name;
}

/** @param interface-string $name */
function kind_for_interface(string $name): void
{
    echo $name;
}

/** @param enum-string $name */
function kind_for_enum(string $name): void
{
    echo $name;
}

/** @param enum-string $enum */
function kind_strings(string $enum): void
{
    kind_for_trait(KindUuid::class);
    kind_for_interface(KindShape::class);
    kind_for_enum(KindStatus::class);
    kind_for_enum($enum);

    // @mago-expect analysis:invalid-argument
    kind_for_trait(KindCircle::class);
    // @mago-expect analysis:invalid-argument
    kind_for_trait(KindShape::class);
    // @mago-expect analysis:invalid-argument
    kind_for_interface(KindCircle::class);
    // @mago-expect analysis:invalid-argument
    kind_for_interface(KindStatus::class);
    // @mago-expect analysis:invalid-argument
    kind_for_enum(KindShape::class);
    // @mago-expect analysis:invalid-argument
    kind_for_interface($enum);
}
