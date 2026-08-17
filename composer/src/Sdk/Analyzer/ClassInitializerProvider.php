<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Identifies framework lifecycle methods that initialize class properties.
 *
 * Mago analyzes the returned methods and their transitive calls using the same
 * definite-initialization rules as constructors.
 *
 * @api
 * @extends TargetedProvider<ClassTarget>
 */
interface ClassInitializerProvider extends TargetedProvider
{
    /** @return list<non-empty-string> */
    public function getClassInitializers(ClassInitializerProviderContext $context): array;
}
