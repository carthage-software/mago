<?php

/**
 * This file is part of the Nette Framework (https://nette.org)
 * Copyright (c) 2004 David Grudl (https://davidgrudl.com)
 */

declare(strict_types=1);

namespace Nette\PhpGenerator;

use Nette;
use Nette\Utils\Reflection;
use function array_diff, array_filter, array_key_exists, array_map, count, explode, file_get_contents, implode, is_object, is_subclass_of, method_exists, reset;
use const PHP_VERSION_ID;


/**
 * Creates a representations based on reflection or source code.
 */
final class Factory
{
	/** @var array<string, array<string, string>> */
	private array $bodyCache = [];

	/** @var array<string, Extractor> */
	private array $extractorCache = [];


	/** @param  \ReflectionClass<object>  $from */
	public function fromClassReflection(
		\ReflectionClass $from,
		bool $withBodies = false,
	): ClassLike
	{
		if ($withBodies && ($from->isAnonymous() || $from->isInternal() || $from->isInterface())) {
			throw new Nette\NotSupportedException('The $withBodies parameter cannot be used for anonymous or internal classes or interfaces.');
		}

		$class = $this->createClassObject($from);
		$this->setupInheritance($class, $from);
		$this->populateMembers($class, $from, $withBodies);
		return $class;
	}


	/** @param \ReflectionClass<object> $from */
	private function createClassObject(\ReflectionClass &$from): ClassLike
	{
		if ($from->isAnonymous()) {
			return new ClassType;
		} elseif ($from->isEnum()) {
			$from = new \ReflectionEnum($from->getName());
			$class = new EnumType($from->getName());
		} elseif ($from->isInterface()) {
			$class = new InterfaceType($from->getName());
		} elseif ($from->isTrait()) {
			$class = new TraitType($from->getName());
		} else {
			$class = new ClassType($from->getShortName());
			$class->setFinal($from->isFinal() && $class->isClass());
			$class->setAbstract($from->isAbstract() && $class->isClass());
			$class->setReadOnly(PHP_VERSION_ID >= 80200 && $from->isReadOnly());
		}

		(new PhpNamespace($from->getNamespaceName()))->add($class);
		return $class;
	}


	/** @param \ReflectionClass<object> $from */
	private function setupInheritance(ClassLike $class, \ReflectionClass $from): void
	{
		$ifaces = $from->getInterfaceNames();
		foreach ($ifaces as $iface) {
			$ifaces = array_filter($ifaces, fn(string $item): bool => !is_subclass_of($iface, $item));
		}

		if ($from->isInterface()) {
			assert($class instanceof InterfaceType);
			$class->setExtends(array_values($ifaces));
		} elseif ($ifaces) {
			assert($class instanceof ClassType || $class instanceof EnumType);
			$ifaces = array_diff($ifaces, [\BackedEnum::class, \UnitEnum::class]);
			$class->setImplements(array_values($ifaces));
		}

		$class->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		$class->setAttributes($this->formatAttributes($from->getAttributes()));
		if ($from->getParentClass()) {
			assert($class instanceof ClassType);
			$class->setExtends($from->getParentClass()->name);
			$class->setImplements(array_values(array_diff($class->getImplements(), $from->getParentClass()->getInterfaceNames())));
		}
	}


	/** @param \ReflectionClass<object> $from */
	private function populateMembers(ClassLike $class, \ReflectionClass $from, bool $withBodies): void
	{
		// Properties
		$props = [];
		foreach ($from->getProperties() as $prop) {
			$declaringClass = Reflection::getPropertyDeclaringClass($prop);

			if ($prop->isDefault()
				&& $declaringClass->name === $from->name
				&& !$prop->isPromoted()
				&& !$class->isEnum()
			) {
				$props[] = $p = $this->fromPropertyReflection($prop);
				if ($withBodies && ($file = $declaringClass->getFileName())) {
					$hookBodies ??= $this->getExtractor($file)->extractPropertyHookBodies($declaringClass->name);
					foreach ($hookBodies[$prop->getName()] ?? [] as $hookType => [$body, $short]) {
						$p->getHook($hookType)->setBody($body, short: $short);
					}
				}
			}
		}

		if ($props) {
			assert($class instanceof ClassType || $class instanceof InterfaceType || $class instanceof TraitType);
			$class->setProperties($props);
		}

		// Methods and trait resolutions
		$methods = $resolutions = [];
		foreach ($from->getMethods() as $method) {
			$declaringMethod = Reflection::getMethodDeclaringMethod($method);
			$declaringClass = $declaringMethod->getDeclaringClass();

			if (
				$declaringClass->name === $from->name
				&& (!$from instanceof \ReflectionEnum || !method_exists($from->isBacked() ? \BackedEnum::class : \UnitEnum::class, $method->name))
			) {
				$methods[] = $m = $this->fromMethodReflection($method);
				if ($withBodies && ($file = $declaringClass->getFileName())) {
					$bodies = &$this->bodyCache[$declaringClass->name];
					$bodies ??= $this->getExtractor($file)->extractMethodBodies($declaringClass->name);
					if (isset($bodies[$declaringMethod->name])) {
						$m->setBody($bodies[$declaringMethod->name]);
					}
				}
			}

			$modifier = $declaringMethod->getModifiers() !== $method->getModifiers()
				? ' ' . $this->getVisibility($method)->value
				: null;
			$alias = $declaringMethod->name !== $method->name ? ' ' . $method->name : '';
			if ($modifier || $alias) {
				$resolutions[] = $declaringMethod->name . ' as' . $modifier . $alias;
			}
		}

		assert($class instanceof ClassType || $class instanceof InterfaceType || $class instanceof TraitType || $class instanceof EnumType);
		$class->setMethods($methods);

		// Traits
		foreach ($from->getTraitNames() as $trait) {
			assert($class instanceof ClassType || $class instanceof TraitType || $class instanceof EnumType);
			$trait = $class->addTrait($trait);
			foreach ($resolutions as $resolution) {
				$trait->addResolution($resolution);
			}
			$resolutions = [];
		}

		// Constants and enum cases
		$consts = $cases = [];
		foreach ($from->getReflectionConstants() as $const) {
			if ($from instanceof \ReflectionEnum && $from->hasCase($const->name)) {
				$cases[] = $this->fromCaseReflection($const);
			} elseif ($const->getDeclaringClass()->name === $from->name) {
				$consts[] = $this->fromConstantReflection($const);
			}
		}

		if ($consts) {
			$class->setConstants($consts);
		}
		if ($cases) {
			assert($class instanceof EnumType);
			$class->setCases($cases);
		}
	}


	public function fromMethodReflection(\ReflectionMethod $from): Method
	{
		$method = new Method($from->name);
		$method->setParameters(array_map($this->fromParameterReflection(...), $from->getParameters()));
		$method->setStatic($from->isStatic());
		$isInterface = $from->getDeclaringClass()->isInterface();
		$method->setVisibility($isInterface ? null : $this->getVisibility($from));
		$method->setFinal($from->isFinal());
		$method->setAbstract($from->isAbstract() && !$isInterface);
		$method->setReturnReference($from->returnsReference());
		$method->setVariadic($from->isVariadic());
		$method->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		$method->setAttributes($this->formatAttributes($from->getAttributes()));
		$method->setReturnType((string) $from->getReturnType());

		return $method;
	}


	public function fromFunctionReflection(\ReflectionFunction $from, bool $withBody = false): GlobalFunction|Closure
	{
		$function = $from->isClosure() ? new Closure : new GlobalFunction($from->name);
		$function->setParameters(array_map($this->fromParameterReflection(...), $from->getParameters()));
		$function->setReturnReference($from->returnsReference());
		$function->setVariadic($from->isVariadic());
		if (!$from->isClosure()) {
			assert($function instanceof GlobalFunction);
			$function->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		}

		$function->setAttributes($this->formatAttributes($from->getAttributes()));
		$function->setReturnType((string) $from->getReturnType());

		if ($withBody) {
			if ($from->isClosure() || $from->isInternal() || !($file = $from->getFileName())) {
				throw new Nette\NotSupportedException('The $withBody parameter cannot be used for closures or internal functions.');
			}

			$function->setBody($this->getExtractor($file)->extractFunctionBody($from->name));
		}

		return $function;
	}


	/** @param callable(): mixed  $from */
	public function fromCallable(callable $from): Method|GlobalFunction|Closure
	{
		$ref = Nette\Utils\Callback::toReflection($from);
		return $ref instanceof \ReflectionMethod
			? $this->fromMethodReflection($ref)
			: $this->fromFunctionReflection($ref);
	}


	public function fromParameterReflection(\ReflectionParameter $from): Parameter
	{
		if ($from->isPromoted()) {
			$property = $from->getDeclaringClass()->getProperty($from->name);
			$param = (new PromotedParameter($from->name))
				->setVisibility($this->getVisibility($property))
				->setReadOnly($property->isReadonly())
				->setFinal(PHP_VERSION_ID >= 80500 && $property->isFinal() && !$property->isPrivateSet());
			$this->addHooks($property, $param);
		} else {
			$param = new Parameter($from->name);
		}
		$param->setReference($from->isPassedByReference());
		$param->setType((string) $from->getType());

		if ($from->isDefaultValueAvailable()) {
			if ($from->isDefaultValueConstant()) {
				$parts = explode('::', $from->getDefaultValueConstantName());
				if (count($parts) > 1) {
					$parts[0] = Helpers::tagName($parts[0]);
				}

				$param->setDefaultValue(new Literal(implode('::', $parts)));
			} elseif (is_object($from->getDefaultValue())) {
				$param->setDefaultValue($this->fromObject($from->getDefaultValue()));
			} else {
				$param->setDefaultValue($from->getDefaultValue());
			}
		}

		$param->setAttributes($this->formatAttributes($from->getAttributes()));
		return $param;
	}


	public function fromConstantReflection(\ReflectionClassConstant $from): Constant
	{
		$const = new Constant($from->name);
		$const->setValue($from->getValue());
		$const->setVisibility($this->getVisibility($from));
		$const->setFinal($from->isFinal());
		$const->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		$const->setAttributes($this->formatAttributes($from->getAttributes()));
		return $const;
	}


	public function fromCaseReflection(\ReflectionClassConstant $from): EnumCase
	{
		$const = new EnumCase($from->name);
		$const->setValue($from->getValue()->value ?? null);
		$const->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		$const->setAttributes($this->formatAttributes($from->getAttributes()));
		return $const;
	}


	public function fromPropertyReflection(\ReflectionProperty $from): Property
	{
		$defaults = $from->getDeclaringClass()->getDefaultProperties();
		$prop = new Property($from->name);
		$prop->setValue($defaults[$prop->getName()] ?? null);
		$prop->setStatic($from->isStatic());
		$prop->setVisibility($this->getVisibility($from));
		$prop->setType((string) $from->getType());
		$prop->setInitialized($from->hasType() && array_key_exists($prop->getName(), $defaults));
		$prop->setReadOnly($from->isReadOnly());
		$prop->setComment(Helpers::unformatDocComment((string) $from->getDocComment()));
		$prop->setAttributes($this->formatAttributes($from->getAttributes()));

		if (PHP_VERSION_ID >= 80400) {
			$this->addHooks($from, $prop);
			$isInterface = $from->getDeclaringClass()->isInterface();
			$prop->setFinal($from->isFinal() && !$prop->isPrivate(PropertyAccessMode::Set));
			$prop->setAbstract($from->isAbstract() && !$isInterface);
		}
		return $prop;
	}


	private function addHooks(\ReflectionProperty $from, Property|PromotedParameter $prop): void
	{
		if (PHP_VERSION_ID < 80400) {
			return;
		}

		$getV = $this->getVisibility($from);
		$setV = $from->isPrivateSet()
			? Visibility::Private
			: ($from->isProtectedSet() ? Visibility::Protected : $getV);
		$defaultSetV = $from->isReadOnly() && $getV !== Visibility::Private
			? Visibility::Protected
			: $getV;
		if ($setV !== $defaultSetV) {
			$prop->setVisibility($getV === Visibility::Public ? null : $getV, $setV);
		}

		foreach ($from->getHooks() as $type => $hook) {
			$params = $hook->getParameters();
			if (
				count($params) === 1
				&& $params[0]->getName() === 'value'
				&& $params[0]->getType() == $from->getType() // intentionally ==
			) {
				$params = [];
			}
			$prop->addHook($type)
				->setParameters(array_map($this->fromParameterReflection(...), $params))
				->setAbstract($hook->isAbstract())
				->setFinal($hook->isFinal())
				->setReturnReference($hook->returnsReference())
				->setComment(Helpers::unformatDocComment((string) $hook->getDocComment()))
				->setAttributes($this->formatAttributes($hook->getAttributes()));
		}
	}


	public function fromObject(object $obj): Literal
	{
		return new Literal('new \\' . $obj::class . '(/* unknown */)');
	}


	public function fromClassCode(string $code): ClassLike
	{
		$classes = $this->fromCode($code)->getClasses();
		return reset($classes) ?: throw new Nette\InvalidStateException('The code does not contain any class.');
	}


	public function fromCode(string $code): PhpFile
	{
		$reader = new Extractor($code);
		return $reader->extractAll();
	}


	/**
	 * @param  list<\ReflectionAttribute<object>>  $attrs
	 * @return list<Attribute>
	 */
	private function formatAttributes(array $attrs): array
	{
		$res = [];
		foreach ($attrs as $attr) {
			$args = $attr->getArguments();
			foreach ($args as &$arg) {
				if (is_object($arg)) {
					$arg = $this->fromObject($arg);
				}
			}
			$res[] = new Attribute($attr->getName(), $args);
		}
		return $res;
	}


	private function getVisibility(\ReflectionProperty|\ReflectionMethod|\ReflectionClassConstant $from): Visibility
	{
		return $from->isPrivate()
			? Visibility::Private
			: ($from->isProtected() ? Visibility::Protected : Visibility::Public);
	}


	private function getExtractor(string $file): Extractor
	{
		$cache = &$this->extractorCache[$file];
		$cache ??= new Extractor(file_get_contents($file) ?: throw new Nette\InvalidStateException("Unable to read file '$file'."));
		return $cache;
	}
}
