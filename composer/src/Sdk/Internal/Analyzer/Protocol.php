<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Argument;
use Mago\Sdk\Analyzer\CallableSignatureProvider;
use Mago\Sdk\Analyzer\EffectiveCallableSignature;
use Mago\Sdk\Analyzer\ExpressionType;
use Mago\Sdk\Analyzer\FileAnalysis;
use Mago\Sdk\Analyzer\Invocation;
use Mago\Sdk\Analyzer\InvocationKind;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\ProjectAnalysis;
use Mago\Sdk\Analyzer\ReferenceKind;
use Mago\Sdk\Analyzer\ReferenceOrigin;
use Mago\Sdk\Analyzer\ReferenceSummary;
use Mago\Sdk\Analyzer\SymbolReference;
use Mago\Sdk\Analyzer\SymbolReferences;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Extension;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;

use function count;
use function intdiv;
use function pack;
use function strncmp;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:too-many-methods
 */
final class Protocol
{
    public const DESCRIBE_REQUEST = 1;
    public const RETURN_TYPE_REQUEST = 2;
    public const TYPE_COMPARISON_REQUEST = 3;
    public const CODEBASE_QUERY_REQUEST = 4;
    public const BEFORE_ANALYSIS_REQUEST = 5;
    public const AFTER_FILE_ANALYSIS_REQUEST = 6;
    public const AFTER_ANALYSIS_REQUEST = 7;
    public const ANALYSIS_QUERY_REQUEST = 8;
    public const AFTER_FILE_ANALYSIS_BATCH_REQUEST = 9;
    public const SYMBOL_REFERENCE_QUERY_REQUEST = 10;
    public const INITIALIZE_REQUEST = 11;
    public const CALLABLE_SIGNATURE_REQUEST = 12;
    public const GET_EXPRESSION_TYPES = 1;
    public const GET_ALL_EXPRESSION_TYPES = 2;
    public const GET_INFERRED_RETURN_TYPES = 3;
    public const GET_INFERRED_YIELD_KEY_TYPES = 4;
    public const GET_INFERRED_YIELD_VALUE_TYPES = 5;
    public const GET_REFERENCES_TO = 1;
    public const GET_REFERENCES_FROM = 2;
    public const GET_CLASS_LIKES = 1;
    public const GET_FUNCTIONS = 2;
    public const GET_METHODS = 3;
    public const GET_CONSTANTS = 4;
    public const GET_PROPERTIES = 5;
    public const GET_CLASS_CONSTANTS = 6;
    public const GET_ENUM_CASES = 7;
    public const LIST_CLASS_LIKES = 8;
    public const LIST_FUNCTIONS = 9;
    public const LIST_CONSTANTS = 10;
    public const GET_DECLARING_METHODS = 11;
    public const GET_DECLARING_PROPERTIES = 12;
    public const CHECK_EXISTENCE = 13;
    public const CHECK_MEMBER_EXISTENCE = 14;
    public const GET_CLASS_LIKE_RELATIONS = 15;
    public const GET_MAGIC_PROPERTIES = 16;
    public const GET_DECLARING_MAGIC_PROPERTIES = 17;
    public const EXISTS_CLASS = 1;
    public const EXISTS_INTERFACE = 2;
    public const EXISTS_TRAIT = 3;
    public const EXISTS_ENUM = 4;
    public const EXISTS_CLASS_LIKE = 5;
    public const EXISTS_NAMESPACE = 6;
    public const EXISTS_FUNCTION = 7;
    public const EXISTS_CONSTANT = 8;
    public const EXISTS_CLASS_OR_TRAIT = 9;
    public const EXISTS_CLASS_OR_INTERFACE = 10;
    public const EXISTS_METHOD = 1;
    public const EXISTS_PROPERTY = 2;
    public const EXISTS_CLASS_CONSTANT = 3;
    public const EXISTS_ENUM_CASE = 4;
    public const EXISTS_MAGIC_PROPERTY = 5;
    public const DIRECT_DESCENDANTS = 1;
    public const ALL_DESCENDANTS = 2;
    public const ALL_ANCESTORS = 3;
    public const ANY_CLASS_LIKE = 0;
    public const CLASS_LIKE_CLASS = 1;
    public const CLASS_LIKE_INTERFACE = 2;
    public const CLASS_LIKE_TRAIT = 3;
    public const CLASS_LIKE_ENUM = 4;
    public const TYPE_COMPARISON_EQUAL = 1;
    public const TYPE_COMPARISON_CONTAINED_BY = 2;
    public const TYPE_COMPARISON_CAN_BE_IDENTICAL = 3;
    private const MAGIC_U32 = 0x4D41_4E41;
    private const MAJOR = 1;
    private const MINOR = 0;
    private const VERSION_U32 = (self::MAJOR << 16) | self::MINOR;
    private const DESCRIBE_RESPONSE = 0x8001;
    private const RETURN_TYPE_RESPONSE = 0x8002;
    private const TYPE_COMPARISON_RESPONSE = 0x8003;
    private const CODEBASE_QUERY_RESPONSE = 0x8004;
    private const BEFORE_ANALYSIS_RESPONSE = 0x8005;
    private const AFTER_FILE_ANALYSIS_RESPONSE = 0x8006;
    private const AFTER_ANALYSIS_RESPONSE = 0x8007;
    private const ANALYSIS_QUERY_RESPONSE = 0x8008;
    private const AFTER_FILE_ANALYSIS_BATCH_RESPONSE = 0x8009;
    private const SYMBOL_REFERENCE_QUERY_RESPONSE = 0x800A;
    private const INITIALIZE_RESPONSE = 0x800B;
    private const CALLABLE_SIGNATURE_RESPONSE = 0x800C;
    private const RETURN_TYPE_REQUEST_HEADER = "MANA\x00\x01\x00\x01\x00\x02\x00\x00";
    private const CALLABLE_SIGNATURE_REQUEST_HEADER = "MANA\x00\x01\x00\x01\x00\x0C\x00\x00";
    private const UNHANDLED_RETURN_TYPE_RESPONSE = "MANA\x00\x01\x00\x01\x80\x02\x00\x00\x00";
    private const UNHANDLED_CALLABLE_SIGNATURE_RESPONSE = "MANA\x00\x01\x00\x01\x80\x0C\x00\x00\x00";
    private const INVOCATION_FUNCTION = 1;
    private const INVOCATION_INSTANCE_METHOD = 2;
    private const INVOCATION_STATIC_METHOD = 3;

    /** @return array{int<0, 65535>, PayloadReader} */
    public static function readRequest(string $payload): array
    {
        if (strncmp($payload, self::RETURN_TYPE_REQUEST_HEADER, 12) === 0) {
            return [self::RETURN_TYPE_REQUEST, new PayloadReader($payload, 12)];
        }

        if (strncmp($payload, self::CALLABLE_SIGNATURE_REQUEST_HEADER, 12) === 0) {
            return [self::CALLABLE_SIGNATURE_REQUEST, new PayloadReader($payload, 12)];
        }

        /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $header */
        $header = unpack('N3', $payload);
        if ($header[1] !== self::MAGIC_U32) {
            throw new ProtocolException('Invalid analyzer message magic.');
        }

        $version = $header[2];
        if ($version !== self::VERSION_U32) {
            $major = $version >> 16;
            $minor = $version & 0xffff;
            throw new ProtocolException("Unsupported analyzer protocol version {$major}.{$minor}.");
        }

        $message = $header[3];
        if (($message & 0xffff) !== 0) {
            throw new ProtocolException('Analyzer message reserved bits are non-zero.');
        }

        return [$message >> 16, new PayloadReader($payload, 12)];
    }

    public static function readDescribeRequest(PayloadReader $reader): PHPVersion
    {
        $version = new PHPVersion($reader->readU32());
        $reader->finish();

        return $version;
    }

    /** @return list<int<0, 65535>> */
    public static function readInitializationRequest(PayloadReader $reader): array
    {
        $count = $reader->readCount(65_536);
        $plugins = [];
        for ($index = 0; $index < $count; ++$index) {
            $plugins[] = $reader->readU16();
        }
        $reader->finish();

        return $plugins;
    }

    /**
     * @param array<int<0, 65535>, list<array{non-empty-string, string}>> $stubs
     */
    public static function writeInitializationResponse(array $stubs): string
    {
        $writer = self::createMessage(self::INITIALIZE_RESPONSE);
        $writer->writeCount($stubs);
        foreach ($stubs as $plugin => $pluginStubs) {
            $writer->writeU16($plugin);
            $writer->writeCount($pluginStubs);
            foreach ($pluginStubs as [$filename, $bytes]) {
                $writer->writeBytes($filename);
                $writer->writeBytes($bytes);
            }
        }

        return $writer->finish();
    }

    /**
     * @param non-empty-list<Extension> $extensions
     * @param list<RegisteredPlugin> $plugins
     */
    public static function writeDescribeResponse(array $extensions, array $plugins): string
    {
        $byExtension = [];
        foreach ($plugins as $plugin) {
            $byExtension[$plugin->extension][] = $plugin;
        }

        $writer = self::createMessage(self::DESCRIBE_RESPONSE);
        $writer->writeCount($extensions);
        foreach ($extensions as $extension) {
            $writer->writeString($extension->identifier);
            $writer->writeString($extension->name);
            $writer->writeString($extension->version);
            $writer->writeBoolean($extension->workerReducer !== null);
            $extensionPlugins = $byExtension[$extension->identifier] ?? [];
            $writer->writeCount($extensionPlugins);
            foreach ($extensionPlugins as $plugin) {
                $definition = $plugin->definition;
                $writer->writeString($definition->identifier);
                $writer->writeString($definition->name);
                $writer->writeString($definition->description);
                $writer->writeBoolean($definition->defaultEnabled);
                $flags = 0;
                $flags |= (int) ($plugin->beforeAnalysisHooks !== []);
                $flags |= (int) ($plugin->afterFileAnalysisHooks !== []) << 1;
                $flags |= (int) ($plugin->afterAnalysisHooks !== []) << 2;
                $flags |= (int) ($plugin->initializationHooks !== []) << 3;
                $writer->writeU8($flags);
                $writer->writeCount($definition->aliases);
                foreach ($definition->aliases as $alias) {
                    $writer->writeString($alias);
                }

                $writer->writeCount($plugin->functionProviders);
                foreach ($plugin->functionProviders as $provider) {
                    $writer->writeU16($provider->index);
                    $writer->writeU8((int) ($provider->provider instanceof CallableSignatureProvider));
                    $writer->writeCount($provider->targets);
                    foreach ($provider->targets as $target) {
                        $writer->writeU8($target->kind->value);
                        $writer->writeBytes($target->value);
                    }
                }

                $writer->writeCount($plugin->methodProviders);
                foreach ($plugin->methodProviders as $provider) {
                    $writer->writeU16($provider->index);
                    $writer->writeU8((int) ($provider->provider instanceof CallableSignatureProvider));
                    $writer->writeCount($provider->targets);
                    foreach ($provider->targets as $target) {
                        $writer->writeBytes($target->class);
                        $writer->writeBytes($target->method);
                    }
                }
            }
        }

        return $writer->finish();
    }

    /** @param array<string, Span> $spans */
    public static function writeAnalysisTypeQuery(
        int $generation,
        string $file,
        int $operation,
        array $spans = [],
    ): string {
        $writer = self::createMessage(self::ANALYSIS_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8($operation);
        $writer->writeBytes($file);
        if ($operation === self::GET_EXPRESSION_TYPES) {
            $writer->writeCount($spans);
            foreach ($spans as $span) {
                $writer->writeU32($span->start);
                $writer->writeU32($span->end);
            }
        }

        return $writer->finish();
    }

    /**
     * @return list<Type|null>
     */
    public static function readOptionalAnalysisTypeQueryResponse(
        string $payload,
        int $generation,
        string $file,
        int $operation,
    ): array {
        $reader = self::readAnalysisTypeQueryResponsePayload($payload, $generation, $file, $operation);
        $count = $reader->readCount(1_000_000);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            if (!$reader->readBoolean()) {
                $types[] = null;
                continue;
            }

            $types[] = TypeCodec::readComplete($reader);
        }

        $reader->finish();

        return $types;
    }

    /**
     * @return list<Type>
     */
    public static function readAnalysisTypeQueryResponse(
        string $payload,
        int $generation,
        string $file,
        int $operation,
    ): array {
        $reader = self::readAnalysisTypeQueryResponsePayload($payload, $generation, $file, $operation);
        $count = $reader->readCount(1_000_000);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            $types[] = TypeCodec::readComplete($reader);
        }

        $reader->finish();

        return $types;
    }

    /** @return list<ExpressionType> */
    public static function readAllExpressionTypesResponse(string $payload, int $generation, string $file): array
    {
        [$kind, $reader] = self::readRequest($payload);
        if (
            $kind !== self::ANALYSIS_QUERY_RESPONSE
            || $reader->readU64() !== $generation
            || $reader->readU8() !== self::GET_ALL_EXPRESSION_TYPES
            || $reader->readBytes() !== $file
        ) {
            throw new ProtocolException('An expression-type response does not match its request.');
        }

        $count = $reader->readCount(1_000_000);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            $types[] = new ExpressionType(
                new Span($reader->readU32(), $reader->readU32()),
                TypeCodec::readComplete($reader),
            );
        }
        $reader->finish();

        return $types;
    }

    /**
     * @param positive-int $requestId
     */
    public static function readLifecycleRequest(
        int $kind,
        PayloadReader $reader,
        HostClient $host,
        int $requestId,
        CancellationTokenInterface $cancellation,
    ): LifecycleRequest {
        $generation = $reader->readU64();
        $pluginCount = $reader->readU16();
        if ($pluginCount === 0) {
            throw new ProtocolException('An analyzer lifecycle request contains no plugins.');
        }
        $plugins = [];
        for ($index = 0; $index < $pluginCount; ++$index) {
            $plugins[] = $reader->readU16();
        }

        $analysis = match ($kind) {
            self::BEFORE_ANALYSIS_REQUEST => null,
            self::AFTER_FILE_ANALYSIS_REQUEST => self::readFileAnalysis(
                $reader,
                $host,
                $requestId,
                $generation,
                $cancellation,
            ),
            self::AFTER_FILE_ANALYSIS_BATCH_REQUEST => self::readFileAnalysisBatch(
                $reader,
                $host,
                $requestId,
                $generation,
                $cancellation,
            ),
            self::AFTER_ANALYSIS_REQUEST => self::readProjectAnalysis(
                $reader,
                $host,
                $requestId,
                $generation,
                $cancellation,
            ),
            default => throw new ProtocolException("Unknown analyzer lifecycle request kind {$kind}."),
        };

        $reader->finish();

        return new LifecycleRequest($generation, $plugins, $analysis);
    }

    private static function readAnalysisTypeQueryResponsePayload(
        string $payload,
        int $generation,
        string $file,
        int $operation,
    ): PayloadReader {
        [$kind, $reader] = self::readRequest($payload);
        if (
            $kind !== self::ANALYSIS_QUERY_RESPONSE
            || $reader->readU64() !== $generation
            || $reader->readU8() !== $operation
            || $reader->readBytes() !== $file
        ) {
            throw new ProtocolException('An analysis artifact response does not match its request.');
        }

        return $reader;
    }

    /**
     * @param positive-int $requestId
     */
    private static function readProjectAnalysis(
        PayloadReader $reader,
        HostClient $host,
        int $requestId,
        int $generation,
        CancellationTokenInterface $cancellation,
    ): ProjectAnalysis {
        $issueCount = $reader->readU32();
        $summary = self::readReferenceSummary($reader);
        $fileCount = $reader->readCount(1_000_000);
        $files = [];
        for ($index = 0; $index < $fileCount; ++$index) {
            $files[] = self::readFileAnalysis($reader, $host, $requestId, $generation, $cancellation);
        }

        return new ProjectAnalysis(
            $files,
            $issueCount,
            new SymbolReferences($host, $requestId, $generation, $cancellation, $summary),
        );
    }

    /**
     * @param list<int|ReportedIssue|string|null> $reportedIssues
     * @param list<int|string|MemberIdentifier|ReferenceOrigin|ReferenceKind> $contributedReferences
     * @mago-expect lint:halstead
     */
    public static function writeLifecycleResponse(
        int $requestKind,
        array $reportedIssues,
        array $contributedReferences = [],
    ): string {
        $responseKind = match ($requestKind) {
            self::BEFORE_ANALYSIS_REQUEST => self::BEFORE_ANALYSIS_RESPONSE,
            self::AFTER_FILE_ANALYSIS_REQUEST => self::AFTER_FILE_ANALYSIS_RESPONSE,
            self::AFTER_ANALYSIS_REQUEST => self::AFTER_ANALYSIS_RESPONSE,
            self::AFTER_FILE_ANALYSIS_BATCH_REQUEST => self::AFTER_FILE_ANALYSIS_BATCH_RESPONSE,
            default => throw new ProtocolException("Unknown analyzer lifecycle request kind {$requestKind}."),
        };
        $writer = self::createMessage($responseKind);
        $writer->writeU32(intdiv(count($reportedIssues), 3));
        for ($index = 0, $count = count($reportedIssues); $index < $count; $index += 3) {
            /** @var int<0, 65535> $plugin */
            $plugin = $reportedIssues[$index];
            /** @var ReportedIssue $reported */
            $reported = $reportedIssues[$index + 1];
            /** @var string|null $defaultFile */
            $defaultFile = $reportedIssues[$index + 2];
            $issue = $reported->issue;
            $writer->writeU16($plugin);
            $writer->writeU8($reported->level->value);
            $writer->writeString($reported->code);
            $writer->writeString($issue->message);
            $writer->writeCount($issue->notes);
            foreach ($issue->notes as $note) {
                $writer->writeString($note);
            }
            $writer->writeOptionalString($issue->help);
            $writer->writeOptionalString($issue->link);
            $writer->writeCount($issue->annotations);
            foreach ($issue->annotations as $annotation) {
                $writer->writeU8($annotation->kind->value);
                $writer->writeOptionalString($annotation->file ?? $defaultFile);
                $writer->writeU32($annotation->span->start);
                $writer->writeU32($annotation->span->end);
                $writer->writeOptionalString($annotation->message);
            }
            $writer->writeCount($issue->edits);
            foreach ($issue->edits as $edit) {
                $writer->writeOptionalString($edit->file ?? $defaultFile);
                $writer->writeU32($edit->span->start);
                $writer->writeU32($edit->span->end);
                $writer->writeU8($edit->safety->value);
                $writer->writeBytes($edit->newText);
            }
        }

        $writer->writeU32(intdiv(count($contributedReferences), 4));
        for ($index = 0, $count = count($contributedReferences); $index < $count; $index += 4) {
            /** @var int<0, 65535> $plugin */
            $plugin = $contributedReferences[$index];
            /** @var string|MemberIdentifier|ReferenceOrigin $source */
            $source = $contributedReferences[$index + 1];
            /** @var string|MemberIdentifier $target */
            $target = $contributedReferences[$index + 2];
            /** @var ReferenceKind $kind */
            $kind = $contributedReferences[$index + 3];
            $writer->writeU16($plugin);
            self::writeReferenceOrigin(
                $writer,
                $source instanceof ReferenceOrigin ? $source : ReferenceOrigin::symbol($source),
            );
            self::writeSymbolIdentifier($writer, $target);
            $writer->writeU8($kind->value);
        }

        return $writer->finish();
    }

    /**
     * @param positive-int $requestId
     */
    private static function readFileAnalysis(
        PayloadReader $reader,
        HostClient $host,
        int $requestId,
        int $generation,
        CancellationTokenInterface $cancellation,
    ): FileAnalysis {
        $file = $reader->readBytes();
        if ($file === '') {
            throw new ProtocolException('An analyzer file result contains an empty file name.');
        }

        return new FileAnalysis(
            $host,
            $requestId,
            $generation,
            $cancellation,
            $file,
            $reader->readU32(),
            $reader->readU32(),
            $reader->readU32(),
            $reader->readU32(),
            $reader->readU32(),
            self::readReferenceSummary($reader),
        );
    }

    /**
     * @param positive-int $requestId
     *
     * @return non-empty-list<FileAnalysis>
     */
    private static function readFileAnalysisBatch(
        PayloadReader $reader,
        HostClient $host,
        int $requestId,
        int $generation,
        CancellationTokenInterface $cancellation,
    ): array {
        $count = $reader->readCount(1_000_000);
        if ($count === 0) {
            throw new ProtocolException('An analyzer after-file batch contains no files.');
        }

        $files = [];
        for ($index = 0; $index < $count; ++$index) {
            $files[] = self::readFileAnalysis($reader, $host, $requestId, $generation, $cancellation);
        }

        return $files;
    }

    private static function readReferenceSummary(PayloadReader $reader): ReferenceSummary
    {
        return new ReferenceSummary($reader->readU64(), $reader->readU64(), $reader->readU64());
    }

    public static function readReturnTypeRequest(PayloadReader $reader): ReturnTypeRequest
    {
        $generation = $reader->readU64();
        $encodedKind = $reader->readU8();
        $kind = match ($encodedKind) {
            self::INVOCATION_FUNCTION => InvocationKind::Function,
            self::INVOCATION_INSTANCE_METHOD => InvocationKind::InstanceMethod,
            self::INVOCATION_STATIC_METHOD => InvocationKind::StaticMethod,
            default => throw new ProtocolException("Unknown analyzer invocation kind {$encodedKind}."),
        };

        $providerCount = $reader->readU16();
        if ($providerCount === 0) {
            throw new ProtocolException('A return-type request contains no providers.');
        }
        $providers = [];
        for ($index = 0; $index < $providerCount; ++$index) {
            $providers[] = $reader->readU16();
        }

        $method = $kind !== InvocationKind::Function;
        $declaringClass = $method ? $reader->readBytes() : null;
        $name = $reader->readBytes();
        if ($name === '' || $declaringClass === '') {
            throw new ProtocolException('Analyzer invocation names cannot be empty.');
        }

        $receiverType = $method ? TypeCodec::read($reader) : null;

        $span = new Span($reader->readU32(), $reader->readU32());
        $argumentCount = $reader->readU16();
        $arguments = [];
        for ($index = 0; $index < $argumentCount; ++$index) {
            $argumentName = $reader->readOptionalString();
            $unpacked = $reader->readBoolean();
            $placeholder = $reader->readBoolean();
            $argumentSpan = new Span($reader->readU32(), $reader->readU32());
            $expression = $reader->readBytes();
            $type = null;
            if ($reader->readBoolean()) {
                $description = $reader->readBytes();
                $type = TypeCodec::read($reader, $description);
            }
            $arguments[] = new Argument($argumentName, $unpacked, $placeholder, $argumentSpan, $expression, $type);
        }

        $reader->finish();

        return new ReturnTypeRequest(
            $generation,
            $providers,
            new Invocation($kind, $name, $declaringClass, $receiverType, $span, $arguments),
        );
    }

    public static function writeReturnTypeResponse(?Type $type): string
    {
        if ($type === null) {
            return self::UNHANDLED_RETURN_TYPE_RESPONSE;
        }

        return pack('N3C', self::MAGIC_U32, self::VERSION_U32, self::RETURN_TYPE_RESPONSE << 16, 1) . $type->encode();
    }

    /** @mago-expect lint:halstead */
    public static function writeCallableSignatureResponse(?EffectiveCallableSignature $signature): string
    {
        if ($signature === null) {
            return self::UNHANDLED_CALLABLE_SIGNATURE_RESPONSE;
        }

        $writer = self::createMessage(self::CALLABLE_SIGNATURE_RESPONSE);
        $writer->writeBoolean(true);
        $writer->writeBoolean($signature->allowsNamedArguments);
        $writer->writeCount($signature->parameters);
        foreach ($signature->parameters as $parameter) {
            $writer->writeOptionalString($parameter->name);
            $writer->writeBoolean($parameter->type !== null);
            if ($parameter->type !== null) {
                $writer->writeRaw($parameter->type->encode());
            }

            $writer->writeBoolean($parameter->closureThisType !== null);
            if ($parameter->closureThisType !== null) {
                $writer->writeRaw($parameter->closureThisType->encode());
            }

            $flags = 0;
            $flags |= (int) $parameter->byReference;
            $flags |= (int) $parameter->variadic << 1;
            $flags |= (int) $parameter->hasDefault << 2;
            $writer->writeU8($flags);
        }

        return $writer->finish();
    }

    public static function writeTypeComparisonRequest(int $operation, Type $left, Type $right): string
    {
        $writer = self::createMessage(self::TYPE_COMPARISON_REQUEST);
        $writer->writeU8($operation);
        $writer->writeRaw($left->encode());
        $writer->writeRaw($right->encode());

        return $writer->finish();
    }

    public static function readTypeComparisonResponse(string $payload): bool
    {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::TYPE_COMPARISON_RESPONSE) {
            throw new ProtocolException("Expected a type comparison response, received analyzer message {$kind}.");
        }

        $result = $reader->readBoolean();
        $reader->finish();

        return $result;
    }

    /**
     * @param list<string>|list<MemberIdentifier> $keys
     */
    public static function writeCodebaseQueryRequest(
        int $generation,
        int $operation,
        array $keys,
        ?int $classLikeKind = null,
    ): string {
        $writer = self::createMessage(self::CODEBASE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8($operation);
        if ($operation === self::GET_CLASS_LIKES) {
            $writer->writeU8($classLikeKind ?? self::ANY_CLASS_LIKE);
        }
        $writer->writeCount($keys);
        foreach ($keys as $key) {
            if ($key instanceof MemberIdentifier) {
                $writer->writeBytes($key->class);
                $writer->writeBytes($key->member);
                continue;
            }

            $writer->writeBytes($key);
        }

        return $writer->finish();
    }

    /** @return array{PayloadReader, int<0, 4294967295>} */
    public static function readCodebaseQueryResponse(
        string $payload,
        int $generation,
        int $operation,
        ?int $classLikeKind = null,
    ): array {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::CODEBASE_QUERY_RESPONSE) {
            throw new ProtocolException("Expected a codebase query response, received analyzer message {$kind}.");
        }
        if ($reader->readU64() !== $generation) {
            throw new ProtocolException('A codebase query response belongs to another analysis generation.');
        }
        if ($reader->readU8() !== $operation) {
            throw new ProtocolException('A codebase query response contains the wrong operation.');
        }
        if ($operation === self::GET_CLASS_LIKES && $reader->readU8() !== ($classLikeKind ?? self::ANY_CLASS_LIKE)) {
            throw new ProtocolException('A codebase query response contains the wrong class-like kind.');
        }

        return [$reader, $reader->readCount(65_536)];
    }

    public static function writeCodebaseListRequest(int $generation, int $operation, ?int $classLikeKind = null): string
    {
        $writer = self::createMessage(self::CODEBASE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8($operation);
        if ($operation === self::LIST_CLASS_LIKES) {
            $writer->writeU8($classLikeKind ?? self::ANY_CLASS_LIKE);
        }

        return $writer->finish();
    }

    /** @return list<string> */
    public static function readCodebaseListResponse(
        string $payload,
        int $generation,
        int $operation,
        ?int $classLikeKind = null,
    ): array {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::CODEBASE_QUERY_RESPONSE) {
            throw new ProtocolException("Expected a codebase query response, received analyzer message {$kind}.");
        }
        if ($reader->readU64() !== $generation || $reader->readU8() !== $operation) {
            throw new ProtocolException('A codebase list response does not match its request.');
        }
        if ($operation === self::LIST_CLASS_LIKES && $reader->readU8() !== ($classLikeKind ?? self::ANY_CLASS_LIKE)) {
            throw new ProtocolException('A codebase list response contains the wrong class-like kind.');
        }

        $count = $reader->readCount(65_536);
        $names = [];
        for ($index = 0; $index < $count; ++$index) {
            $names[] = $reader->readBytes();
        }
        $reader->finish();

        return $names;
    }

    /** @param list<string> $names */
    public static function writeCodebaseExistenceRequest(int $generation, int $predicate, array $names): string
    {
        $writer = self::createMessage(self::CODEBASE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8(self::CHECK_EXISTENCE);
        $writer->writeU8($predicate);
        $writer->writeCount($names);
        foreach ($names as $name) {
            $writer->writeBytes($name);
        }

        return $writer->finish();
    }

    /** @return list<bool> */
    public static function readCodebaseExistenceResponse(string $payload, int $generation, int $predicate): array
    {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::CODEBASE_QUERY_RESPONSE) {
            throw new ProtocolException("Expected a codebase query response, received analyzer message {$kind}.");
        }
        if (
            $reader->readU64() !== $generation
            || $reader->readU8() !== self::CHECK_EXISTENCE
            || $reader->readU8() !== $predicate
        ) {
            throw new ProtocolException('A codebase existence response does not match its request.');
        }

        $count = $reader->readCount(65_536);
        $results = [];
        for ($index = 0; $index < $count; ++$index) {
            $results[] = $reader->readBoolean();
        }
        $reader->finish();

        return $results;
    }

    /** @param list<MemberIdentifier> $members */
    public static function writeCodebaseMemberExistenceRequest(int $generation, int $predicate, array $members): string
    {
        $writer = self::createMessage(self::CODEBASE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8(self::CHECK_MEMBER_EXISTENCE);
        $writer->writeU8($predicate);
        $writer->writeCount($members);
        foreach ($members as $member) {
            $writer->writeBytes($member->class);
            $writer->writeBytes($member->member);
        }

        return $writer->finish();
    }

    /** @return list<bool> */
    public static function readCodebaseMemberExistenceResponse(string $payload, int $generation, int $predicate): array
    {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::CODEBASE_QUERY_RESPONSE) {
            throw new ProtocolException("Expected a codebase query response, received analyzer message {$kind}.");
        }
        if (
            $reader->readU64() !== $generation
            || $reader->readU8() !== self::CHECK_MEMBER_EXISTENCE
            || $reader->readU8() !== $predicate
        ) {
            throw new ProtocolException('A codebase member-existence response does not match its request.');
        }

        $count = $reader->readCount(65_536);
        $results = [];
        for ($index = 0; $index < $count; ++$index) {
            $results[] = $reader->readBoolean();
        }
        $reader->finish();

        return $results;
    }

    /** @param list<string> $names */
    public static function writeCodebaseRelationsRequest(int $generation, int $relation, array $names): string
    {
        $writer = self::createMessage(self::CODEBASE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8(self::GET_CLASS_LIKE_RELATIONS);
        $writer->writeU8($relation);
        $writer->writeCount($names);
        foreach ($names as $name) {
            $writer->writeBytes($name);
        }

        return $writer->finish();
    }

    /** @return list<list<string>> */
    public static function readCodebaseRelationsResponse(string $payload, int $generation, int $relation): array
    {
        [$kind, $reader] = self::readRequest($payload);
        if ($kind !== self::CODEBASE_QUERY_RESPONSE) {
            throw new ProtocolException("Expected a codebase query response, received analyzer message {$kind}.");
        }
        if (
            $reader->readU64() !== $generation
            || $reader->readU8() !== self::GET_CLASS_LIKE_RELATIONS
            || $reader->readU8() !== $relation
        ) {
            throw new ProtocolException('A codebase relation response does not match its request.');
        }

        $resultCount = $reader->readCount(65_536);
        $results = [];
        for ($resultIndex = 0; $resultIndex < $resultCount; ++$resultIndex) {
            $nameCount = $reader->readCount(65_536);
            $names = [];
            for ($nameIndex = 0; $nameIndex < $nameCount; ++$nameIndex) {
                $names[] = $reader->readBytes();
            }
            $results[] = $names;
        }
        $reader->finish();

        return $results;
    }

    /**
     * @param array<string, string|MemberIdentifier|ReferenceOrigin> $queries
     */
    public static function writeReferenceQuery(int $generation, int $operation, array $queries): string
    {
        $writer = self::createMessage(self::SYMBOL_REFERENCE_QUERY_REQUEST);
        $writer->writeU64($generation);
        $writer->writeU8($operation);
        $writer->writeCount($queries);
        foreach ($queries as $query) {
            if ($operation === self::GET_REFERENCES_TO) {
                /** @var string|MemberIdentifier $query */
                self::writeSymbolIdentifier($writer, $query);
                continue;
            }

            $origin = $query instanceof ReferenceOrigin ? $query : ReferenceOrigin::symbol($query);
            self::writeReferenceOrigin($writer, $origin);
        }

        return $writer->finish();
    }

    /** @return list<list<SymbolReference>> */
    public static function readReferenceQueryResponse(
        string $payload,
        int $generation,
        int $operation,
        int $expectedCount,
    ): array {
        [$kind, $reader] = self::readRequest($payload);
        if (
            $kind !== self::SYMBOL_REFERENCE_QUERY_RESPONSE
            || $reader->readU64() !== $generation
            || $reader->readU8() !== $operation
        ) {
            throw new ProtocolException('A symbol-reference response does not match its request.');
        }

        $resultCount = $reader->readCount(1_000_000);
        if ($resultCount !== $expectedCount) {
            throw new ProtocolException('A symbol-reference response contains the wrong number of results.');
        }
        $results = [];
        for ($resultIndex = 0; $resultIndex < $resultCount; ++$resultIndex) {
            $referenceCount = $reader->readCount(1_000_000);
            $references = [];
            for ($referenceIndex = 0; $referenceIndex < $referenceCount; ++$referenceIndex) {
                $source = self::readReferenceOrigin($reader);
                $target = self::readSymbolIdentifier($reader);
                $referenceKind = ReferenceKind::tryFrom($reader->readU8());
                if ($referenceKind === null) {
                    throw new ProtocolException('A symbol reference has an unknown kind.');
                }

                $references[] = new SymbolReference($source, $target, $referenceKind);
            }
            $results[] = $references;
        }
        $reader->finish();

        return $results;
    }

    private static function writeReferenceOrigin(PayloadWriter $writer, ReferenceOrigin $origin): void
    {
        if ($origin->file !== null) {
            $writer->writeU8(3);
            $writer->writeBytes($origin->file);
            return;
        }

        self::writeSymbolIdentifier($writer, $origin->symbol ?? '');
    }

    private static function writeSymbolIdentifier(PayloadWriter $writer, string|MemberIdentifier $symbol): void
    {
        if ($symbol instanceof MemberIdentifier) {
            $writer->writeU8(2);
            $writer->writeBytes($symbol->class);
            $writer->writeBytes($symbol->member);
            return;
        }

        $writer->writeU8(1);
        $writer->writeBytes($symbol);
    }

    private static function readReferenceOrigin(PayloadReader $reader): ReferenceOrigin
    {
        $kind = $reader->readU8();
        if ($kind === 3) {
            return ReferenceOrigin::file($reader->readBytes());
        }

        return ReferenceOrigin::symbol(self::readSymbolIdentifier($reader, $kind));
    }

    private static function readSymbolIdentifier(PayloadReader $reader, ?int $kind = null): string|MemberIdentifier
    {
        return match ($kind ?? $reader->readU8()) {
            1 => $reader->readBytes(),
            2 => new MemberIdentifier($reader->readBytes(), $reader->readBytes()),
            default => throw new ProtocolException('A symbol reference has an unknown endpoint kind.'),
        };
    }

    /** @param int<0, 65535> $kind */
    private static function createMessage(int $kind): PayloadWriter
    {
        return new PayloadWriter(pack('N3', self::MAGIC_U32, self::VERSION_U32, $kind << 16));
    }
}
