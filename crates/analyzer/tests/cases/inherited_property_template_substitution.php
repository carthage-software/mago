<?php

declare(strict_types=1);

namespace Example;

use function is_a;

class Util
{
    /**
     * @template T of object
     * @param class-string<T> $expected
     * @psalm-assert class-string<T> $actual
     */
    public static function isClass(string $actual, string $expected): bool
    {
        return is_a($actual, $expected, allow_string: true);
    }
}

interface Model {}

class User implements Model {}

/**
 * @template TModel of Model
 */
abstract class Serializer
{
    /** @var TModel */
    protected Model $model;

    /**
     * @param TModel $m
     */
    public function __construct(Model $m)
    {
        $this->model = $m;
    }
}

/**
 * @extends Serializer<User>
 */
class UserSerializer extends Serializer {}

/**
 * @template TModel of Model
 */
class Controller
{
    /**
     * @var class-string<Serializer<TModel>>
     */
    protected string $serializerClass;

    /**
     * @return class-string<Serializer<TModel>>
     */
    public function getSerializerClass(): string
    {
        return $this->serializerClass;
    }

    /**
     * @param class-string<Serializer<TModel>> $c
     */
    public function __construct(string $c)
    {
        $this->serializerClass = $c;
    }
}

/**
 * @extends Controller<User>
 */
class UserController extends Controller
{
    public function foo(): void
    {
        Util::isClass($this->getSerializerClass(), UserSerializer::class);
        Util::isClass($this->serializerClass, UserSerializer::class);
    }
}
