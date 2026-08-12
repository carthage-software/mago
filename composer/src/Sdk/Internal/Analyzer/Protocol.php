<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Argument;
use Mago\Sdk\Analyzer\Invocation;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Extension;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;

use function pack;
use function strncmp;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:psl-string-functions
 * @mago-expect lint:too-many-methods
 */
final class Protocol
{
    public const DESCRIBE_REQUEST = 1;
    public const RETURN_TYPE_REQUEST = 2;
    public const TYPE_COMPARISON_REQUEST = 3;
    public const CODEBASE_QUERY_REQUEST = 4;
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
    private const RETURN_TYPE_REQUEST_HEADER = "MANA\x00\x01\x00\x00\x00\x02\x00\x00";
    private const UNHANDLED_RETURN_TYPE_RESPONSE = "MANA\x00\x01\x00\x00\x80\x02\x00\x00\x00";
    private const INVOCATION_FUNCTION = 1;
    private const INVOCATION_METHOD = 2;

    /** @return array{int<0, 65535>, PayloadReader} */
    public static function readRequest(string $payload): array
    {
        if (strncmp($payload, self::RETURN_TYPE_REQUEST_HEADER, 12) === 0) {
            return [self::RETURN_TYPE_REQUEST, new PayloadReader($payload, 12)];
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
            $extensionPlugins = $byExtension[$extension->identifier] ?? [];
            $writer->writeCount($extensionPlugins);
            foreach ($extensionPlugins as $plugin) {
                $definition = $plugin->definition;
                $writer->writeString($definition->identifier);
                $writer->writeString($definition->name);
                $writer->writeString($definition->description);
                $writer->writeBoolean($definition->defaultEnabled);
                $writer->writeCount($definition->aliases);
                foreach ($definition->aliases as $alias) {
                    $writer->writeString($alias);
                }

                $writer->writeCount($plugin->functionProviders);
                foreach ($plugin->functionProviders as $provider) {
                    $writer->writeU16($provider->index);
                    $writer->writeCount($provider->targets);
                    foreach ($provider->targets as $target) {
                        $writer->writeU8($target->kind->value);
                        $writer->writeBytes($target->value);
                    }
                }

                $writer->writeCount($plugin->methodProviders);
                foreach ($plugin->methodProviders as $provider) {
                    $writer->writeU16($provider->index);
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

    public static function readReturnTypeRequest(PayloadReader $reader): ReturnTypeRequest
    {
        $generation = $reader->readU64();
        $kind = $reader->readU8();
        if ($kind !== self::INVOCATION_FUNCTION && $kind !== self::INVOCATION_METHOD) {
            throw new ProtocolException("Unknown analyzer invocation kind {$kind}.");
        }

        $providerCount = $reader->readU16();
        if ($providerCount === 0) {
            throw new ProtocolException('A return-type request contains no providers.');
        }
        $providers = [];
        for ($index = 0; $index < $providerCount; ++$index) {
            $providers[] = $reader->readU16();
        }

        $class = $kind === self::INVOCATION_METHOD ? $reader->readBytes() : null;
        $name = $reader->readBytes();
        if ($name === '' || $class === '') {
            throw new ProtocolException('Analyzer invocation names cannot be empty.');
        }

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
            $kind === self::INVOCATION_METHOD,
            $providers,
            new Invocation($name, $class, $span, $arguments),
        );
    }

    public static function writeReturnTypeResponse(?Type $type): string
    {
        if ($type === null) {
            return self::UNHANDLED_RETURN_TYPE_RESPONSE;
        }

        return pack('N3C', self::MAGIC_U32, self::VERSION_U32, self::RETURN_TYPE_RESPONSE << 16, 1) . $type->encode();
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

    /** @param int<0, 65535> $kind */
    private static function createMessage(int $kind): PayloadWriter
    {
        return new PayloadWriter(pack('N3', self::MAGIC_U32, self::VERSION_U32, $kind << 16));
    }
}
