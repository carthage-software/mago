<?php

class ReflectionException extends Exception
{
}

interface Reflector extends Stringable
{
    /**
     * @return string
     */
    public function __toString();
}

class Reflection
{
    /**
     * @return list<string>
     */
    public static function getModifierNames(int $modifiers): array
    {
    }
}

final class ReflectionFiber
{
    public function __construct(Fiber $fiber) {}

    public function getFiber(): Fiber
    {
    }

    public function getExecutingFile(): null|string
    {
    }

    public function getExecutingLine(): null|int
    {
    }

    public function getCallable(): callable
    {
    }

    public function getTrace(int $options = DEBUG_BACKTRACE_PROVIDE_OBJECT): array
    {
    }
}

final class ReflectionGenerator
{
    public function __construct(Generator $generator) {}

    /**
     * @pure
     */
    public function getExecutingLine(): int
    {
    }

    /**
     * @pure
     */
    public function getExecutingFile(): string
    {
    }

    /**
     * @pure
     */
    public function getTrace(int $options = DEBUG_BACKTRACE_PROVIDE_OBJECT): array
    {
    }

    /**
     * @pure
     */
    public function getFunction(): ReflectionFunctionAbstract
    {
    }

    /**
     * @pure
     */
    public function getThis(): null|object
    {
    }

    /**
     * @pure
     */
    public function getExecutingGenerator(): Generator
    {
    }

    /**
     * @pure
     */
    public function isClosed(): bool
    {
    }
}

class ReflectionExtension implements Reflector
{
    /**
     * @readonly
     */
    public string $name;

    /**
     * @throws ReflectionException
     */
    public function __construct(string $name) {}

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getVersion(): null|string
    {
    }

    /**
     * @return array<string, ReflectionFunction>
     *
     * @pure
     */
    public function getFunctions(): array
    {
    }

    /**
     * @return array<string, mixed>
     *
     * @pure
     */
    public function getConstants(): array
    {
    }

    /**
     * @return array<string, mixed>
     *
     * @pure
     */
    public function getINIEntries(): array
    {
    }

    /**
     * @return array<class-string, ReflectionClass>
     *
     * @pure
     */
    public function getClasses(): array
    {
    }

    /**
     * @return list<class-string>
     *
     * @pure
     */
    public function getClassNames(): array
    {
    }

    /**
     * @return array<string, string>
     *
     * @pure
     */
    public function getDependencies(): array
    {
    }

    public function info(): void
    {
    }

    /**
     * @pure
     */
    public function isPersistent(): bool
    {
    }

    /**
     * @pure
     */
    public function isTemporary(): bool
    {
    }

    private function __clone(): void
    {
    }
}

abstract class ReflectionType implements Stringable
{
    /**
     * @pure
     */
    public function allowsNull(): bool
    {
    }

    public function __toString(): string
    {
    }

    private function __clone(): void
    {
    }
}

class ReflectionNamedType extends ReflectionType
{
    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function isBuiltin(): bool
    {
    }
}

class ReflectionIntersectionType extends ReflectionType
{
    /**
     * @return list<ReflectionType>
     *
     * @pure
     */
    public function getTypes(): array
    {
    }
}

class ReflectionUnionType extends ReflectionType
{
    /**
     * @return list<ReflectionNamedType|ReflectionIntersectionType>
     *
     * @pure
     */
    public function getTypes(): array
    {
    }
}

class ReflectionZendExtension implements Reflector
{
    /**
     * @readonly
     */
    public string $name;

    /**
     * @throws ReflectionException
     */
    public function __construct(string $name) {}

    /**
     * @param string $name
     * @param bool $return
     *
     * @return ($return is true ? string : null)
     */
    public static function export($name, $return = false)
    {
    }

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getVersion(): string
    {
    }

    /**
     * @pure
     */
    public function getAuthor(): string
    {
    }

    /**
     * @pure
     */
    public function getURL(): string
    {
    }

    /**
     * @pure
     */
    public function getCopyright(): string
    {
    }

    private function __clone(): void
    {
    }
}

/**
 * @template T of object
 */
class ReflectionAttribute implements Reflector
{
    public string $name;

    public const IS_INSTANCEOF = 2;

    private function __construct() {}

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getTarget(): int
    {
    }

    /**
     * @pure
     */
    public function isRepeated(): bool
    {
    }

    /**
     * @pure
     */
    public function getArguments(): array
    {
    }

    /**
     * @return T
     */
    public function newInstance(): object
    {
    }

    private function __clone(): void
    {
    }

    public function __toString(): string
    {
    }

    public static function export()
    {
    }
}

class ReflectionClassConstant implements Reflector
{
    /**
     * @readonly
     */
    public string $name;

    /**
     * @readonly
     */
    public string $class;

    /**
     * @readonly
     */
    public bool $isFinal;

    public const IS_PUBLIC = 1;

    public const IS_PROTECTED = 2;

    public const IS_PRIVATE = 4;

    public const IS_FINAL = 5;

    public function __construct(string|object $class, string $constant) {}

    /**
     * @pure
     */
    public function getDeclaringClass(): ReflectionClass
    {
    }

    /**
     * @pure
     */
    public function getDocComment(): string|false
    {
    }

    /**
     * @pure
     */
    public function getModifiers(): int
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getValue(): mixed
    {
    }

    /**
     * @pure
     */
    public function isPrivate(): bool
    {
    }

    /**
     * @pure
     */
    public function isProtected(): bool
    {
    }

    /**
     * @pure
     */
    public function isPublic(): bool
    {
    }

    public function __toString(): string
    {
    }

    /**
     * @template A
     *
     * @param class-string<A>|null $name
     * @param int $flags
     *
     * @return list<ReflectionAttribute<A>>
     *
     * @pure
     */
    public function getAttributes(null|string $name = null, int $flags = 0): array
    {
    }

    private function __clone(): void
    {
    }

    /**
     * @pure
     */
    public function isEnumCase(): bool
    {
    }

    /**
     * @pure
     */
    public function isFinal(): bool
    {
    }

    /**
     * @pure
     */
    public function hasType(): bool
    {
    }

    /**
     * @pure
     */
    public function getType(): null|ReflectionType
    {
    }

    /**
     * @pure
     */
    public function isDeprecated(): bool
    {
    }
}

enum PropertyHookType: string
{
    case Get = 'get';
    case Set = 'set';
}

class ReflectionProperty implements Reflector
{
    public const IS_ABSTRACT = 64;

    public const IS_VIRTUAL = 512;

    public const IS_STATIC = 16;

    public const IS_PUBLIC = 1;

    public const IS_PROTECTED = 2;

    public const IS_PRIVATE = 4;

    public const IS_READONLY = 128;

    public const IS_PROTECTED_SET = 2048;

    public const IS_PRIVATE_SET = 4096;

    public const IS_FINAL = 32;

    /**
     * @readonly
     */
    public string $name;

    /**
     * @readonly
     */
    public string $class;

    /**
     * @template T of object
     *
     * @param class-string<T>|object $class
     * @param key-of<properties-of<T>>|string $property
     *
     * @throws ReflectionException
     */
    public function __construct(object|string $class, string $property) {}

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getValue(null|object $object = null): mixed
    {
    }

    public function setValue(mixed $objectOrValue, mixed $value): void
    {
    }

    /**
     * @pure
     */
    public function isPublic(): bool
    {
    }

    /**
     * @pure
     */
    public function isPrivate(): bool
    {
    }

    /**
     * @pure
     */
    public function isProtected(): bool
    {
    }

    /**
     * @pure
     */
    public function isStatic(): bool
    {
    }

    /**
     * @pure
     */
    public function isDefault(): bool
    {
    }

    /**
     * @pure
     */
    public function getModifiers(): int
    {
    }

    /**
     * @return ReflectionClass
     *
     * @pure
     */
    public function getDeclaringClass(): ReflectionClass
    {
    }

    /**
     * @pure
     */
    public function getDocComment(): string|false
    {
    }

    /**
     * @deprecated
     *
     * @pure
     */
    public function setAccessible(bool $accessible): void
    {
    }

    /**
     * @pure
     */
    public function getType(): ReflectionNamedType|ReflectionUnionType|ReflectionIntersectionType|null
    {
    }

    /**
     * @pure
     */
    public function hasType(): bool
    {
    }

    /**
     * @pure
     */
    public function isInitialized(null|object $object = null): bool
    {
    }

    /**
     * @pure
     */
    public function isPromoted(): bool
    {
    }

    private function __clone(): void
    {
    }

    public function hasDefaultValue(): bool
    {
    }

    /**
     *
     * @pure
     */
    public function getDefaultValue(): mixed
    {
    }

    /**
     * @template A
     *
     * @param class-string<A>|null $name
     * @param int $flags
     *
     * @return list<ReflectionAttribute<A>>
     *
     * @pure
     */
    public function getAttributes(null|string $name = null, int $flags = 0): array
    {
    }

    public function isReadOnly(): bool
    {
    }

    public function getRawValue(object $object): mixed
    {
    }

    public function setRawValue(object $object, mixed $value): void
    {
    }

    public function isAbstract(): bool
    {
    }

    public function isVirtual(): bool
    {
    }

    public function getSettableType(): null|ReflectionType
    {
    }

    public function hasHooks(): bool
    {
    }

    public function getHooks(): array
    {
    }

    public function hasHook(PropertyHookType $type): bool
    {
    }

    public function getHook(PropertyHookType $type): null|ReflectionMethod
    {
    }

    public function isPrivateSet(): bool
    {
    }

    public function isProtectedSet(): bool
    {
    }

    public function setRawValueWithoutLazyInitialization(object $object, mixed $value): void
    {
    }

    public function skipLazyInitialization(object $object): void
    {
    }

    public function isDynamic(): bool
    {
    }

    public function isFinal(): bool
    {
    }

    public function isLazy(object $object): bool
    {
    }
}

final class ReflectionReference
{
    private function __construct() {}

    public static function fromArrayElement(array $array, string|int $key): null|ReflectionReference
    {
    }

    /**
     * @pure
     */
    public function getId(): string
    {
    }

    private function __clone(): void
    {
    }
}

final class ReflectionConstant implements Reflector
{
    public string $name;

    public function __construct(string $name) {}

    public function getName(): string
    {
    }

    public function getNamespaceName(): string
    {
    }

    public function getShortName(): string
    {
    }

    public function getValue(): mixed
    {
    }

    public function isDeprecated(): bool
    {
    }

    public function __toString(): string
    {
    }
}

class ReflectionParameter implements Reflector
{
    /**
     * @readonly
     */
    public string $name;

    /**
     * @throws ReflectionException
     */
    public function __construct($function, string|int $param) {}

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function isPassedByReference(): bool
    {
    }

    public function canBePassedByValue(): bool
    {
    }

    /**
     * @pure
     */
    public function getDeclaringFunction(): ReflectionFunctionAbstract
    {
    }

    /**
     * @pure
     */
    public function getDeclaringClass(): null|ReflectionClass
    {
    }

    #[Deprecated(
        since: '8.0',
        reason: 'Use ReflectionParameter::getType() and the ReflectionType APIs should be used instead.',
    )]
    public function getClass(): null|ReflectionClass
    {
    }

    public function hasType(): bool
    {
    }

    /**
     * @pure
     */
    public function getType(): ReflectionNamedType|ReflectionUnionType|ReflectionIntersectionType|null
    {
    }

    #[Deprecated(
        since: '8.0',
        reason: 'Use ReflectionParameter::getType() and the ReflectionType APIs should be used instead.',
    )]
    public function isArray(): bool
    {
    }

    #[Deprecated(
        since: '8.0',
        reason: 'Use ReflectionParameter::getType() and the ReflectionType APIs should be used instead.',
    )]
    public function isCallable(): bool
    {
    }

    public function allowsNull(): bool
    {
    }

    /**
     * @pure
     */
    public function getPosition(): int
    {
    }

    /**
     * @pure
     */
    public function isOptional(): bool
    {
    }

    /**
     * @pure
     */
    public function isDefaultValueAvailable(): bool
    {
    }

    /**
     * @throws ReflectionException
     *
     * @pure
     */
    public function getDefaultValue(): mixed
    {
    }

    /**
     * @pure
     */
    public function isDefaultValueConstant(): bool
    {
    }

    /**
     * @throws ReflectionException
     *
     * @pure
     */
    public function getDefaultValueConstantName(): null|string
    {
    }

    /**
     * @pure
     */
    public function isVariadic(): bool
    {
    }

    /**
     * @pure
     */
    public function isPromoted(): bool
    {
    }

    /**
     * @template A
     *
     * @param class-string<A>|null $name
     * @param int $flags
     *
     * @return list<ReflectionAttribute<A>>
     *
     * @pure
     */
    public function getAttributes(null|string $name = null, int $flags = 0): array
    {
    }

    private function __clone(): void
    {
    }
}

abstract class ReflectionFunctionAbstract implements Reflector
{
    /**
     * @readonly
     */
    public string $name;

    private function __clone(): void
    {
    }

    public function inNamespace(): bool
    {
    }

    /**
     * @pure
     */
    public function isClosure(): bool
    {
    }

    /**
     * @pure
     */
    public function isDeprecated(): bool
    {
    }

    /**
     * @pure
     */
    public function isInternal(): bool
    {
    }

    /**
     * @pure
     */
    public function isUserDefined(): bool
    {
    }

    /**
     * @pure
     */
    public function isGenerator(): bool
    {
    }

    /**
     * @pure
     */
    public function isVariadic(): bool
    {
    }

    /**
     * @pure
     */
    public function getClosureThis(): null|object
    {
    }

    /**
     * @pure
     */
    public function getClosureScopeClass(): null|ReflectionClass
    {
    }

    /**
     * @pure
     */
    public function getClosureCalledClass(): null|ReflectionClass
    {
    }

    /**
     * @pure
     */
    public function getDocComment(): string|false
    {
    }

    /**
     * @pure
     */
    public function getEndLine(): int|false
    {
    }

    /**
     * @pure
     */
    public function getExtension(): null|ReflectionExtension
    {
    }

    /**
     * @pure
     */
    public function getExtensionName(): string|false
    {
    }

    /**
     * @pure
     */
    public function getFileName(): string|false
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function getNamespaceName(): string
    {
    }

    /**
     * @pure
     */
    public function getNumberOfParameters(): int
    {
    }

    /**
     * @pure
     */
    public function getNumberOfRequiredParameters(): int
    {
    }

    /**
     * @return ReflectionParameter[]
     *
     * @pure
     */
    public function getParameters(): array
    {
    }

    /**
     * @pure
     */
    public function getReturnType(): ReflectionNamedType|ReflectionUnionType|ReflectionIntersectionType|null
    {
    }

    /**
     * @pure
     */
    public function getShortName(): string
    {
    }

    /**
     * @pure
     */
    public function getStartLine(): int|false
    {
    }

    /**
     * @pure
     */
    public function getStaticVariables(): array
    {
    }

    /**
     * @pure
     */
    public function returnsReference(): bool
    {
    }

    /**
     * @pure
     */
    public function hasReturnType(): bool
    {
    }

    /**
     * @template A
     *
     * @param class-string<A>|null $name
     *
     * @return list<ReflectionAttribute<A>>
     *
     * @pure
     */
    public function getAttributes(null|string $name = null, int $flags = 0): array
    {
    }

    /**
     * @pure
     */
    public function getClosureUsedVariables(): array
    {
    }

    /**
     * @pure
     */
    public function hasTentativeReturnType(): bool
    {
    }

    /**
     * @pure
     */
    public function getTentativeReturnType(): null|ReflectionType
    {
    }

    /**
     * @pure
     */
    public function isStatic(): bool
    {
    }

    public function __toString()
    {
    }
}

class ReflectionFunction extends ReflectionFunctionAbstract
{
    /**
     * @readonly
     * @var string
     */
    public $name;

    public const IS_DEPRECATED = 2048;

    /**
     * @throws ReflectionException
     */
    public function __construct(Closure|string $function) {}

    public function __toString(): string
    {
    }

    /**
     * @deprecated
     * @pure
     */
    public function isDisabled(): bool
    {
    }

    public function invoke(mixed ...$args): mixed
    {
    }

    public function invokeArgs(array $args): mixed
    {
    }

    /**
     * @return Closure|null
     *
     * @pure
     */
    public function getClosure(): Closure
    {
    }

    public function isAnonymous(): bool
    {
    }
}

class ReflectionMethod extends ReflectionFunctionAbstract
{
    /**
     * @readonly
     */
    public string $name;

    /**
     * @readonly
     */
    public string $class;

    public const IS_STATIC = 16;

    public const IS_PUBLIC = 1;

    public const IS_PROTECTED = 2;

    public const IS_PRIVATE = 4;

    public const IS_ABSTRACT = 64;

    public const IS_FINAL = 32;

    /**
     * @throws ReflectionException
     */
    public function __construct(object|string $objectOrMethod, string|null $method = null) {}

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function isPublic(): bool
    {
    }

    /**
     * @pure
     */
    public function isPrivate(): bool
    {
    }

    /**
     * @pure
     */
    public function isProtected(): bool
    {
    }

    /**
     * @pure
     */
    public function isAbstract(): bool
    {
    }

    /**
     * @pure
     */
    public function isFinal(): bool
    {
    }

    /**
     * @pure
     */
    public function isStatic(): bool
    {
    }

    /**
     * @pure
     */
    public function isConstructor(): bool
    {
    }

    /**
     * @pure
     */
    public function isDestructor(): bool
    {
    }

    /**
     * @throws ValueError
     * @throws ReflectionException
     */
    public function getClosure(object|null $object = null): Closure
    {
    }

    /**
     * @pure
     */
    public function getModifiers(): int
    {
    }

    /**
     * @param object|null $object
     *
     * @throws ReflectionException
     */
    public function invoke($object, ...$args)
    {
    }

    /**
     * @throws ReflectionException
     */
    public function invokeArgs(null|object $object, array $args): mixed
    {
    }

    /**
     * @pure
     */
    public function getDeclaringClass(): ReflectionClass
    {
    }

    /**
     * @throws ReflectionException
     *
     * @pure
     */
    public function getPrototype(): ReflectionMethod
    {
    }

    /**
     * @deprecated
     *
     * @pure
     */
    public function setAccessible(bool $accessible): void
    {
    }

    public function hasPrototype(): bool
    {
    }

    public static function createFromMethodName(string $method): static
    {
    }
}

/**
 * @template T of object
 */
class ReflectionClass implements Reflector
{
    /**
     * @var class-string<T>
     */
    public string $name;

    public const IS_IMPLICIT_ABSTRACT = 16;

    public const IS_EXPLICIT_ABSTRACT = 64;

    public const IS_FINAL = 32;

    public const IS_READONLY = 65536;

    public const int SKIP_INITIALIZATION_ON_SERIALIZE = 0;

    public const int SKIP_DESTRUCTOR = 0;

    /**
     * @param class-string<T>|T $objectOrClass
     *
     * @throws ReflectionException
     */
    public function __construct(object|string $objectOrClass) {}

    public function __toString(): string
    {
    }

    /**
     * @pure
     */
    public function getName(): string
    {
    }

    /**
     * @pure
     */
    public function isInternal(): bool
    {
    }

    /**
     * @pure
     */
    public function isUserDefined(): bool
    {
    }

    /**
     * @pure
     */
    public function isInstantiable(): bool
    {
    }

    /**
     * @pure
     */
    public function isCloneable(): bool
    {
    }

    /**
     * @pure
     */
    public function getFileName(): string|false
    {
    }

    /**
     * @pure
     */
    public function getStartLine(): int|false
    {
    }

    /**
     * @pure
     */
    public function getEndLine(): int|false
    {
    }

    /**
     * @pure
     */
    public function getDocComment(): string|false
    {
    }

    /**
     * @pure
     */
    public function getConstructor(): null|ReflectionMethod
    {
    }

    /**
     * @pure
     */
    public function hasMethod(string $name): bool
    {
    }

    /**
     * @throws ReflectionException
     *
     * @pure
     */
    public function getMethod(string $name): ReflectionMethod
    {
    }

    /**
     * @return list<ReflectionMethod>
     *
     * @pure
     */
    public function getMethods(int|null $filter = null): array
    {
    }

    /**
     * @pure
     */
    public function hasProperty(string $name): bool
    {
    }

    /**
     * @throws ReflectionException
     *
     * @pure
     */
    public function getProperty(string $name): ReflectionProperty
    {
    }

    /**
     * @return list<ReflectionProperty>
     *
     * @pure
     */
    public function getProperties(null|int $filter = null): array
    {
    }

    /**
     * @pure
     */
    public function getReflectionConstant(string $name): ReflectionClassConstant|false
    {
    }

    /**
     * @pure
     */
    public function getReflectionConstants(null|int $filter = null): array
    {
    }

    /**
     * @pure
     */
    public function hasConstant(string $name): bool
    {
    }

    /**
     * @return array<string, mixed>
     *
     * @pure
     */
    public function getConstants(null|int $filter = null): array
    {
    }

    /**
     * @pure
     */
    public function getConstant(string $name): mixed
    {
    }

    /**
     * @return array<interface-string, ReflectionClass>
     *
     * @pure
     */
    public function getInterfaces(): array
    {
    }

    /**
     * @return list<interface-string>
     */
    public function getInterfaceNames(): array
    {
    }

    /**
     * @pure
     */
    public function isAnonymous(): bool
    {
    }

    /**
     * @pure
     */
    public function isInterface(): bool
    {
    }

    /**
     * @return array<trait-string, ReflectionClass>
     *
     * @pure
     */
    public function getTraits(): array
    {
    }

    /**
     * @return list<trait-string>
     *
     * @pure
     */
    public function getTraitNames(): array
    {
    }

    /**
     * @return array<string, string>
     *
     * @pure
     */
    public function getTraitAliases(): array
    {
    }

    /**
     * @pure
     */
    public function isTrait(): bool
    {
    }

    /**
     * @pure
     */
    public function isAbstract(): bool
    {
    }

    /**
     * @pure
     */
    public function isFinal(): bool
    {
    }

    /**
     * @pure
     */
    public function isReadOnly(): bool
    {
    }

    /**
     * @pure
     */
    public function getModifiers(): int
    {
    }

    /**
     * @pure
     */
    public function isInstance(object $object): bool
    {
    }

    /**
     * @return T
     *
     * @throws ReflectionException
     */
    public function newInstance(...$args)
    {
    }

    /**
     * @return T
     *
     * @throws ReflectionException
     */
    public function newInstanceWithoutConstructor(): object
    {
    }

    /**
     * @param array $args
     *
     * @return T|null
     *
     * @throws ReflectionException
     */
    public function newInstanceArgs(array $args = []): null|object
    {
    }

    /**
     * @return ReflectionClass|false
     *
     * @pure
     */
    public function getParentClass(): ReflectionClass|false
    {
    }

    /**
     * @param string|ReflectionClass $class
     *
     * @pure
     */
    public function isSubclassOf(ReflectionClass|string $class): bool
    {
    }

    /**
     * @return array
     *
     * @pure
     */
    public function getStaticProperties(): array
    {
    }

    /**
     * @pure
     */
    public function getStaticPropertyValue(string $name, mixed $default): mixed
    {
    }

    public function setStaticPropertyValue(string $name, mixed $value): void
    {
    }

    /**
     * @pure
     */
    public function getDefaultProperties(): array
    {
    }

    /**
     * @pure
     */
    public function isIterateable(): bool
    {
    }

    /**
     * @pure
     */
    public function isIterable(): bool
    {
    }

    /**
     * @pure
     */
    public function implementsInterface(ReflectionClass|string $interface): bool
    {
    }

    /**
     * @pure
     */
    public function getExtension(): null|ReflectionExtension
    {
    }

    /**
     * @pure
     */
    public function getExtensionName(): string|false
    {
    }

    /**
     * @pure
     */
    public function inNamespace(): bool
    {
    }

    /**
     * @pure
     */
    public function getNamespaceName(): string
    {
    }

    /**
     * @pure
     */
    public function getShortName(): string
    {
    }

    /**
     * @template A
     *
     * @param class-string<A>|null $name
     *
     * @return list<ReflectionAttribute<A>>
     *
     * @pure
     */
    public function getAttributes(null|string $name = null, int $flags = 0): array
    {
    }

    private function __clone(): void
    {
    }

    public function isEnum(): bool
    {
    }

    public function newLazyGhost(callable $initializer, int $options = 0): object
    {
    }

    /**
     * @return T
     */
    public function newLazyProxy(callable $factory, int $options = 0): object
    {
    }

    public function resetAsLazyGhost(object $object, callable $initializer, int $options = 0): void
    {
    }

    public function resetAsLazyProxy(object $object, callable $factory, int $options = 0): void
    {
    }

    public function initializeLazyObject(object $object): object
    {
    }

    public function isUninitializedLazyObject(object $object): bool
    {
    }

    public function markLazyObjectAsInitialized(object $object): object
    {
    }

    public function getLazyInitializer(object $object): null|callable
    {
    }
}

class ReflectionObject extends ReflectionClass
{
    public function __construct(object $object) {}
}

class ReflectionEnum extends ReflectionClass
{
    public function __construct(object|string $objectOrClass) {}

    public function hasCase(string $name): bool
    {
    }

    /**
     * @return list<ReflectionEnumUnitCase|ReflectionEnumBackedCase>
     */
    public function getCases(): array
    {
    }

    /**
     * @throws ReflectionException
     */
    public function getCase(string $name): ReflectionEnumUnitCase
    {
    }

    /**
     * @return bool
     */
    public function isBacked(): bool
    {
    }

    /**
     * @return ReflectionNamedType|null
     */
    public function getBackingType()
    {
    }
}

class ReflectionEnumUnitCase extends ReflectionClassConstant
{
    public function __construct(object|string $class, string $constant) {}

    /**
     * @pure
     */
    public function getValue(): UnitEnum
    {
    }

    /**
     * @pure
     */
    public function getEnum(): ReflectionEnum
    {
    }
}

class ReflectionEnumBackedCase extends ReflectionEnumUnitCase
{
    public function __construct(object|string $class, string $constant) {}

    /**
     * @pure
     */
    public function getBackingValue(): int|string
    {
    }
}
