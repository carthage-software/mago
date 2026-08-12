<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\GenericParent;
use Mago\Sdk\Analyzer\Type\Variance;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class TemplateMetadata
{
    public function __construct(
        public readonly string $name,
        public readonly GenericParent $definingEntity,
        public readonly Type $constraint,
        public readonly ?Type $default,
        public readonly Variance $variance,
        public readonly bool $readonly,
    ) {}
}
