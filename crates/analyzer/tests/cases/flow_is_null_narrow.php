<?php

declare(strict_types=1);

namespace NullChecks;

use function is_null as is_null_alias;

function flow_is_null_narrow(null|string $value): string
{
    if (is_null($value)) {
        return 'null';
    }

    return $value;
}

final class NullCheckNode
{
    public function __construct(public ?object $value) {}

    public function value(): ?object
    {
        return $this->value;
    }
}

function nullsafe_method_guard(?NullCheckNode $node): ?NullCheckNode
{
    if (!is_null($node?->value())) {
        return require_node($node);
    }

    return null;
}

function nullsafe_property_guard(?NullCheckNode $node): ?object
{
    if (is_null_alias($node?->value)) {
        return null;
    }

    return $node->value;
}

function nullsafe_method_early_return(?NullCheckNode $node): ?NullCheckNode
{
    if (\is_null(value: $node?->value())) {
        return null;
    }

    return require_node($node);
}

function nullsafe_null_branch(?NullCheckNode $node): void
{
    if (is_null($node?->value())) {
        // @mago-expect analysis:possibly-null-argument
        require_node($node);
    }
}

function require_node(NullCheckNode $node): NullCheckNode
{
    return $node;
}

namespace Shadowed;

function is_null(?object $value): bool
{
    return (bool) $value;
}

function shadowed_predicate(?\NullChecks\NullCheckNode $node): void
{
    if (!is_null($node?->value())) {
        // @mago-expect analysis:possibly-null-argument
        \NullChecks\require_node($node);
    }
}

namespace StrictGuard;

final class Page
{
    public function __construct(public int $number, public int $total)
    {
    }
}

final class Response
{
    public function __construct(public ?Page $page)
    {
    }
}

function require_page(Page $page): int
{
    return $page->total;
}

function total_after_strict_guard(Response $response, int $page): int
{
    if ($response->page?->number !== $page) {
        return 0;
    }

    return $response->page->total;
}

function total_after_reversed_guard(Response $response, int $page): int
{
    if ($page !== $response->page?->number) {
        return 0;
    }

    return $response->page->total;
}

function strict_guard_true_branch(Response $response, int $page): void
{
    if ($response->page?->number !== $page) {
        // @mago-expect analysis:possibly-null-argument
        require_page($response->page);
    }
}

function nullable_value_no_narrow(Response $response, ?int $page): ?int
{
    if ($response->page?->number !== $page) {
        return 0;
    }

    /** @mago-expect analysis:possibly-null-property-access */
    return $response->page->total;
}

function loose_comparison_no_narrow(Response $response, int $page): ?int
{
    // Loose `!=` on a nullable operand stays suspicious even without narrowing.
    /** @mago-expect analysis:possibly-null-operand */
    if ($response->page?->number != $page) {
        return 0;
    }

    /** @mago-expect analysis:possibly-null-property-access */
    return $response->page->total;
}
