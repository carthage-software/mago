<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Span;

use function array_key_exists;
use function array_keys;

/**
 * Completed semantic artifacts for one source file. Expensive data is fetched lazily.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class FileAnalysis
{
    /**
     * @var array<string, Type|null>
     */
    private array $expressionTypes = [];

    /**
     * @var array<int, list<Type>>
     */
    private array $inferredTypes = [];

    /**
     * @internal
     * @param positive-int $requestId
     * @param non-empty-string $file
     */
    public function __construct(
        private readonly HostClient $host,
        private readonly int $requestId,
        private readonly int $generation,
        private readonly CancellationTokenInterface $cancellation,
        public readonly string $file,
        public readonly int $size,
        public readonly int $expressionCount,
        public readonly int $inferredReturnCount,
        public readonly int $inferredYieldKeyCount,
        public readonly int $inferredYieldValueCount,
        public readonly ReferenceSummary $references,
    ) {}

    public function getExpressionType(Span $span): ?Type
    {
        return $this->getMultipleExpressionTypes([$span])[0];
    }

    /**
     * @param list<Span> $spans
     *
     * @return list<Type|null>
     */
    public function getMultipleExpressionTypes(array $spans): array
    {
        $missing = [];
        foreach ($spans as $span) {
            $key = $span->start . ':' . $span->end;
            if (!array_key_exists($key, $this->expressionTypes)) {
                $missing[$key] = $span;
            }
        }

        if ($missing !== []) {
            $this->cancellation->throwIfCancelled();
            $response = $this->host->request(
                $this->requestId,
                Protocol::writeAnalysisTypeQuery(
                    $this->generation,
                    $this->file,
                    Protocol::GET_EXPRESSION_TYPES,
                    $missing,
                ),
            );

            $types = Protocol::readOptionalAnalysisTypeQueryResponse(
                $response,
                $this->generation,
                $this->file,
                Protocol::GET_EXPRESSION_TYPES,
            );

            foreach (array_keys($missing) as $index => $key) {
                $this->expressionTypes[$key] = $types[$index];
            }
        }

        $types = [];
        foreach ($spans as $span) {
            $types[] = $this->expressionTypes[$span->start . ':' . $span->end];
        }

        return $types;
    }

    /**
     * @return list<ExpressionType>
     */
    public function getAllExpressionTypes(): array
    {
        $this->cancellation->throwIfCancelled();
        $response = $this->host->request(
            $this->requestId,
            Protocol::writeAnalysisTypeQuery($this->generation, $this->file, Protocol::GET_ALL_EXPRESSION_TYPES),
        );

        return Protocol::readAllExpressionTypesResponse($response, $this->generation, $this->file);
    }

    /**
     * @return list<Type>
     */
    public function getInferredReturnTypes(): array
    {
        return $this->getInferredTypes(Protocol::GET_INFERRED_RETURN_TYPES);
    }

    /**
     * @return list<Type>
     */
    public function getInferredYieldKeyTypes(): array
    {
        return $this->getInferredTypes(Protocol::GET_INFERRED_YIELD_KEY_TYPES);
    }

    /**
     * @return list<Type>
     */
    public function getInferredYieldValueTypes(): array
    {
        return $this->getInferredTypes(Protocol::GET_INFERRED_YIELD_VALUE_TYPES);
    }

    /**
     * @return list<Type>
     */
    private function getInferredTypes(int $operation): array
    {
        if (!array_key_exists($operation, $this->inferredTypes)) {
            $this->cancellation->throwIfCancelled();
            $response = $this->host->request(
                $this->requestId,
                Protocol::writeAnalysisTypeQuery($this->generation, $this->file, $operation),
            );

            $this->inferredTypes[$operation] = Protocol::readAnalysisTypeQueryResponse(
                $response,
                $this->generation,
                $this->file,
                $operation,
            );
        }

        return $this->inferredTypes[$operation];
    }
}
