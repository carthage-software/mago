<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Marks a callable-signature provider that replaces existing declarations.
 *
 * Ordinary callable-signature providers are dispatched only when Mago cannot
 * resolve a declared callable. Implement this interface when the provider must
 * refine the parameters of a callable that already exists in the codebase.
 *
 * @api
 */
interface CallableSignatureOverride extends CallableSignatureProvider {}
