<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Syntax;

use Mago\Sdk\Internal\Syntax\LiteralStringStore;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;
use Mago\Sdk\Syntax\TriviaKind;
use PHPUnit\Framework\TestCase;

use function pack;
use function strlen;

final class SourceFileTest extends TestCase
{
    public function testPackedSourceDataIsExposed(): void
    {
        $noNode = 4_294_967_295;
        $nodeRecords =
            pack('CNNNNN', 0, 0, 10, $noNode, 1, $noNode)
            . pack('CNNNNN', 1, 1, 4, 0, $noNode, 2)
            . pack('CNNNNN', 1, 5, 8, 0, $noNode, $noNode);
        $nodeStore = new NodeStore([NodeKind::Program, NodeKind::FunctionCall], $nodeRecords, 3);
        $resolvedName = 'Psl\\Iter\\any';
        $nameStarts = pack('N', 1);
        $nameRecords = pack('NNNC', 4, 0, strlen($resolvedName), 0);
        $nameStore = new ResolvedNameStore($nameStarts, $nameRecords, $resolvedName, 1);
        $triviaStore = new TriviaStore(pack('CNN', 4, 0, 10), 1);
        $literalStringStore = new LiteralStringStore(pack('N3', 1, 0, 7), 'decoded', 1);
        $sourceFile = new SourceFile(
            PHPVersion::fromParts(8, 3),
            'fixture.php',
            '0123456789',
            [1, 2],
            $nodeStore,
            $nameStore,
            $triviaStore,
            $literalStringStore,
        );

        $targets = $sourceFile->getTargetNodes();
        self::assertCount(2, $targets);
        self::assertSame(NodeKind::FunctionCall, $targets[0]->kind);
        self::assertCount(3, $sourceFile->getNodes());
        self::assertSame($targets, $sourceFile->getNodes(NodeKind::FunctionCall));
        self::assertSame($targets, $sourceFile->getChildren($sourceFile->getNode(0)));
        self::assertSame(1, $sourceFile->getFirstDescendant($sourceFile->getNode(0), NodeKind::FunctionCall)?->id);
        self::assertNull($sourceFile->getFirstDescendant($sourceFile->getNode(0), NodeKind::LiteralString));
        self::assertSame(0, $sourceFile->getParent($targets[0])?->id);
        self::assertSame('123', $sourceFile->getText($targets[0]));
        self::assertSame($resolvedName, $sourceFile->getResolvedName($targets[0])?->name);
        self::assertSame('decoded', $sourceFile->getLiteralString($targets[0]));
        self::assertNull($sourceFile->getLiteralString($targets[1]));
        self::assertSame(TriviaKind::DocBlockComment, $sourceFile->getTrivia()[0]->kind);
    }
}
