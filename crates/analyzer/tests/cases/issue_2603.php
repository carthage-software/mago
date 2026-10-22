<?php

declare(strict_types=1);

/** @template TId of int|null = int|null */
class Article
{
    /** @var TId */
    protected $id;

    /** @return TId */
    public function getId()
    {
        return $this->id;
    }
}

interface ArticleRepository
{
    /** @return Article<int> */
    public function getById(int $id): Article;
}

function needsInt(int $id): void
{
    echo $id;
}

function needsString(string $id): void
{
    echo $id;
}

function newEntity(): void
{
    $article = new Article();

    // @mago-expect analysis:possibly-null-argument
    needsInt($article->getId());
    // @mago-expect analysis:invalid-argument,possibly-null-argument
    needsString($article->getId());
}

function loadedEntity(ArticleRepository $repository): void
{
    $article = $repository->getById(1);

    needsInt($article->getId());
    // @mago-expect analysis:invalid-argument
    needsString($article->getId());
}

/** @template TId of int|null = int|null */
class ArticleWithConstructor
{
    /** @var TId */
    protected $id;

    public function __construct() {}

    /** @return TId */
    public function getId()
    {
        return $this->id;
    }
}

function newEntityWithConstructor(): void
{
    $article = new ArticleWithConstructor();

    // @mago-expect analysis:possibly-null-argument
    needsInt($article->getId());
    // @mago-expect analysis:invalid-argument,possibly-null-argument
    needsString($article->getId());
}

/** @template T of int|string = int */
class ScalarDefault
{
    /** @var T */
    public $value;
}

/** @template T of int = 42 */
class LiteralDefault
{
    /** @var T */
    public $value;
}

/** @template T of int = 42 */
class LiteralDefaultWithConstructor
{
    /** @var T */
    public $value;

    public function __construct() {}
}

/** @template T of int = self::DEFAULT_VALUE */
class ClassConstantDefault
{
    public const DEFAULT_VALUE = 42;

    /** @var T */
    public $value;
}

/** @param ScalarDefault<int|string> $value */
function needsWiderScalarDefault(mixed $value): void
{
    echo $value->value;
}

/** @param 42 $value */
function needs42(int $value): void
{
    echo $value;
}

/** @param 43 $value */
function needs43(int $value): void
{
    echo $value;
}

function narrowerDefaults(): void
{
    $scalar = new ScalarDefault();
    needsInt($scalar->value);
    // @mago-expect analysis:invalid-argument
    needsString($scalar->value);
    // @mago-expect analysis:invalid-argument
    needsWiderScalarDefault($scalar);

    $literal = new LiteralDefault();
    needs42($literal->value);
    // @mago-expect analysis:invalid-argument
    needs43($literal->value);

    $literalWithConstructor = new LiteralDefaultWithConstructor();
    needs42($literalWithConstructor->value);
    // @mago-expect analysis:invalid-argument
    needs43($literalWithConstructor->value);

    $classConstant = new ClassConstantDefault();
    needs42($classConstant->value);
    // @mago-expect analysis:invalid-argument
    needs43($classConstant->value);
}

/** @template T of int|string = int */
class InferredDefault
{
    /** @param T $value */
    public function __construct(
        public int|string $value,
    ) {}
}

/**
 * @template T
 * @template U = T
 */
class DependentDefault
{
    /** @var U */
    public $second;

    /** @param T $first */
    public function __construct(
        public mixed $first,
    ) {}
}

/**
 * @template T = string
 * @template U = T
 */
class DependentDefaultsWithoutConstructor
{
    /** @var T */
    public $first;

    /** @var U */
    public $second;
}

/** @param DependentDefault<string, string> $value */
function needsDependentStrings(DependentDefault $value): void
{
    needsString($value->first);
    needsString($value->second);
}

function inferredDefaults(string $value): void
{
    $inferred = new InferredDefault($value);
    needsString($inferred->value);
    // @mago-expect analysis:invalid-argument
    needsInt($inferred->value);

    $dependent = new DependentDefault($value);
    needsDependentStrings($dependent);
    needsString($dependent->second);
    // @mago-expect analysis:invalid-argument
    needsInt($dependent->second);

    needsDependentStrings(new DependentDefault('value'));
}

function dependentDefaultsWithoutConstructor(): void
{
    $dependent = new DependentDefaultsWithoutConstructor();
    needsString($dependent->first);
    needsString($dependent->second);
    // @mago-expect analysis:invalid-argument
    needsInt($dependent->first);
    // @mago-expect analysis:invalid-argument
    needsInt($dependent->second);
}

/** @template T of object = stdClass */
class ObjectDefault
{
    /** @var T */
    public $value;
}

function needsStdClass(stdClass $value): void {}

function objectDefault(): void
{
    $object = new ObjectDefault();
    needsStdClass($object->value);
    // @mago-expect analysis:invalid-argument
    needsString($object->value);
}

/** @template T of int|null */
class BoundWithoutDefault
{
    /** @var T */
    public $value;
}

function noDefault(): void
{
    $object = new BoundWithoutDefault();
    needsInt($object->value);
    needsString($object->value);
}

/** @template T */
class DefaultConstructorParent
{
    /** @param T $value */
    public function __construct(public mixed $value) {}
}

/**
 * @template T = string
 * @template U = list<T>
 * @extends DefaultConstructorParent<T>
 */
class InheritedConstructorDefault extends DefaultConstructorParent
{
    /** @var U */
    public $items;
}

/** @param list<string> $value */
function needsStringList(array $value): void {}

/** @param list<int> $value */
function needsIntList(array $value): void {}

/** @param list<mixed> $value */
function needsMixedList(array $value): void {}

function inheritedConstructorDefaults(string $value, mixed $unknown): void
{
    $strings = new InheritedConstructorDefault($value);
    needsStringList($strings->items);
    // @mago-expect analysis:invalid-argument
    needsIntList($strings->items);

    $integers = new InheritedConstructorDefault(42);
    needsIntList($integers->items);
    // @mago-expect analysis:invalid-argument
    needsStringList($integers->items);

    $mixed = new InheritedConstructorDefault($unknown);
    needsMixedList($mixed->items);
    // @mago-expect analysis:mixed-argument
    needsString($mixed->value);
}

/**
 * @template TId of int|null = int|null
 * @extends DefaultConstructorParent<string>
 */
class InheritedScalarDefault extends DefaultConstructorParent
{
    /** @var TId */
    public $id;
}

function inheritedScalarDefault(): void
{
    $article = new InheritedScalarDefault('value');
    // @mago-expect analysis:possibly-null-argument
    needsInt($article->id);
    // @mago-expect analysis:invalid-argument,possibly-null-argument
    needsString($article->id);
}

class DefaultConstantParent
{
    public const DEFAULT_VALUE = 42;
}

/** @template T of int = parent::DEFAULT_VALUE */
class ParentConstantDefault extends DefaultConstantParent
{
    public const DEFAULT_VALUE = 43;

    /** @var T */
    public $value;
}

function parentConstantDefault(): void
{
    $object = new ParentConstantDefault();
    needs42($object->value);
    // @mago-expect analysis:invalid-argument
    needs43($object->value);
}
