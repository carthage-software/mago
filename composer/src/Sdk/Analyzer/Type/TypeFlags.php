<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class TypeFlags
{
    public function __construct(
        public readonly bool $hadTemplate = false,
        public readonly bool $byReference = false,
        public readonly bool $referenceFree = false,
        public readonly bool $possiblyUndefinedFromTry = false,
        public readonly bool $possiblyUndefined = false,
        public readonly bool $ignoreNullableIssues = false,
        public readonly bool $ignoreFalsableIssues = false,
        public readonly bool $fromTemplateDefault = false,
        public readonly bool $populated = false,
        public readonly bool $nullsafeNull = false,
        public readonly bool $fromUnspecifiedTemplate = false,
    ) {}
}
