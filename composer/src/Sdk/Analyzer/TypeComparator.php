<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;

use function pack;

/**
 * Performs codebase-aware type comparisons using Mago's native type system.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:no-isset
 */
final class TypeComparator
{
    /** @var array<string, bool> */
    private array $results = [];

    /**
     * @param positive-int $requestId
     * @internal
     */
    public function __construct(
        private readonly HostClient $host,
        private readonly int $requestId,
        private readonly CancellationTokenInterface $cancellation,
        private readonly ?MetadataCache $cache = null,
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
        $key = pack('C', $operation) . $left->encode() . $right->encode();
        $results =
            !$left->isRequestReference() && !$right->isRequestReference() && $this->cache !== null
                ? $this->cache->typeComparisons
                : $this->results;
        if (isset($results[$key])) {
            return $results[$key];
        }

        $this->cancellation->throwIfCancelled();
        $response = $this->host->request(
            $this->requestId,
            Protocol::writeTypeComparisonRequest($operation, $left, $right),
        );
        $this->cancellation->throwIfCancelled();

        $result = Protocol::readTypeComparisonResponse($response);
        if (!$left->isRequestReference() && !$right->isRequestReference() && $this->cache !== null) {
            return $this->cache->typeComparisons[$key] = $result;
        }

        return $this->results[$key] = $result;
    }
}
