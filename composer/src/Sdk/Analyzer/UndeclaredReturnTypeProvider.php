<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Marks a return-type provider that only handles unresolved callables.
 *
 * Mago skips these providers when native metadata already declares the
 * function or method, avoiding needless extension requests.
 *
 * @api
 */
interface UndeclaredReturnTypeProvider {}
