<?php

declare(strict_types=1);

namespace Mago\Sdk;

/**
 * Reduces process-local extension data when a worker pool shuts down.
 *
 * The same reducer instance may be shared with linter rules and analyzer
 * plugins so they can accumulate data without communicating between workers.
 *
 * @api
 */
interface WorkerReducer
{
    /**
     * Export this worker's complete contribution as an opaque byte string.
     */
    public function collect(): string;

    /**
     * Reduce every worker contribution on the last surviving worker.
     */
    public function reduce(WorkerReductionContext $context): void;
}
