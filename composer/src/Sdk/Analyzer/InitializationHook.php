<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Initializes one analyzer plugin before Mago parses any project source.
 *
 * @api
 */
interface InitializationHook
{
    public function initialize(InitializationContext $context): void;
}
