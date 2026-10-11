<?php

declare(strict_types=1);

namespace Acme;

use BackedEnum;
use Stringable;

function accepts_stringable(BackedEnum|Stringable|string $value): string
{
    return match (true) {
        $value instanceof BackedEnum => (string) $value->value,
        default => (string) $value,
    };
}

interface Uri
{
    public function __toString(): string;
}

final class BasicUri implements Uri
{
    public function __construct(
        private string $value,
    ) {}

    #[\Override]
    public function __toString(): string
    {
        return $this->value;
    }
}

function x(Uri $uri): void
{
    namespace\accepts_stringable($uri);
}

interface ChildUri extends Uri {}

function from_uri(Uri $uri): Stringable
{
    return $uri;
}

function from_child_uri(ChildUri $uri): Stringable
{
    namespace\accepts_stringable($uri);

    return $uri;
}

interface ExplicitUri extends Stringable {}

function from_explicit_uri(ExplicitUri $uri): Stringable
{
    namespace\accepts_stringable($uri);

    return $uri;
}

interface NotStringable {}

function reject_unrelated_interface(NotStringable $value): void
{
    // @mago-expect analysis:possibly-invalid-argument
    namespace\accepts_stringable($value);
}

function check_uri(Uri $uri): string
{
    // @mago-expect analysis:redundant-condition
    if ($uri instanceof Stringable) {
        return (string) $uri;
    }

    return '';
}

$uri = new BasicUri('https://acme.com');
x($uri);
