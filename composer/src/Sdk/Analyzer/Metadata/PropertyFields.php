<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/**
 * Field groups available in projected property metadata.
 *
 * @api
 */
final class PropertyFields
{
    /** Property name. */
    public const NAMES = 1;

    /** Declaration and name locations. */
    public const LOCATIONS = 1 << 1;

    /** Read and write visibility. */
    public const VISIBILITY = 1 << 2;

    /** Declared, effective, write, and default types. */
    public const TYPES = 1 << 3;

    /** Resolved attributes and their arguments. */
    public const ATTRIBUTES = 1 << 4;

    /** Property metadata flags. */
    public const FLAGS = 1 << 5;

    /** Property hook metadata. */
    public const HOOKS = 1 << 6;

    /** PHP version availability ranges. */
    public const AVAILABLE_VERSIONS = 1 << 7;

    /** Every field group. */
    public const ALL = (1 << 8) - 1;

    private function __construct() {}
}
