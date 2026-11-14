<?php

declare(strict_types=1);

namespace Issue2698;

use RuntimeException;

/**
 * @psalm-assert true $fact
 *
 * @throws RuntimeException
 */
function invariant(bool $fact, string $message): void
{
    if (!$fact) {
        throw new RuntimeException($message);
    }
}

final class Config
{
    public function __construct(
        private readonly string $environment,
        private readonly ?string $tenant,
        private readonly ?string $client,
    ) {}

    /**
     * @return list<?string>
     *
     * @throws RuntimeException
     */
    public function get(): array
    {
        if ('prod' === $this->environment) {
            invariant(!empty($this->tenant), 'tenant');
            invariant(!empty($this->client), 'client');
        }

        return [$this->tenant, $this->client];
    }

    /** @throws RuntimeException */
    public function repeated(): string
    {
        invariant(!empty($this->tenant), 'tenant');
        // @mago-expect analysis:redundant-condition
        invariant(!empty($this->tenant), 'tenant again');

        return $this->tenant;
    }
}

/** @throws RuntimeException */
function properties_of_another_object(Config $first, Holder $holder): void
{
    invariant(!empty($holder->left), 'left');
    invariant(!empty($holder->right), 'right');
    $first->get();
}

/** @throws RuntimeException */
function derived(?string $left, ?string $right): string
{
    invariant($left !== null || $right !== null, 'one of them');
    invariant($left === null, 'left is null');

    return $right;
}

final class Holder
{
    public ?string $left = null;
    public ?string $right = null;
}
