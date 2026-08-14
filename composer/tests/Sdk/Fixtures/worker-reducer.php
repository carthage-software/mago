<?php

declare(strict_types=1);

use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use Mago\Sdk\WorkerReducer;
use Mago\Sdk\WorkerReductionContext;

require_once __DIR__ . '/../../../../vendor/autoload.php';

/**
 * @mago-expect lint:file-name
 */
final class AuditWorkerReducer implements WorkerReducer
{
    public function __construct(
        private readonly string $identifier,
        private readonly string $directory,
    ) {}

    public function collect(): string
    {
        return $this->identifier . ':' . (string) getmypid();
    }

    public function reduce(WorkerReductionContext $context): void
    {
        $context->cancellation->throwIfCancelled();
        file_put_contents($this->directory . '/' . $this->identifier . '.txt', implode("\n", [
            $this->identifier,
            (string) getmypid(),
            $context->cancellation->isCancelled() ? 'cancelled' : 'active',
            ...$context->workerPayloads,
        ]));
    }
}

$directory = getenv('MAGO_REDUCTION_AUDIT_DIRECTORY');
if ($directory === false || $directory === '') {
    throw new RuntimeException('MAGO_REDUCTION_AUDIT_DIRECTORY is required.');
}

(new Worker(
    new Extension(
        identifier: 'mago/reduction-alpha',
        name: 'Reduction Alpha',
        version: '1.0.0',
        workerReducer: new AuditWorkerReducer('alpha', $directory),
    ),
    new Extension(
        identifier: 'mago/reduction-beta',
        name: 'Reduction Beta',
        version: '1.0.0',
        workerReducer: new AuditWorkerReducer('beta', $directory),
    ),
))->run();
