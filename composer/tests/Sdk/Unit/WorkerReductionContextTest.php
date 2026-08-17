<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit;

use Mago\Sdk\Extension;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\WorkerReducer;
use Mago\Sdk\WorkerReductionContext;
use PHPUnit\Framework\TestCase;

final class WorkerReductionContextTest extends TestCase
{
    public function testPreservesOrderedPayloadsAndCancellation(): void
    {
        $cancellation = new SignalCancellationToken();
        $context = new WorkerReductionContext(['first', '', 'third'], $cancellation);

        self::assertSame(['first', '', 'third'], $context->workerPayloads);
        self::assertSame($cancellation, $context->cancellation);
    }

    public function testExtensionRetainsItsWorkerReducer(): void
    {
        $reducer = new class() implements WorkerReducer {
            public function collect(): string
            {
                return '';
            }

            public function reduce(WorkerReductionContext $context): void {}
        };

        $extension = new Extension('acme/reducer', 'Reducer', '1.0.0', workerReducer: $reducer);

        self::assertSame($reducer, $extension->workerReducer);
    }
}
