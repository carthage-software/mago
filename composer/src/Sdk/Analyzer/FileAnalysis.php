<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Analyzer\TypeCodec;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;

use function array_key_exists;
use function array_keys;
use function count;
use function strlen;
use function unpack;

/**
 * Completed semantic artifacts for one source file.
 *
 * Data requested by a registered hook may be embedded in the lifecycle batch. Other expensive data is fetched lazily.
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

    private ?SourceFile $sourceFile;

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
        ?SourceFile $sourceFile,
        private readonly bool $hasLocalExpressionTypes,
        private readonly string $expressionTypeRecords,
        private readonly string $encodedExpressionTypes,
        public readonly string $file,
        public readonly int $size,
        public readonly int $expressionCount,
        public readonly int $inferredReturnCount,
        public readonly int $inferredYieldKeyCount,
        public readonly int $inferredYieldValueCount,
        public readonly ReferenceSummary $references,
    ) {
        $this->sourceFile = $sourceFile;
    }

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

        if ($missing !== [] && $this->hasLocalExpressionTypes) {
            foreach ($missing as $key => $span) {
                $this->expressionTypes[$key] = $this->readLocalExpressionType($span);
            }
        }

        if ($missing !== [] && !$this->hasLocalExpressionTypes) {
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

            [$types, $prefetched] = Protocol::readOptionalAnalysisTypeQueryResponse(
                $response,
                $this->generation,
                $this->file,
                Protocol::GET_EXPRESSION_TYPES,
            );

            foreach (array_keys($missing) as $index => $key) {
                $this->expressionTypes[$key] = $types[$index];
            }
            for ($index = 0, $count = count($prefetched); $index < $count; $index += 3) {
                /** @var int $start */
                $start = $prefetched[$index];
                /** @var int $end */
                $end = $prefetched[$index + 1];
                /** @var Type $type */
                $type = $prefetched[$index + 2];
                $this->expressionTypes[$start . ':' . $end] = $type;
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
     * When it was not embedded in the lifecycle batch, the snapshot is requested on the first call. It is never
     * reconstructed from the filesystem.
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

    private function readLocalExpressionType(Span $span): ?Type
    {
        $low = 0;
        $high = $this->expressionCount - 1;
        while ($low <= $high) {
            $middle = ($low + $high) >> 1;
            /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>, 4: int<0, 4294967295>} $record */
            $record = unpack('N4', $this->expressionTypeRecords, $middle * 16);
            if ($record[1] < $span->start || $record[1] === $span->start && $record[2] < $span->end) {
                $low = $middle + 1;
                continue;
            }
            if ($record[1] > $span->start || $record[2] > $span->end) {
                $high = $middle - 1;
                continue;
            }

            if (($record[3] + $record[4]) > strlen($this->encodedExpressionTypes)) {
                return null;
            }

            return TypeCodec::readComplete(new PayloadReader($this->encodedExpressionTypes, $record[3]));
        }

        return null;
    }
}
