<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Closure;
use Mago\Sdk\Analyzer\Metadata\ClassConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\ConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\EnumCaseMetadata;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Metadata\PropertyMetadata;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\FunctionLikeKind;
use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\Analyzer\MetadataCodec;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Protocol\PayloadReader;

use function array_key_exists;
use function count;
use function strtolower;

/**
 * Read-only access to Mago's frozen, merged codebase metadata.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:too-many-methods
 */
final class Codebase
{
    /**
     * @internal
     * @param positive-int $parentRequestId
     */
    public function __construct(
        protected readonly HostClient $host,
        protected readonly int $parentRequestId,
        protected readonly CancellationTokenInterface $cancellation,
        protected readonly MetadataCache $cache,
    ) {}

    public function getClass(string $name): ?ClassLikeMetadata
    {
        return $this->getMultipleClasses([$name])[0] ?? null;
    }

    public function classExists(string $name): bool
    {
        return $this->checkMultipleClassesExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleClassesExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_CLASS, $names);
    }

    public function interfaceExists(string $name): bool
    {
        return $this->checkMultipleInterfacesExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleInterfacesExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_INTERFACE, $names);
    }

    public function traitExists(string $name): bool
    {
        return $this->checkMultipleTraitsExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleTraitsExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_TRAIT, $names);
    }

    public function enumExists(string $name): bool
    {
        return $this->checkMultipleEnumsExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleEnumsExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_ENUM, $names);
    }

    public function classLikeExists(string $name): bool
    {
        return $this->checkMultipleClassLikesExist([$name])[0] ?? false;
    }

    public function classOrTraitExists(string $name): bool
    {
        return $this->checkMultipleClassesOrTraitsExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleClassesOrTraitsExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_CLASS_OR_TRAIT, $names);
    }

    public function classOrInterfaceExists(string $name): bool
    {
        return $this->checkMultipleClassesOrInterfacesExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleClassesOrInterfacesExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_CLASS_OR_INTERFACE, $names);
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleClassLikesExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_CLASS_LIKE, $names);
    }

    public function namespaceExists(string $name): bool
    {
        return $this->checkMultipleNamespacesExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleNamespacesExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_NAMESPACE, $names);
    }

    /** @return list<string> */
    public function getClassNames(): array
    {
        return $this->listNames(Protocol::LIST_CLASS_LIKES, Protocol::CLASS_LIKE_CLASS);
    }

    /** @return list<string> */
    public function getInterfaceNames(): array
    {
        return $this->listNames(Protocol::LIST_CLASS_LIKES, Protocol::CLASS_LIKE_INTERFACE);
    }

    /** @return list<string> */
    public function getTraitNames(): array
    {
        return $this->listNames(Protocol::LIST_CLASS_LIKES, Protocol::CLASS_LIKE_TRAIT);
    }

    /** @return list<string> */
    public function getEnumNames(): array
    {
        return $this->listNames(Protocol::LIST_CLASS_LIKES, Protocol::CLASS_LIKE_ENUM);
    }

    /** @return list<string> */
    public function getClassLikeNames(): array
    {
        return $this->listNames(Protocol::LIST_CLASS_LIKES, Protocol::ANY_CLASS_LIKE);
    }

    /** @return list<string> */
    public function getDirectClassDescendants(string $name): array
    {
        return $this->getMultipleDirectClassDescendants([$name])[0] ?? [];
    }

    /**
     * @param list<string> $names
     * @return list<list<string>>
     */
    public function getMultipleDirectClassDescendants(array $names): array
    {
        return $this->getRelations(Protocol::DIRECT_DESCENDANTS, $names);
    }

    /** @return list<string> */
    public function getClassDescendants(string $name): array
    {
        return $this->getMultipleClassDescendants([$name])[0] ?? [];
    }

    /**
     * @param list<string> $names
     * @return list<list<string>>
     */
    public function getMultipleClassDescendants(array $names): array
    {
        return $this->getRelations(Protocol::ALL_DESCENDANTS, $names);
    }

    /** @return list<string> */
    public function getClassAncestors(string $name): array
    {
        return $this->getMultipleClassAncestors([$name])[0] ?? [];
    }

    /**
     * @param list<string> $names
     * @return list<list<string>>
     */
    public function getMultipleClassAncestors(array $names): array
    {
        return $this->getRelations(Protocol::ALL_ANCESTORS, $names);
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeMetadata|null>
     */
    public function getMultipleClasses(array $names): array
    {
        return $this->queryNames(
            Protocol::GET_CLASS_LIKES,
            $names,
            Protocol::CLASS_LIKE_CLASS,
            MetadataCodec::readClassLike(...),
        );
    }

    public function getInterface(string $name): ?ClassLikeMetadata
    {
        return $this->getMultipleInterfaces([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeMetadata|null>
     */
    public function getMultipleInterfaces(array $names): array
    {
        return $this->queryNames(
            Protocol::GET_CLASS_LIKES,
            $names,
            Protocol::CLASS_LIKE_INTERFACE,
            MetadataCodec::readClassLike(...),
        );
    }

    public function getTrait(string $name): ?ClassLikeMetadata
    {
        return $this->getMultipleTraits([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeMetadata|null>
     */
    public function getMultipleTraits(array $names): array
    {
        return $this->queryNames(
            Protocol::GET_CLASS_LIKES,
            $names,
            Protocol::CLASS_LIKE_TRAIT,
            MetadataCodec::readClassLike(...),
        );
    }

    public function getEnum(string $name): ?ClassLikeMetadata
    {
        return $this->getMultipleEnums([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeMetadata|null>
     */
    public function getMultipleEnums(array $names): array
    {
        return $this->queryNames(
            Protocol::GET_CLASS_LIKES,
            $names,
            Protocol::CLASS_LIKE_ENUM,
            MetadataCodec::readClassLike(...),
        );
    }

    public function getClassLike(string $name): ?ClassLikeMetadata
    {
        return $this->getMultipleClassLikes([$name])[0] ?? null;
    }

    /**
     * @param list<string> $names
     * @return list<ClassLikeMetadata|null>
     */
    public function getMultipleClassLikes(array $names): array
    {
        return $this->queryNames(
            Protocol::GET_CLASS_LIKES,
            $names,
            Protocol::ANY_CLASS_LIKE,
            MetadataCodec::readClassLike(...),
        );
    }

    public function getFunction(string $name): ?FunctionLikeMetadata
    {
        return $this->getMultipleFunctions([$name])[0] ?? null;
    }

    public function functionExists(string $name): bool
    {
        return $this->checkMultipleFunctionsExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleFunctionsExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_FUNCTION, $names);
    }

    /** @return list<string> */
    public function getFunctionNames(): array
    {
        return $this->listNames(Protocol::LIST_FUNCTIONS);
    }

    /**
     * @param list<string> $names
     * @return list<FunctionLikeMetadata|null>
     */
    public function getMultipleFunctions(array $names): array
    {
        return $this->queryNames(Protocol::GET_FUNCTIONS, $names, null, MetadataCodec::readFunctionLike(...));
    }

    public function getFunctionLike(FunctionLikeIdentifier $identifier): ?FunctionLikeMetadata
    {
        return $this->getMultipleFunctionLikes([$identifier])[0] ?? null;
    }

    /**
     * @param list<FunctionLikeIdentifier> $identifiers
     * @return list<FunctionLikeMetadata|null>
     */
    public function getMultipleFunctionLikes(array $identifiers): array
    {
        $keys = [];
        foreach ($identifiers as $identifier) {
            $keys[] = match ($identifier->kind) {
                FunctionLikeKind::Function_ => "f\0" . strtolower($identifier->name),
                FunctionLikeKind::Method => "m\0"
                    . strtolower($identifier->class ?? '')
                    . "\0"
                    . strtolower($identifier->name),
                FunctionLikeKind::Closure => "c\0" . $identifier->name,
            };
        }

        return $this->query(
            Protocol::GET_FUNCTION_LIKES,
            null,
            $identifiers,
            $keys,
            MetadataCodec::readFunctionLike(...),
        );
    }

    public function getMethod(string $class, string $method): ?FunctionLikeMetadata
    {
        return $this->getMultipleMethods([new MemberIdentifier($class, $method)])[0] ?? null;
    }

    public function methodExists(string $class, string $method): bool
    {
        return $this->checkMultipleMethodsExist([new MemberIdentifier($class, $method)])[0] ?? false;
    }

    /**
     * @param list<MemberIdentifier> $methods
     * @return list<bool>
     */
    public function checkMultipleMethodsExist(array $methods): array
    {
        return $this->checkMemberExistence(Protocol::EXISTS_METHOD, $methods);
    }

    public function getDeclaringMethod(string $class, string $method): ?FunctionLikeMetadata
    {
        return $this->getMultipleDeclaringMethods([new MemberIdentifier($class, $method)])[0] ?? null;
    }

    /**
     * @param list<MemberIdentifier> $methods
     * @return list<FunctionLikeMetadata|null>
     */
    public function getMultipleDeclaringMethods(array $methods): array
    {
        return $this->queryMembers(Protocol::GET_DECLARING_METHODS, $methods, MetadataCodec::readFunctionLike(...));
    }

    /**
     * @param list<MemberIdentifier> $methods
     * @return list<FunctionLikeMetadata|null>
     */
    public function getMultipleMethods(array $methods): array
    {
        return $this->queryMembers(Protocol::GET_METHODS, $methods, MetadataCodec::readFunctionLike(...));
    }

    public function getConstant(string $name): ?ConstantMetadata
    {
        return $this->getMultipleConstants([$name])[0] ?? null;
    }

    public function constantExists(string $name): bool
    {
        return $this->checkMultipleConstantsExist([$name])[0] ?? false;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    public function checkMultipleConstantsExist(array $names): array
    {
        return $this->checkExistence(Protocol::EXISTS_CONSTANT, $names);
    }

    /** @return list<string> */
    public function getConstantNames(): array
    {
        return $this->listNames(Protocol::LIST_CONSTANTS);
    }

    /**
     * @param list<string> $names
     * @return list<ConstantMetadata|null>
     */
    public function getMultipleConstants(array $names): array
    {
        return $this->queryNames(Protocol::GET_CONSTANTS, $names, null, MetadataCodec::readConstant(...));
    }

    public function getProperty(string $class, string $property): ?PropertyMetadata
    {
        return $this->getMultipleProperties([new MemberIdentifier($class, $property)])[0] ?? null;
    }

    public function propertyExists(string $class, string $property): bool
    {
        return $this->checkMultiplePropertiesExist([new MemberIdentifier($class, $property)])[0] ?? false;
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<bool>
     */
    public function checkMultiplePropertiesExist(array $properties): array
    {
        return $this->checkMemberExistence(Protocol::EXISTS_PROPERTY, $properties);
    }

    public function getDeclaringProperty(string $class, string $property): ?PropertyMetadata
    {
        return $this->getMultipleDeclaringProperties([new MemberIdentifier($class, $property)])[0] ?? null;
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<PropertyMetadata|null>
     */
    public function getMultipleDeclaringProperties(array $properties): array
    {
        return $this->queryMembers(Protocol::GET_DECLARING_PROPERTIES, $properties, MetadataCodec::readProperty(...));
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<PropertyMetadata|null>
     */
    public function getMultipleProperties(array $properties): array
    {
        return $this->queryMembers(Protocol::GET_PROPERTIES, $properties, MetadataCodec::readProperty(...));
    }

    public function getMagicProperty(string $class, string $property): ?PropertyMetadata
    {
        return $this->getMultipleMagicProperties([new MemberIdentifier($class, $property)])[0] ?? null;
    }

    public function magicPropertyExists(string $class, string $property): bool
    {
        return $this->checkMultipleMagicPropertiesExist([new MemberIdentifier($class, $property)])[0] ?? false;
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<bool>
     */
    public function checkMultipleMagicPropertiesExist(array $properties): array
    {
        return $this->checkMemberExistence(Protocol::EXISTS_MAGIC_PROPERTY, $properties);
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<PropertyMetadata|null>
     */
    public function getMultipleMagicProperties(array $properties): array
    {
        return $this->queryMembers(Protocol::GET_MAGIC_PROPERTIES, $properties, MetadataCodec::readProperty(...));
    }

    public function getDeclaringMagicProperty(string $class, string $property): ?PropertyMetadata
    {
        return $this->getMultipleDeclaringMagicProperties([new MemberIdentifier($class, $property)])[0] ?? null;
    }

    /**
     * @param list<MemberIdentifier> $properties
     * @return list<PropertyMetadata|null>
     */
    public function getMultipleDeclaringMagicProperties(array $properties): array
    {
        return $this->queryMembers(
            Protocol::GET_DECLARING_MAGIC_PROPERTIES,
            $properties,
            MetadataCodec::readProperty(...),
        );
    }

    public function getClassConstant(string $class, string $constant): ?ClassConstantMetadata
    {
        return $this->getMultipleClassConstants([new MemberIdentifier($class, $constant)])[0] ?? null;
    }

    public function classConstantExists(string $class, string $constant): bool
    {
        return $this->checkMultipleClassConstantsExist([new MemberIdentifier($class, $constant)])[0] ?? false;
    }

    /**
     * @param list<MemberIdentifier> $constants
     * @return list<bool>
     */
    public function checkMultipleClassConstantsExist(array $constants): array
    {
        return $this->checkMemberExistence(Protocol::EXISTS_CLASS_CONSTANT, $constants);
    }

    /**
     * @param list<MemberIdentifier> $constants
     * @return list<ClassConstantMetadata|null>
     */
    public function getMultipleClassConstants(array $constants): array
    {
        return $this->queryMembers(Protocol::GET_CLASS_CONSTANTS, $constants, MetadataCodec::readClassConstant(...));
    }

    public function getEnumCase(string $enum, string $case): ?EnumCaseMetadata
    {
        return $this->getMultipleEnumCases([new MemberIdentifier($enum, $case)])[0] ?? null;
    }

    public function enumCaseExists(string $enum, string $case): bool
    {
        return $this->checkMultipleEnumCasesExist([new MemberIdentifier($enum, $case)])[0] ?? false;
    }

    /**
     * @param list<MemberIdentifier> $cases
     * @return list<bool>
     */
    public function checkMultipleEnumCasesExist(array $cases): array
    {
        return $this->checkMemberExistence(Protocol::EXISTS_ENUM_CASE, $cases);
    }

    /**
     * @param list<MemberIdentifier> $cases
     * @return list<EnumCaseMetadata|null>
     */
    public function getMultipleEnumCases(array $cases): array
    {
        return $this->queryMembers(Protocol::GET_ENUM_CASES, $cases, MetadataCodec::readEnumCase(...));
    }

    /**
     * @template T of object
     * @param list<string> $names
     * @param Closure(PayloadReader): T $decode
     * @return list<T|null>
     */
    private function queryNames(int $operation, array $names, ?int $classLikeKind, Closure $decode): array
    {
        $keys = [];
        foreach ($names as $name) {
            if ($name === '') {
                throw new InvalidArgumentException('Codebase metadata names cannot be empty.');
            }
            $keys[] = $operation === Protocol::GET_CONSTANTS ? $name : strtolower($name);
        }

        return $this->query($operation, $classLikeKind, $names, $keys, $decode);
    }

    /**
     * @template T of object
     * @param list<MemberIdentifier> $members
     * @param Closure(PayloadReader): T $decode
     * @return list<T|null>
     */
    private function queryMembers(int $operation, array $members, Closure $decode): array
    {
        $keys = [];
        foreach ($members as $member) {
            $memberName = match ($operation) {
                Protocol::GET_METHODS, Protocol::GET_DECLARING_METHODS => strtolower($member->member),
                default => $member->member,
            };
            $keys[] = strtolower($member->class) . "\0" . $memberName;
        }

        return $this->query($operation, null, $members, $keys, $decode);
    }

    /**
     * @template T of object
     * @param list<string>|list<MemberIdentifier>|list<FunctionLikeIdentifier> $requests
     * @param list<string> $keys
     * @param Closure(PayloadReader): T $decode
     * @return list<T|null>
     */
    private function query(int $operation, ?int $classLikeKind, array $requests, array $keys, Closure $decode): array
    {
        if ($requests === []) {
            return [];
        }

        $bucketId = ($operation << 4) | ($classLikeKind ?? 0);
        $bucket = $this->cache->values[$bucketId] ?? [];
        $missingRequests = [];
        $missingKeys = [];
        foreach ($keys as $index => $key) {
            if (array_key_exists($key, $bucket)) {
                continue;
            }

            $missingRequests[] = $requests[$index] ?? throw new ProtocolException('A codebase query is misaligned.');
            $missingKeys[] = $key;
        }

        if ($missingRequests !== []) {
            $this->cancellation->throwIfCancelled();
            $payload = Protocol::writeCodebaseQueryRequest(
                $this->cache->generation,
                $operation,
                $missingRequests,
                $classLikeKind,
            );

            $response = $this->host->request($this->parentRequestId, $payload);
            [$reader, $count] = Protocol::readCodebaseQueryResponse(
                $response,
                $this->cache->generation,
                $operation,
                $classLikeKind,
            );

            if ($count !== count($missingKeys)) {
                throw new ProtocolException('A codebase query response contains the wrong result count.');
            }

            foreach ($missingKeys as $key) {
                $bucket[$key] = $reader->readBoolean() ? $decode($reader) : null;
            }

            $reader->finish();
            $this->cache->values[$bucketId] = $bucket;
        }

        $result = [];
        foreach ($keys as $key) {
            /** @var T|null $value */
            $value = $bucket[$key] ?? null;
            $result[] = $value;
        }

        return $result;
    }

    /** @return list<string> */
    private function listNames(int $operation, ?int $classLikeKind = null): array
    {
        $bucketId = ($operation << 4) | ($classLikeKind ?? 0);
        if (array_key_exists($bucketId, $this->cache->lists)) {
            return $this->cache->lists[$bucketId];
        }

        $this->cancellation->throwIfCancelled();
        $payload = Protocol::writeCodebaseListRequest($this->cache->generation, $operation, $classLikeKind);
        $response = $this->host->request($this->parentRequestId, $payload);
        $names = Protocol::readCodebaseListResponse($response, $this->cache->generation, $operation, $classLikeKind);
        $this->cache->lists[$bucketId] = $names;

        return $names;
    }

    /**
     * @param list<string> $names
     * @return list<bool>
     */
    private function checkExistence(int $predicate, array $names): array
    {
        if ($names === []) {
            return [];
        }

        $bucket = $this->cache->existence[$predicate] ?? [];
        $keys = [];
        $missingNames = [];
        $missingKeys = [];
        foreach ($names as $name) {
            if ($name === '') {
                throw new InvalidArgumentException('Codebase metadata names cannot be empty.');
            }

            $key = $predicate === Protocol::EXISTS_CONSTANT ? $name : strtolower($name);
            $keys[] = $key;
            if (!array_key_exists($key, $bucket)) {
                $missingNames[] = $name;
                $missingKeys[] = $key;
            }
        }

        if ($missingNames !== []) {
            $this->cancellation->throwIfCancelled();
            $payload = Protocol::writeCodebaseExistenceRequest($this->cache->generation, $predicate, $missingNames);
            $response = $this->host->request($this->parentRequestId, $payload);
            $results = Protocol::readCodebaseExistenceResponse($response, $this->cache->generation, $predicate);
            if (count($results) !== count($missingKeys)) {
                throw new ProtocolException('A codebase existence response contains the wrong result count.');
            }

            foreach ($missingKeys as $index => $key) {
                $bucket[$key] = $results[$index] ?? throw new ProtocolException(
                    'A codebase existence response is misaligned.',
                );
            }

            $this->cache->existence[$predicate] = $bucket;
        }

        $result = [];
        foreach ($keys as $key) {
            $result[] = $bucket[$key] ?? false;
        }

        return $result;
    }

    /**
     * @param list<MemberIdentifier> $members
     * @return list<bool>
     */
    private function checkMemberExistence(int $predicate, array $members): array
    {
        if ($members === []) {
            return [];
        }

        $bucketId = 256 | $predicate;
        $bucket = $this->cache->existence[$bucketId] ?? [];
        $keys = [];
        $missingMembers = [];
        $missingKeys = [];
        foreach ($members as $member) {
            $memberName = $predicate === Protocol::EXISTS_METHOD ? strtolower($member->member) : $member->member;
            $key = strtolower($member->class) . "\0" . $memberName;
            $keys[] = $key;
            if (!array_key_exists($key, $bucket)) {
                $missingMembers[] = $member;
                $missingKeys[] = $key;
            }
        }

        if ($missingMembers !== []) {
            $this->cancellation->throwIfCancelled();
            $payload = Protocol::writeCodebaseMemberExistenceRequest(
                $this->cache->generation,
                $predicate,
                $missingMembers,
            );

            $response = $this->host->request($this->parentRequestId, $payload);
            $results = Protocol::readCodebaseMemberExistenceResponse($response, $this->cache->generation, $predicate);
            if (count($results) !== count($missingKeys)) {
                throw new ProtocolException('A codebase member-existence response contains the wrong result count.');
            }

            foreach ($missingKeys as $index => $key) {
                $bucket[$key] = $results[$index] ?? throw new ProtocolException(
                    'A codebase member-existence response is misaligned.',
                );
            }

            $this->cache->existence[$bucketId] = $bucket;
        }

        $result = [];
        foreach ($keys as $key) {
            $result[] = $bucket[$key] ?? false;
        }

        return $result;
    }

    /**
     * @param list<string> $names
     * @return list<list<string>>
     */
    private function getRelations(int $relation, array $names): array
    {
        if ($names === []) {
            return [];
        }

        $bucket = $this->cache->relations[$relation] ?? [];
        $keys = [];
        $missingNames = [];
        $missingKeys = [];
        foreach ($names as $name) {
            if ($name === '') {
                throw new InvalidArgumentException('Codebase metadata names cannot be empty.');
            }

            $key = strtolower($name);
            $keys[] = $key;
            if (!array_key_exists($key, $bucket)) {
                $missingNames[] = $name;
                $missingKeys[] = $key;
            }
        }

        if ($missingNames !== []) {
            $this->cancellation->throwIfCancelled();
            $payload = Protocol::writeCodebaseRelationsRequest($this->cache->generation, $relation, $missingNames);
            $response = $this->host->request($this->parentRequestId, $payload);
            $results = Protocol::readCodebaseRelationsResponse($response, $this->cache->generation, $relation);
            if (count($results) !== count($missingKeys)) {
                throw new ProtocolException('A codebase relation response contains the wrong result count.');
            }

            foreach ($missingKeys as $index => $key) {
                $bucket[$key] = $results[$index] ?? throw new ProtocolException(
                    'A codebase relation response is misaligned.',
                );
            }

            $this->cache->relations[$relation] = $bucket;
        }

        $result = [];
        foreach ($keys as $key) {
            $result[] = $bucket[$key] ?? [];
        }

        return $result;
    }
}
