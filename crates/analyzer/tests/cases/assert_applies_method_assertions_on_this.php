<?php

declare(strict_types=1);

abstract class AssertThisMapping
{
    /**
     * @phpstan-assert-if-true AssertThisManyToOne $this
     */
    final public function isManyToOne(): bool
    {
        return $this instanceof AssertThisManyToOne;
    }
}

final class AssertThisManyToOne extends AssertThisMapping
{
    /**
     * @var list<string>
     */
    public array $columns = [];
}

final class AssertThisOneToMany extends AssertThisMapping
{
}

/**
 * @return list<string>
 */
function assert_applies_method_assertions_on_this(AssertThisManyToOne|AssertThisOneToMany $mapping): array
{
    assert($mapping->isManyToOne());

    return $mapping->columns;
}
