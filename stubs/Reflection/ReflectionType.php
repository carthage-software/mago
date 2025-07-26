<?php

use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\TentativeType;
use JetBrains\PhpStorm\Pure;

/**
 * The ReflectionType class reports information about a function's parameters.
 *
 * @link https://www.php.net/manual/en/class.reflectiontype.php
 * @since 7.0
 */
abstract class ReflectionType implements Stringable
{
    /**
     * Checks if null is allowed
     *
     * @link https://php.net/manual/en/reflectiontype.allowsnull.php
     * @return bool Returns {@see true} if {@see null} is allowed, otherwise {@see false}
     * @since 7.0
     */
    public function allowsNull(): bool
    {
    }

    /**
     * To string
     *
     * @link https://php.net/manual/en/reflectiontype.tostring.php
     * @return string Returns the type of the parameter.
     * @since 7.0
     * @see ReflectionNamedType::getName()
     */
    public function __toString(): string
    {
    }

    /**
     * Cloning of this class is prohibited
     *
     * @return void
     */
    #[PhpStormStubsElementAvailable(from: '5.4', to: '8.0')]
    final private function __clone(): void
    {
    }

    /**
     * Cloning of this class is prohibited
     *
     * @return void
     */
    #[PhpStormStubsElementAvailable(from: '8.1')]
    private function __clone(): void
    {
    }
}
