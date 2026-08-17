<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;

use function array_key_exists;
use function array_keys;
use function count;

/**
 * Lazy, read-only access to Mago's final merged symbol-reference graph.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class SymbolReferences
{
    public readonly int $body;
    public readonly int $signature;
    public readonly int $maps;

    /** @var array<string, list<SymbolReference>> */
    private array $to = [];

    /** @var array<string, list<SymbolReference>> */
    private array $from = [];

    /**
     * @internal
     * @param positive-int $requestId
     */
    public function __construct(
        private readonly HostClient $host,
        private readonly int $requestId,
        private readonly int $generation,
        private readonly CancellationTokenInterface $cancellation,
        public readonly ReferenceSummary $summary,
    ) {
        $this->body = $summary->body;
        $this->signature = $summary->signature;
        $this->maps = $summary->maps;
    }

    /** @return list<SymbolReference> */
    public function getReferencesTo(string|MemberIdentifier $target): array
    {
        return $this->getMultipleReferencesTo([$target])[0];
    }

    /**
     * @param list<string|MemberIdentifier> $targets
     * @return list<list<SymbolReference>>
     */
    public function getMultipleReferencesTo(array $targets): array
    {
        $missing = [];
        foreach ($targets as $target) {
            $key = self::symbolKey($target);
            if (!array_key_exists($key, $this->to)) {
                $missing[$key] = $target;
            }
        }

        if ($missing !== []) {
            $this->cancellation->throwIfCancelled();
            $response = $this->host->request(
                $this->requestId,
                Protocol::writeReferenceQuery($this->generation, Protocol::GET_REFERENCES_TO, $missing),
            );
            $references = Protocol::readReferenceQueryResponse(
                $response,
                $this->generation,
                Protocol::GET_REFERENCES_TO,
                count($missing),
            );
            foreach (array_keys($missing) as $index => $key) {
                $this->to[$key] = $references[$index];
            }
        }

        $references = [];
        foreach ($targets as $target) {
            $references[] = $this->to[self::symbolKey($target)];
        }

        return $references;
    }

    /** @return list<SymbolReference> */
    public function getReferencesFrom(string|MemberIdentifier|ReferenceOrigin $source): array
    {
        return $this->getMultipleReferencesFrom([$source])[0];
    }

    /**
     * @param list<string|MemberIdentifier|ReferenceOrigin> $sources
     * @return list<list<SymbolReference>>
     */
    public function getMultipleReferencesFrom(array $sources): array
    {
        $missing = [];
        foreach ($sources as $source) {
            $origin = $source instanceof ReferenceOrigin ? $source : ReferenceOrigin::symbol($source);
            $key = self::originKey($origin);
            if (!array_key_exists($key, $this->from)) {
                $missing[$key] = $origin;
            }
        }

        if ($missing !== []) {
            $this->cancellation->throwIfCancelled();
            $response = $this->host->request(
                $this->requestId,
                Protocol::writeReferenceQuery($this->generation, Protocol::GET_REFERENCES_FROM, $missing),
            );
            $references = Protocol::readReferenceQueryResponse(
                $response,
                $this->generation,
                Protocol::GET_REFERENCES_FROM,
                count($missing),
            );
            foreach (array_keys($missing) as $index => $key) {
                $this->from[$key] = $references[$index];
            }
        }

        $references = [];
        foreach ($sources as $source) {
            $origin = $source instanceof ReferenceOrigin ? $source : ReferenceOrigin::symbol($source);
            $references[] = $this->from[self::originKey($origin)];
        }

        return $references;
    }

    private static function symbolKey(string|MemberIdentifier $symbol): string
    {
        return $symbol instanceof MemberIdentifier ? "m\0{$symbol->class}\0{$symbol->member}" : "s\0{$symbol}";
    }

    private static function originKey(ReferenceOrigin $origin): string
    {
        if ($origin->file !== null) {
            return "f\0{$origin->file}";
        }

        return self::symbolKey($origin->symbol ?? '');
    }
}
