<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;

use function array_fill;
use function array_values;
use function count;
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
    private const MAXIMUM_COMPARISONS = 65_536;

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

    /**
     * Evaluates multiple, potentially different type relationships in one native request.
     *
     * @param list<TypeComparison> $comparisons
     *
     * @return list<bool>
     *
     * @mago-expect analysis:impossible-condition Runtime validation protects untyped callers.
     */
    public function compareMultiple(array $comparisons): array
    {
        $count = count($comparisons);
        if ($count === 0) {
            return [];
        }

        if ($count > self::MAXIMUM_COMPARISONS) {
            throw new InvalidArgumentException('A type-comparison batch cannot contain more than 65,536 comparisons.');
        }

        $results = array_fill(0, $count, false);
        $pending = [];
        $positions = [];
        $position = 0;
        foreach ($comparisons as $comparison) {
            if (!$comparison instanceof TypeComparison) {
                throw new InvalidArgumentException('A type-comparison batch must contain only TypeComparison values.');
            }

            $key = $comparison->cacheKey();
            $cached = $this->cached($comparison, $key);
            if ($cached !== null) {
                $results[$position++] = $cached;
                continue;
            }

            $pending[$key] ??= $comparison;
            $positions[$key][] = $position++;
        }

        if ($pending === []) {
            return $results;
        }

        $this->cancellation->throwIfCancelled();
        $requests = array_values($pending);
        $requestCount = count($requests);
        $comparison = $requests[0];
        $payload = $requestCount === 1
            ? Protocol::writeTypeComparisonRequest(
                $comparison->kind->value,
                $comparison->encodeLeft(),
                $comparison->encodeRight(),
            )
            : Protocol::writeTypeComparisonBatchRequest($requests);
        $response = $this->host->request($this->requestId, $payload);
        $this->cancellation->throwIfCancelled();
        $resolved = $requestCount === 1
            ? [Protocol::readTypeComparisonResponse($response)]
            : Protocol::readTypeComparisonBatchResponse($response, $requestCount);
        $index = 0;
        foreach ($pending as $key => $comparison) {
            $result = $resolved[$index++];
            $this->remember($comparison, $key, $result);
            foreach ($positions[$key] as $resultPosition) {
                $results[$resultPosition] = $result;
            }
        }

        return $results;
    }

    private function compare(int $operation, Type $left, Type $right): bool
    {
        $leftEncoding = $left->encode();
        $rightEncoding = $right->encode();
        $key = pack('C', $operation) . $leftEncoding . $rightEncoding;
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
            Protocol::writeTypeComparisonRequest($operation, $leftEncoding, $rightEncoding),
        );
        $this->cancellation->throwIfCancelled();

        $result = Protocol::readTypeComparisonResponse($response);
        if (!$left->isRequestReference() && !$right->isRequestReference() && $this->cache !== null) {
            return $this->cache->typeComparisons[$key] = $result;
        }

        return $this->results[$key] = $result;
    }

    private function cached(TypeComparison $comparison, string $key): ?bool
    {
        if (
            !$comparison->left->isRequestReference()
            && !$comparison->right->isRequestReference()
            && $this->cache !== null
        ) {
            return $this->cache->typeComparisons[$key] ?? null;
        }

        return $this->results[$key] ?? null;
    }

    private function remember(TypeComparison $comparison, string $key, bool $result): void
    {
        if (
            !$comparison->left->isRequestReference()
            && !$comparison->right->isRequestReference()
            && $this->cache !== null
        ) {
            $this->cache->typeComparisons[$key] = $result;
            return;
        }

        $this->results[$key] = $result;
    }
}
