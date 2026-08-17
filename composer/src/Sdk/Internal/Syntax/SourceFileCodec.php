<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Syntax;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;

use function count;

/**
 * Decodes the syntax snapshot shared by external linter and analyzer messages.
 *
 * @internal
 */
final class SourceFileCodec
{
    private const MAXIMUM_NODE_KINDS = 256;

    /**
     * @param list<NodeKind> $kinds
     */
    public static function read(
        PayloadReader $reader,
        PHPVersion $phpVersion,
        array $kinds,
        string $path,
        string $contents,
    ): SourceFile {
        $targetCount = $reader->readU32();
        $targetIds = $reader->readU32List($targetCount);
        $nodeCount = $reader->readU32();
        $nodeRecords = $reader->readRaw($nodeCount * NodeStore::RECORD_SIZE);
        $nodes = new NodeStore($kinds, $nodeRecords, $nodeCount);
        $nameCount = $reader->readU32();
        $nameStarts = $reader->readRaw($nameCount * ResolvedNameStore::START_SIZE);
        $nameRecords = $reader->readRaw($nameCount * ResolvedNameStore::RECORD_SIZE);
        $names = new ResolvedNameStore($nameStarts, $nameRecords, $reader->readBytes(), $nameCount);
        $triviaCount = $reader->readU32();
        $triviaRecords = $reader->readRaw($triviaCount * TriviaStore::RECORD_SIZE);

        return new SourceFile(
            $phpVersion,
            $path,
            $contents,
            $targetIds,
            $nodes,
            $names,
            new TriviaStore($triviaRecords, $triviaCount),
            new LiteralStringStore('', '', 0),
        );
    }

    /**
     * @param list<NodeKind> $kinds
     */
    public static function readWithLiteralStrings(
        PayloadReader $reader,
        PHPVersion $phpVersion,
        array $kinds,
        string $path,
        string $contents,
    ): SourceFile {
        $targetCount = $reader->readU32();
        $targetIds = $reader->readU32List($targetCount);
        $nodeCount = $reader->readU32();
        $nodeRecords = $reader->readRaw($nodeCount * NodeStore::RECORD_SIZE);
        $nodes = new NodeStore($kinds, $nodeRecords, $nodeCount);
        $nameCount = $reader->readU32();
        $nameStarts = $reader->readRaw($nameCount * ResolvedNameStore::START_SIZE);
        $nameRecords = $reader->readRaw($nameCount * ResolvedNameStore::RECORD_SIZE);
        $names = new ResolvedNameStore($nameStarts, $nameRecords, $reader->readBytes(), $nameCount);
        $triviaCount = $reader->readU32();
        $triviaRecords = $reader->readRaw($triviaCount * TriviaStore::RECORD_SIZE);
        $literalStringCount = $reader->readU32();
        $literalStringRecords = $reader->readRaw($literalStringCount * LiteralStringStore::RECORD_SIZE);

        return new SourceFile(
            $phpVersion,
            $path,
            $contents,
            $targetIds,
            $nodes,
            $names,
            new TriviaStore($triviaRecords, $triviaCount),
            new LiteralStringStore($literalStringRecords, $reader->readBytes(), $literalStringCount),
        );
    }

    /**
     * @return list<NodeKind>
     */
    public static function readNodeKinds(PayloadReader $reader): array
    {
        $kinds = NodeKind::cases();
        $count = $reader->readCount(self::MAXIMUM_NODE_KINDS);
        if ($count !== count($kinds)) {
            throw new ProtocolException('The Mago and extension SDK node-kind tables differ.');
        }

        for ($index = 0; $index < $count; ++$index) {
            if ($reader->readString() !== $kinds[$index]->value) {
                throw new ProtocolException('The Mago and extension SDK node-kind tables differ.');
            }
        }

        return $kinds;
    }
}
