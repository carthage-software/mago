<?php

namespace Fixture;

class MissingPropertyType
{
    /** @mago-expect analysis:missing-property-type */
    public $withoutDefault;

    /** @mago-expect analysis:missing-property-type */
    protected $withoutDefaultProtected;

    /** @mago-expect analysis:missing-property-type */
    public $withDefault = 1;

    /**
     * Reported once per declared item, defaulted or not.
     *
     * @mago-expect analysis:missing-property-type(2)
     */
    public $firstItem, $secondItem = 2;

    // Hooked properties never carry a default, so they only report via the
    // `Property::Hooked` arm.
    /** @mago-expect analysis:missing-property-type */
    public $hooked {
        get => 1;
    }

    // A docblock `@var` does not satisfy the check; a native hint is required,
    // matching how `@param` and `@return` are treated by
    // `missing-parameter-type` and `missing-return-type`.
    /**
     * @var int
     *
     * @mago-expect analysis:missing-property-type
     */
    public $docblockOnly;

    // `imprecise-type` on properties has the same per-item behavior
    /** @mago-expect analysis:imprecise-type */
    public array $impreciseWithoutDefault;

    // Natively typed properties are never reported, defaulted or not
    public int $typedWithDefault = 1;
    protected ?string $typedWithoutDefault = null;

    // Properties prefixed with `$_` stay ignored by convention
    public $_ignored;
}
