<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Linter;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Extension;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;

use function count;
use function pack;
use function strlen;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class Protocol
{
    public const DESCRIBE_REQUEST = 1;
    public const LINT_FILE_REQUEST = 2;

    private const MAGIC = 'MLNT';
    private const MAGIC_U32 = 0x4D4C_4E54;
    private const MAJOR = 1;
    private const MINOR = 0;
    private const VERSION_U32 = (self::MAJOR << 16) | self::MINOR;
    private const DESCRIBE_RESPONSE = 0x8001;
    private const LINT_FILE_RESPONSE = 0x8002;
    private const MAXIMUM_NODE_KINDS = 256;
    private const MAXIMUM_ISSUES = 1_000_000;

    /**
     * @return array{int<0, 65535>, PayloadReader}
     */
    public static function readRequest(string $payload): array
    {
        /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $header */
        $header = unpack('N3', $payload);
        if ($header[1] !== self::MAGIC_U32) {
            throw new ProtocolException('Invalid linter message magic.');
        }

        $version = $header[2];
        if ($version !== self::VERSION_U32) {
            $major = $version >> 16;
            $minor = $version & 0xffff;
            throw new ProtocolException("Unsupported linter protocol version {$major}.{$minor}.");
        }

        $message = $header[3];
        $reserved = $message & 0xffff;
        if ($reserved !== 0) {
            throw new ProtocolException("Linter message reserved bits are non-zero: {$reserved}.");
        }

        return [$message >> 16, new PayloadReader($payload, 12)];
    }

    /**
     * @return array{PHPVersion, list<NodeKind>}
     */
    public static function readDescribeRequest(PayloadReader $reader): array
    {
        $version = new PHPVersion($reader->readU32());
        $kinds = self::readNodeKinds($reader);
        $reader->finish();

        return [$version, $kinds];
    }

    /**
     * @param non-empty-list<Extension> $extensions
     */
    public static function writeDescribeResponse(array $extensions): string
    {
        $writer = self::createMessage(self::DESCRIBE_RESPONSE);
        $writer->writeCount($extensions);
        foreach ($extensions as $extension) {
            $writer->writeString($extension->identifier);
            $writer->writeString($extension->name);
            $writer->writeString($extension->version);
            $writer->writeCount($extension->linterRules);
            foreach ($extension->linterRules as $rule) {
                $definition = $rule->getDefinition();
                $writer->writeString($definition->code);
                $writer->writeString($definition->name);
                $writer->writeString($definition->description);
                $writer->writeU8($definition->defaultLevel->value);
                $writer->writeBoolean($definition->defaultEnabled);
                $writer->writeCount($definition->targets);
                foreach ($definition->targets as $target) {
                    $writer->writeString($target->value);
                }
            }
        }

        return $writer->finish();
    }

    /**
     * @param list<NodeKind> $kinds
     */
    public static function readLintRequest(PayloadReader $reader, PHPVersion $phpVersion, array $kinds): LintRequest
    {
        $file = $reader->readBytes();
        $source = $reader->readBytes();
        $activeRules = self::readActiveRules($reader);
        $targetIds = self::readNodeIds($reader);
        $nodeCount = $reader->readU32();
        $nodeRecords = $reader->readRaw($nodeCount * NodeStore::RECORD_SIZE);
        $nodes = new NodeStore($kinds, $nodeRecords, $nodeCount);
        $nameCount = $reader->readU32();
        $nameStarts = $reader->readRaw($nameCount * ResolvedNameStore::START_SIZE);
        $nameRecords = $reader->readRaw($nameCount * ResolvedNameStore::RECORD_SIZE);
        $names = new ResolvedNameStore($nameStarts, $nameRecords, $reader->readBytes(), $nameCount);
        $triviaCount = $reader->readU32();
        $triviaRecords = $reader->readRaw($triviaCount * TriviaStore::RECORD_SIZE);
        $trivia = new TriviaStore($triviaRecords, $triviaCount);

        $reader->finish();

        return new LintRequest(
            $activeRules,
            new SourceFile($phpVersion, $file, $source, $targetIds, $nodes, $names, $trivia),
        );
    }

    /**
     * @return list<int<0, 65535>>
     */
    private static function readActiveRules(PayloadReader $reader): array
    {
        $activeRules = [];
        $count = $reader->readU16();
        for ($index = 0; $index < $count; ++$index) {
            $activeRules[] = $reader->readU16();
        }

        return $activeRules;
    }

    /**
     * @return list<NodeKind>
     */
    private static function readNodeKinds(PayloadReader $reader): array
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

    /**
     * @return array<int, int<0, 4294967295>>
     */
    private static function readNodeIds(PayloadReader $reader): array
    {
        $count = $reader->readU32();

        return $reader->readU32List($count);
    }

    /**
     * @param list<int<0, 65535>|Issue> $reportedIssues Flat rule index and issue pairs.
     */
    public static function writeLintResponse(array $reportedIssues): string
    {
        $valueCount = count($reportedIssues);
        $issueCount = $valueCount >> 1;
        if ($issueCount > self::MAXIMUM_ISSUES) {
            throw new ProtocolException('A linter response contains too many issues.');
        }

        $payload = pack('N4', self::MAGIC_U32, self::VERSION_U32, self::LINT_FILE_RESPONSE << 16, $issueCount);
        for ($index = 0; $index < $valueCount; $index += 2) {
            /** @var int<0, 65535> $ruleIndex */
            $ruleIndex = $reportedIssues[$index];
            /** @var Issue $issue */
            $issue = $reportedIssues[$index + 1];
            $payload .= pack('nN', $ruleIndex, strlen($issue->message)) . $issue->message;
            $payload .= pack('N', count($issue->notes));
            foreach ($issue->notes as $note) {
                $payload .= pack('N', strlen($note)) . $note;
            }

            $payload .= $issue->help === null ? "\0" : "\1" . pack('N', strlen($issue->help)) . $issue->help;
            $payload .= $issue->link === null ? "\0" : "\1" . pack('N', strlen($issue->link)) . $issue->link;
            $payload .= pack('N', count($issue->annotations));
            foreach ($issue->annotations as $annotation) {
                $payload .= pack('CNN', $annotation->kind->value, $annotation->span->start, $annotation->span->end);
                $payload .= $annotation->message === null
                    ? "\0"
                    : "\1" . pack('N', strlen($annotation->message)) . $annotation->message;
            }

            $payload .= pack('N', count($issue->edits));
            foreach ($issue->edits as $edit) {
                if ($edit->file !== null) {
                    throw new ProtocolException('A linter text edit cannot target another file.');
                }

                $payload .=
                    pack('NNCN', $edit->span->start, $edit->span->end, $edit->safety->value, strlen($edit->newText))
                    . $edit->newText;
            }
        }

        return $payload;
    }

    /**
     * @param int<0, 65535> $kind
     */
    private static function createMessage(int $kind): PayloadWriter
    {
        return new PayloadWriter(pack('N3', self::MAGIC_U32, self::VERSION_U32, $kind << 16));
    }
}
