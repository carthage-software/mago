<?php

declare(strict_types=1);

abstract class AbstractController
{
}

/**
 * @template TEntity of object
 */
interface CrudControllerInterface
{
    /**
     * @return class-string<TEntity>
     */
    public static function getEntityFqcn(): string;
}

/**
 * @template TEntity of object
 * @implements CrudControllerInterface<TEntity>
 */
abstract class AbstractCrudController extends AbstractController implements CrudControllerInterface
{
    abstract public static function getEntityFqcn(): string;
}

/**
 * @extends AbstractCrudController<stdClass>
 */
final class CrudController1 extends AbstractCrudController
{
    public static function getEntityFqcn(): string
    {
        return stdClass::class;
    }
}

/**
 * @extends AbstractCrudController<stdClass>
 */
final class CrudController2 extends AbstractCrudController
{
    /**
     * {@inheritDoc}
     */
    public static function getEntityFqcn(): string
    {
        return stdClass::class;
    }
}

/**
 * @extends AbstractCrudController<stdClass>
 */
final class CrudController3 extends AbstractCrudController
{
    /**
     * @inheritDoc
     */
    public static function getEntityFqcn(): string
    {
        return stdClass::class;
    }
}

/**
 * @extends AbstractCrudController<stdClass>
 */
final class CrudController4 extends AbstractCrudController
{
    /** @return class-string<stdClass> */
    public static function getEntityFqcn(): string
    {
        return stdClass::class;
    }
}
