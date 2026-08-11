<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;

/**
 * Performs codebase-aware type comparisons using Mago's native type system.
 *
 * @api
 */
final class TypeComparator
{
    /**
     * @param positive-int $requestId
     * @internal
     */
    public function __construct(
        private readonly HostClient $host,
        private readonly int $requestId,
        private readonly CancellationTokenInterface $cancellation,
    ) {}

    public function equals(Type $left, Type $right): bool
    {
        return $this->compare(Protocol::TYPE_COMPARISON_EQUAL, $left, $right);
    }

    public function isContainedBy(Type $input, Type $container): bool
    {
        return $this->compare(Protocol::TYPE_COMPARISON_CONTAINED_BY, $input, $container);
    }

    public function canBeIdentical(Type $left, Type $right): bool
    {
        return $this->compare(Protocol::TYPE_COMPARISON_CAN_BE_IDENTICAL, $left, $right);
    }

    private function compare(int $operation, Type $left, Type $right): bool
    {
        $this->cancellation->throwIfCancelled();
        $response = $this->host->request(
            $this->requestId,
            Protocol::writeTypeComparisonRequest($operation, $left, $right),
        );
        $this->cancellation->throwIfCancelled();

        return Protocol::readTypeComparisonResponse($response);
    }
}
