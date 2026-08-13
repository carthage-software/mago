<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Closure;
use Mago\Sdk\Analyzer\Definition\ClassLikeDefinition;
use Mago\Sdk\Analyzer\Definition\ConstantDefinition;
use Mago\Sdk\Analyzer\Definition\FunctionDefinition;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\DefinitionCodec;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;

use function count;

/**
 * Transactional metadata access available before parallel analysis begins.
 *
 * Every mutation batch is immediately visible to later reads in the same
 * hook. Mago commits the complete transaction only when that hook succeeds.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
 */
final class MutableCodebase extends Codebase
{
    public function removeClassLike(string $name): ?ClassLikeDefinition
    {
        return $this->removeMultipleClassLikes([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeDefinition|null>
     */
    public function removeMultipleClassLikes(array $names): array
    {
        return $this->remove(Protocol::REMOVE_CLASS_LIKES, $names, DefinitionCodec::readClassLike(...));
    }

    public function insertClassLike(ClassLikeDefinition $definition): void
    {
        $this->insertMultipleClassLikes([$definition]);
    }

    /** @param list<ClassLikeDefinition> $definitions */
    public function insertMultipleClassLikes(array $definitions): void
    {
        $this->insert(Protocol::INSERT_CLASS_LIKES, $definitions, DefinitionCodec::writeClassLike(...));
    }

    public function removeFunction(string $name): ?FunctionDefinition
    {
        return $this->removeMultipleFunctions([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<FunctionDefinition|null>
     */
    public function removeMultipleFunctions(array $names): array
    {
        return $this->remove(Protocol::REMOVE_FUNCTIONS, $names, DefinitionCodec::readFunction(...));
    }

    public function insertFunction(FunctionDefinition $definition): void
    {
        $this->insertMultipleFunctions([$definition]);
    }

    /** @param list<FunctionDefinition> $definitions */
    public function insertMultipleFunctions(array $definitions): void
    {
        $this->insert(Protocol::INSERT_FUNCTIONS, $definitions, DefinitionCodec::writeFunction(...));
    }

    public function removeConstant(string $name): ?ConstantDefinition
    {
        return $this->removeMultipleConstants([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ConstantDefinition|null>
     */
    public function removeMultipleConstants(array $names): array
    {
        return $this->remove(Protocol::REMOVE_CONSTANTS, $names, DefinitionCodec::readConstant(...));
    }

    public function insertConstant(ConstantDefinition $definition): void
    {
        $this->insertMultipleConstants([$definition]);
    }

    /** @param list<ConstantDefinition> $definitions */
    public function insertMultipleConstants(array $definitions): void
    {
        $this->insert(Protocol::INSERT_CONSTANTS, $definitions, DefinitionCodec::writeConstant(...));
    }

    /**
     * @template T of object
     * @param list<string> $names
     * @param Closure(PayloadReader): T $read
     * @return list<T|null>
     */
    private function remove(int $operation, array $names, Closure $read): array
    {
        if ($names === []) {
            return [];
        }

        foreach ($names as $name) {
            if ($name === '') {
                throw new InvalidArgumentException('A codebase mutation name cannot be empty.');
            }
        }

        $this->cancellation->throwIfCancelled();
        $payload = Protocol::writeCodebaseRemovalRequest($this->cache->generation, $operation, $names);
        $response = $this->host->request($this->parentRequestId, $payload);
        [$reader, $resultCount] = Protocol::readCodebaseMutationResponse(
            $response,
            $this->cache->generation,
            $operation,
        );
        if ($resultCount !== count($names)) {
            throw new ProtocolException('A codebase removal response contains the wrong result count.');
        }

        $removed = [];
        for ($index = 0; $index < $resultCount; ++$index) {
            $removed[] = $reader->readBoolean() ? $read($reader) : null;
        }
        $reader->finish();

        return $removed;
    }

    /**
     * @template T of object
     * @param list<T> $definitions
     * @param Closure(PayloadWriter, T): void $write
     */
    private function insert(int $operation, array $definitions, Closure $write): void
    {
        if ($definitions === []) {
            return;
        }

        $this->cancellation->throwIfCancelled();
        $payload = Protocol::writeCodebaseInsertionRequest($this->cache->generation, $operation, $definitions, $write);
        $response = $this->host->request($this->parentRequestId, $payload);
        [$reader, $resultCount] = Protocol::readCodebaseMutationResponse(
            $response,
            $this->cache->generation,
            $operation,
        );
        if ($resultCount !== count($definitions)) {
            throw new ProtocolException('A codebase insertion response contains the wrong result count.');
        }
        $reader->finish();
    }
}
