<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/**
 * Field groups available in projected method metadata.
 *
 * The appearing and declaring method identifiers are always included.
 *
 * @api
 */
final class MethodFields
{
    /** Normalized and source-cased method names. */
    public const NAMES = 1;

    /** Declaration and name locations. */
    public const LOCATIONS = 1 << 1;

    /** Parameter metadata. */
    public const PARAMETERS = 1 << 2;

    /** Native and effective return types. */
    public const RETURN_TYPES = 1 << 3;

    /** Template declarations. */
    public const TEMPLATES = 1 << 4;

    /** Resolved attributes and their arguments. */
    public const ATTRIBUTES = 1 << 5;

    /** Declared thrown types. */
    public const THROWN_TYPES = 1 << 6;

    /** Unconditional and conditional type assertions. */
    public const ASSERTIONS = 1 << 7;

    /** Accessed global variables. */
    public const GLOBALS = 1 << 8;

    /** Whether the method has a docblock. */
    public const DOCBLOCK = 1 << 9;

    /** Function-like metadata flags. */
    public const FLAGS = 1 << 10;

    /** PHP version availability ranges. */
    public const AVAILABLE_VERSIONS = 1 << 11;

    /** Method visibility and final, abstract, static, and constructor state. */
    public const METHOD_DETAILS = 1 << 12;

    /** Method-level where constraints. */
    public const WHERE_CONSTRAINTS = 1 << 13;

    /** Every optional field group. */
    public const ALL = (1 << 14) - 1;

    private function __construct() {}
}
