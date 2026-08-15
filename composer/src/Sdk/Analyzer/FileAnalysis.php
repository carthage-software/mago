<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;

use function array_key_exists;
use function array_keys;

/**
 * Completed semantic artifacts for one source file. Expensive data is fetched lazily.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
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

    private ?SourceFile $sourceFile = null;

    /**
     * @internal
     * @param positive-int $requestId
     * @param list<NodeKind> $nodeKinds
     * @param non-empty-string $file
     */
    public function __construct(
        private readonly HostClient $host,
        private readonly int $requestId,
        private readonly int $generation,
        private readonly CancellationTokenInterface $cancellation,
        private readonly PHPVersion $phpVersion,
        private readonly array $nodeKinds,
        public readonly string $file,
        public readonly int $size,
        public readonly int $expressionCount,
        public readonly int $inferredReturnCount,
        public readonly int $inferredYieldKeyCount,
        public readonly int $inferredYieldValueCount,
        public readonly ReferenceSummary $references,
    ) {}

    public function getExpressionType(Node|Span $selection): ?Type
    {
        return $this->getMultipleExpressionTypes([$selection])[0];
    }

    /**
     * @param list<Node|Span> $selections
     *
     * @return list<Type|null>
     */
    public function getMultipleExpressionTypes(array $selections): array
    {
        $missing = [];
        $spans = [];
        foreach ($selections as $selection) {
            $span = $selection instanceof Node ? $selection->span : $selection;
            $spans[] = $span;
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
     * Returns the exact in-memory source analyzed by Mago, including its complete syntax and resolved names.
     *
     * The snapshot is requested only on the first call and is never reconstructed from the filesystem.
     *
     * @mago-expect lint:halstead
     */
    public function getSourceFile(): SourceFile
    {
        if ($this->sourceFile !== null) {
            return $this->sourceFile;
        }

        $this->cancellation->throwIfCancelled();
        $response = $this->host->request(
            $this->requestId,
            Protocol::writeAnalysisTypeQuery($this->generation, $this->file, Protocol::GET_SOURCE_FILE),
        );

        return $this->sourceFile = Protocol::readSourceFileResponse(
            $response,
            $this->generation,
            $this->file,
            $this->phpVersion,
            $this->nodeKinds,
        );
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
