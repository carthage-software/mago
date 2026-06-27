<?php

declare(strict_types=1);

use ReflectionClass;

/** @template-covariant T of object */
interface MetaPlain
{
    /** @return ReflectionClass<T> */
    public function getReflectionClass();
}

/** @template-covariant T of object */
interface MetaCovariant
{
    /** @return ReflectionClass<covariant T> */
    public function getReflectionClass();
}

function plain(MetaPlain $m): string
{
    return $m->getReflectionClass()->getShortName();
}

function covariant(MetaCovariant $m): string
{
    return $m->getReflectionClass()->getShortName();
}
