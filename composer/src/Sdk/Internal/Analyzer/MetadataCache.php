<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MethodMetadataProjection;

/** @internal */
final class MetadataCache
{
    /** @var array<int, array<string, object|null>> */
    public array $values = [];

    /** @var array<int, list<string>> */
    public array $lists = [];

    /** @var array<int, array<string, bool>> */
    public array $existence = [];

    /** @var array<int, array<string, list<string>>> */
    public array $relations = [];

    /** @var array<string, list<MethodMetadataProjection>> */
    public array $methodProjections = [];

    /** @var array<string, bool> */
    public array $typeComparisons = [];

    public function __construct(
        public readonly int $generation,
    ) {}
}
