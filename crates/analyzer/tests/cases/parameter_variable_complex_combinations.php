<?php

declare(strict_types=1);

interface ComplexMarker {}

final class ComplexFoo implements ComplexMarker
{
    public string $name = '';
}

final class ComplexBar implements ComplexMarker {}

final class ComplexContainer
{
    public ComplexFoo $item;

    public function __construct()
    {
        $this->item = new ComplexFoo();
    }
}

interface ComplexContract {}

enum ComplexState: string
{
    case Ready = 'ready';
}

trait ComplexBehavior {}

/** @template-covariant T */
interface ComplexGenericCollection extends Countable
{
    /** @return T */
    public function current(): mixed;
}

/**
 * @type ComplexRegistry = array{
 *   foo: ComplexFoo,
 *   bar: ComplexBar,
 *   integer: 42,
 *   string: 'complex',
 *   truth: true,
 *   nullable: ComplexFoo|null,
 *   union: ComplexFoo|ComplexBar,
 *   intersection: ComplexFoo&ComplexMarker,
 *   list: list<ComplexFoo>,
 *   shape: array{item: ComplexFoo},
 *   class: class-string<ComplexFoo>,
 *   callable: callable(): ComplexFoo,
 *   positive: positive-int,
 *   range: int<1, 10>,
 *   numeric-string: numeric-string,
 *   non-empty-string: non-empty-string,
 *   lowercase-string: lowercase-string,
 *   non-empty-array: non-empty-array<string, ComplexFoo>,
 *   iterable: iterable<string, ComplexFoo>,
 *   resource: resource,
 *   open-resource: open-resource,
 *   closed-resource: closed-resource,
 * }
 * @type ComplexObjects = array{foo: ComplexFoo, bar: ComplexBar}
 * @type ComplexNumericObjects = array{0: ComplexFoo, -1: ComplexBar}
 * @type ComplexInterfaces = array{contract: ComplexContract}
 * @type ComplexEnums = array{state: ComplexState}
 * @type ComplexTraits = array{behavior: ComplexBehavior}
 */
interface ParameterVariableComplexCombinations
{
    /** @return ComplexRegistry[$key] */
    public function get(string $key): mixed;

    /** @return $value */
    public function identity(mixed $value): mixed;

    /** @param list<$prototype> $values */
    public function sameList(mixed $prototype, array $values): void;

    /** @return list{$first, $second} */
    public function tuple(mixed $first, mixed $second): array;

    /** @return $maps[$group][$key] */
    public function nested(array $maps, string $group, string $key): mixed;

    /** @return key-of<$maps[$group]> */
    public function nestedKey(array $maps, string $group): int|string;

    /** @return value-of<$maps[$group]> */
    public function nestedValue(array $maps, string $group): mixed;

    /** @return $map[key-of<$map>] */
    public function allValues(array $map): mixed;

    /** @return value-of<array{item: $value}> */
    public function wrappedValue(mixed $value): mixed;

    /** @return value-of<$state> */
    public function stateValue(ComplexState $state): string;

    /** @return new<value-of<$classes>> */
    public function makeFromMap(array $classes): object;

    /** @return key-of<properties-of<$object>> */
    public function propertyName(object $object): string;

    /** @return value-of<properties-of<$object>> */
    public function propertyValue(object $object): mixed;

    /** @return value-of<properties-of<new<$class>>> */
    public function constructedPropertyValue(string $class): mixed;

    /** @return properties-of<ComplexObjects[$key]> */
    public function propertiesByKey(string $key): array;

    /** @return value-of<properties-of<ComplexObjects[$key]>> */
    public function propertyValueByKey(string $key): mixed;

    /** @return class-string<ComplexObjects[$key]> */
    public function className(string $key): string;

    /** @return interface-string<ComplexInterfaces[$key]> */
    public function interfaceName(string $key): string;

    /** @return enum-string<ComplexEnums[$key]> */
    public function enumName(string $key): string;

    /** @return trait-string<ComplexTraits[$key]> */
    public function traitName(string $key): string;

    /** @return class-string<($key is 'foo' ? ComplexFoo : ComplexBar)> */
    public function conditionalClassName(string $key): string;

    /** @return new<($key is 'foo' ? class-string<ComplexFoo> : class-string<ComplexBar>)> */
    public function conditionalCreate(string $key): object;

    /** @return class-string<value-of<$objects>> */
    public function classNameFromObjects(array $objects): string;

    /** @return new<class-string<value-of<$objects>>> */
    public function createFromObjects(array $objects): object;

    /** @return array{required: ComplexRegistry[$key], optional?: ComplexRegistry[$key], ...<string, ComplexRegistry[$key]>} */
    public function openShape(string $key): array;

    /** @return ComplexGenericCollection<ComplexObjects[$key]>&Countable */
    public function genericIntersection(string $key): ComplexGenericCollection;

    /** @return iterable<ComplexObjects[$key]>&Countable */
    public function iterableIntersection(string $key): iterable;

    /** @return ComplexObjects[$key]|null */
    public function nullableObject(string $key): ?object;

    /** @return ComplexObjects[$key] */
    public function getObject(string $key): object;

    /** @return ComplexNumericObjects[$key] */
    public function getNumericObject(int $key): object;

    /** @return ($first is true ? ($second is true ? ComplexFoo : ComplexBar) : ComplexContainer) */
    public function choose(bool $first, bool $second): object;

    /** @return ($value is not ComplexFoo ? ComplexBar : ComplexFoo) */
    public function chooseNegated(object $value): object;

    /** @return ($value is $target ? ComplexFoo : ComplexBar) */
    public function chooseAgainst(object $value, object $target): object;

    /**
     * @param array{foo: class-string<ComplexFoo>, bar: class-string<ComplexBar>} $classes
     * @param key-of<$classes> $key
     * @return new<$classes[$key]>
     */
    public function makeDefault(
        array $classes = ['foo' => ComplexFoo::class, 'bar' => ComplexBar::class],
        string $key = 'foo',
    ): object;

    /**
     * @template TMap of array<array-key, object>
     * @param TMap $map
     * @param key-of<TMap> $key
     * @return TMap[$key]
     */
    public function templated(array $map, int|string $key): object;

    /**
     * @template TMap of array<array-key, object>
     * @param TMap $map
     * @param key-of<TMap> $key
     * @param TMap[$key] $value
     */
    public function templatedSet(array $map, int|string $key, object $value): void;

    /**
     * @template TFallback of object
     * @param TFallback $fallback
     * @return ($key is key-of<ComplexObjects> ? ComplexObjects[$key] : TFallback)
     */
    public function templatedFallback(string $key, object $fallback): object;

    /**
     * @template TFallback of object
     * @param TFallback $fallback
     * @param ($key is key-of<ComplexObjects> ? ComplexObjects[$key] : TFallback) $value
     */
    public function templatedConditionalSet(string $key, object $fallback, object $value): void;

    /** @param ComplexObjects[$key] ...$values */
    public function consume(string $key, object ...$values): void;

    /** @param callable(ComplexObjects[$key]): void $consumer */
    public function withConsumer(string $key, callable $consumer): void;

    /** @param callable(): (ComplexObjects[$key]) $factory */
    public function withFactory(string $key, callable $factory): void;

    /** @return callable(string $inner): (ComplexObjects[$outer]) */
    public function callableCapture(string $outer): callable;

    /** @param ComplexObjects[$key] $value */
    public function store(string $key, object $value): void;

    /** @param-out ComplexObjects[$key] $value */
    public function load(string $key, mixed &$value): void;

    /** @psalm-assert-if-true ComplexObjects[$key] $value */
    public function is(string $key, object $value): bool;
}

interface ParameterVariableComplexInvoker
{
    /** @return !ParameterVariableComplexCombinations::ComplexObjects[$key] */
    public function __invoke(string $key): object;
}

abstract class ParameterVariableComplexStatic
{
    /** @return $value */
    abstract public static function identity(mixed $value): mixed;

    /** @return !ParameterVariableComplexCombinations::ComplexObjects[$key] */
    abstract public static function get(string $key): mixed;
}

final class ParameterVariableComplexConstructor
{
    /** @param !ParameterVariableComplexCombinations::ComplexObjects[$key] $value */
    public function __construct(string $key, object $value) {}
}

function exercise_parameter_variable_heterogeneous_values(ParameterVariableComplexCombinations $types): void
{
    take_complex_foo($types->get('foo'));
    take_complex_bar($types->get('bar'));
    take_complex_integer($types->get('integer'));
    take_complex_string($types->get('string'));
    take_complex_truth($types->get('truth'));
    take_complex_nullable($types->get('nullable'));
    take_complex_union($types->get('union'));
    take_complex_marker($types->get('intersection'));
    take_complex_foo_list($types->get('list'));
    take_complex_foo($types->get('shape')['item']);
    take_complex_foo_class($types->get('class'));
    take_complex_foo($types->get('callable')());
    take_complex_positive($types->get('positive'));
    take_complex_range($types->get('range'));
    take_complex_numeric_string($types->get('numeric-string'));
    take_complex_non_empty_string($types->get('non-empty-string'));
    take_complex_lowercase_string($types->get('lowercase-string'));
    take_complex_non_empty_array($types->get('non-empty-array'));
    take_complex_iterable($types->get('iterable'));
    take_complex_resource($types->get('resource'));
    take_complex_open_resource($types->get('open-resource'));
    take_complex_closed_resource($types->get('closed-resource'));

    take_complex_foo($types->identity(new ComplexFoo()));
    $types->sameList(new ComplexFoo(), [new ComplexFoo()]);
    // @mago-expect analysis:possibly-invalid-argument
    $types->sameList(new ComplexFoo(), [new ComplexBar()]);

    take_complex_tuple($types->tuple(new ComplexFoo(), new ComplexBar()));
    take_complex_foo(ParameterVariableComplexStatic::identity(new ComplexFoo()));
    take_complex_foo(ParameterVariableComplexStatic::get('foo'));
}

function exercise_parameter_variable_nested_utilities(
    ParameterVariableComplexCombinations $types,
    bool $selectFoo,
): void {
    $maps = ['services' => ['foo' => new ComplexFoo()]];
    take_complex_foo($types->nested($maps, 'services', 'foo'));
    take_complex_foo_key($types->nestedKey($maps, 'services'));
    take_complex_foo($types->nestedValue($maps, 'services'));
    take_complex_foo($types->allValues(['foo' => new ComplexFoo()]));
    take_complex_foo($types->wrappedValue(new ComplexFoo()));
    take_complex_ready_value($types->stateValue(ComplexState::Ready));

    take_complex_foo($types->makeFromMap(['foo' => ComplexFoo::class]));
    take_complex_item_key($types->propertyName(new ComplexContainer()));
    take_complex_foo($types->propertyValue(new ComplexContainer()));
    take_complex_foo($types->constructedPropertyValue(ComplexContainer::class));
    take_complex_foo_properties($types->propertiesByKey('foo'));
    take_complex_string_value($types->propertyValueByKey('foo'));

    take_complex_foo_class($types->className('foo'));
    take_complex_contract_class($types->interfaceName('contract'));
    take_complex_state_class($types->enumName('state'));
    take_complex_behavior_class($types->traitName('behavior'));
    take_complex_foo_class($types->conditionalClassName('foo'));
    take_complex_bar_class($types->conditionalClassName('bar'));
    take_complex_foo($types->conditionalCreate('foo'));
    take_complex_bar($types->conditionalCreate('bar'));
    take_complex_foo_class($types->classNameFromObjects(['foo' => new ComplexFoo()]));
    take_complex_foo($types->createFromObjects(['foo' => new ComplexFoo()]));

    $shape = $types->openShape('foo');
    take_complex_foo($shape['required']);
    take_complex_foo($shape['anything']);

    take_complex_generic_collection($types->genericIntersection('foo'));
    take_complex_iterable_collection($types->iterableIntersection('foo'));

    take_complex_nullable($types->nullableObject('foo'));
    take_complex_foo($types->getObject('foo'));
    take_complex_foo($types->getNumericObject(0));
    take_complex_bar($types->getNumericObject(-1));
    take_complex_foo($types->choose(true, true));
    take_complex_bar($types->choose(true, false));
    take_complex_container($types->choose(false, true));
    take_complex_foo($types->chooseNegated(new ComplexFoo()));
    take_complex_bar($types->chooseNegated(new ComplexBar()));
    take_complex_foo($types->chooseAgainst(new ComplexFoo(), new ComplexFoo()));
    take_complex_bar($types->chooseAgainst(new ComplexFoo(), new ComplexBar()));

    $dynamicKey = $selectFoo ? 'foo' : 'bar';
    take_complex_union($types->getObject($dynamicKey));
    take_complex_object_class($types->className($dynamicKey));

    take_complex_foo($types->makeDefault());
    take_complex_bar($types->makeDefault(key: 'bar'));
    // @mago-expect analysis:invalid-argument
    $types->makeDefault(key: 'invalid');
}

function exercise_parameter_variable_templates_and_parameters(ParameterVariableComplexCombinations $types): void
{
    $map = ['foo' => new ComplexFoo(), 'bar' => new ComplexBar()];
    take_complex_foo($types->templated($map, 'foo'));
    $types->templatedSet($map, 'foo', new ComplexFoo());
    // @mago-expect analysis:invalid-argument
    $types->templatedSet($map, 'foo', new ComplexBar());

    take_complex_foo($types->templatedFallback('foo', new ComplexContainer()));
    take_complex_container($types->templatedFallback('other', new ComplexContainer()));
    $types->templatedConditionalSet('foo', new ComplexContainer(), new ComplexFoo());
    // @mago-expect analysis:invalid-argument
    $types->templatedConditionalSet('foo', new ComplexContainer(), new ComplexBar());
    $types->templatedConditionalSet('other', new ComplexContainer(), new ComplexContainer());

    $types->consume('foo', new ComplexFoo(), new ComplexFoo());
    // @mago-expect analysis:invalid-argument
    $types->consume('foo', new ComplexFoo(), new ComplexBar());

    $types->withConsumer('foo', function ($value): void {
        take_complex_foo($value);
    });
    $types->withConsumer(consumer: function ($value): void {
        take_complex_foo($value);
    }, key: 'foo');

    $types->withFactory('foo', static fn(): ComplexFoo => new ComplexFoo());
    // @mago-expect analysis:invalid-argument
    $types->withFactory('foo', static fn(): ComplexBar => new ComplexBar());

    new ParameterVariableComplexConstructor('foo', new ComplexFoo());
    // @mago-expect analysis:invalid-argument
    new ParameterVariableComplexConstructor('foo', new ComplexBar());
}

function exercise_parameter_variable_callable_scopes(ParameterVariableComplexCombinations $types): void
{
    $captured = $types->callableCapture('foo');
    take_complex_foo($captured('ignored'));
}

function exercise_parameter_variable_additional_invocation_forms(ParameterVariableComplexInvoker $invoker): void
{
    take_complex_foo($invoker('foo'));

    $firstClassInvoker = $invoker(...);
    take_complex_foo($firstClassInvoker('foo'));
    take_complex_foo($firstClassInvoker(key: 'foo'));

    $firstClassStatic = ParameterVariableComplexStatic::get(...);
    take_complex_foo($firstClassStatic('foo'));
    take_complex_foo($firstClassStatic(key: 'foo'));

    $arrayStatic = [ParameterVariableComplexStatic::class, 'get'];
    take_complex_foo($arrayStatic('foo'));
    take_complex_foo($arrayStatic(key: 'foo'));
}

function exercise_parameter_variable_assertions(
    ParameterVariableComplexCombinations $types,
    ComplexFoo|ComplexBar $value,
): void {
    $loaded = null;
    $types->load('foo', $loaded);
    take_complex_foo($loaded);

    if ($types->is('foo', $value)) {
        take_complex_foo($value);
    }
}

function take_complex_foo(ComplexFoo $_): void {}

function take_complex_bar(ComplexBar $_): void {}

function take_complex_container(ComplexContainer $_): void {}

function take_complex_marker(ComplexMarker $_): void {}

/** @param 42 $_ */
function take_complex_integer(int $_): void {}

/** @param 'complex' $_ */
function take_complex_string(string $_): void {}

/** @param true $_ */
function take_complex_truth(bool $_): void {}

/** @param ComplexFoo|null $_ */
function take_complex_nullable(?ComplexFoo $_): void {}

/** @param ComplexFoo|ComplexBar $_ */
function take_complex_union(ComplexFoo|ComplexBar $_): void {}

/** @param list<ComplexFoo> $_ */
function take_complex_foo_list(array $_): void {}

/** @param list{ComplexFoo, ComplexBar} $_ */
function take_complex_tuple(array $_): void {}

/** @param 'foo' $_ */
function take_complex_foo_key(string $_): void {}

/** @param 'item' $_ */
function take_complex_item_key(string $_): void {}

/** @param class-string<ComplexFoo> $_ */
function take_complex_foo_class(string $_): void {}

/** @param class-string<ComplexBar> $_ */
function take_complex_bar_class(string $_): void {}

/** @param interface-string<ComplexContract> $_ */
function take_complex_contract_class(string $_): void {}

/** @param enum-string<ComplexState> $_ */
function take_complex_state_class(string $_): void {}

/** @param trait-string<ComplexBehavior> $_ */
function take_complex_behavior_class(string $_): void {}

/** @param array{name: string} $_ */
function take_complex_foo_properties(array $_): void {}

function take_complex_string_value(string $_): void {}

/** @param class-string<ComplexFoo|ComplexBar> $_ */
function take_complex_object_class(string $_): void {}

/** @param positive-int $_ */
function take_complex_positive(int $_): void {}

/** @param int<1, 10> $_ */
function take_complex_range(int $_): void {}

/** @param numeric-string $_ */
function take_complex_numeric_string(string $_): void {}

/** @param non-empty-string $_ */
function take_complex_non_empty_string(string $_): void {}

/** @param lowercase-string $_ */
function take_complex_lowercase_string(string $_): void {}

/** @param non-empty-array<string, ComplexFoo> $_ */
function take_complex_non_empty_array(array $_): void {}

/** @param iterable<string, ComplexFoo> $_ */
function take_complex_iterable(iterable $_): void {}

/** @param resource $_ */
function take_complex_resource(mixed $_): void {}

/** @param open-resource $_ */
function take_complex_open_resource(mixed $_): void {}

/** @param closed-resource $_ */
function take_complex_closed_resource(mixed $_): void {}

/** @param 'ready' $_ */
function take_complex_ready_value(string $_): void {}

/** @param ComplexGenericCollection<ComplexFoo>&Countable $_ */
function take_complex_generic_collection(ComplexGenericCollection $_): void {}

/** @param iterable<ComplexFoo>&Countable $_ */
function take_complex_iterable_collection(iterable $_): void {}
