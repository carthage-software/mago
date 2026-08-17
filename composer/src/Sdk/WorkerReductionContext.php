<?php

declare(strict_types=1);

namespace Mago\Sdk;

/**
 * Context provided to a worker reducer on the last surviving worker.
 *
 * @api
 */
final class WorkerReductionContext
{
    /**
     * @param non-empty-list<string> $workerPayloads Contributions in stable worker-index order.
     *
     * @internal Constructed by the worker runtime.
     */
    public function __construct(
        public readonly array $workerPayloads,
        public readonly CancellationTokenInterface $cancellation,
    ) {}
}
