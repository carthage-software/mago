<?php

use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Immutable;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\TentativeType;
use JetBrains\PhpStorm\Pure;

/**
 * The <b>ReflectionFunction</b> class reports
 * information about a function.
 *
 * @link https://php.net/manual/en/class.reflectionfunction.php
 */
class ReflectionFunction extends ReflectionFunctionAbstract
{
    /**
     * @var string Function name, same as calling the {@see ReflectionFunction::getName()} method
     */
    #[Immutable]
    public $name;

    /**
     * Indicates deprecated functions.
     *
     * @link https://www.php.net/manual/en/class.reflectionfunction.php#reflectionfunction.constants.is-deprecated
     */
    public const IS_DEPRECATED = 2048;

    /**
     * Constructs a ReflectionFunction object
     *
     * @link https://php.net/manual/en/reflectionfunction.construct.php
     * @param string|Closure $function The name of the function to reflect or a closure.
     * @throws ReflectionException if the function does not exist.
     */
    public function __construct(#[LanguageLevelTypeAware(['8.0' => 'Closure|string'], default: '')]  $function) {}

    /**
     * Returns the string representation of the ReflectionFunction object.
     *
     * @link https://php.net/manual/en/reflectionfunction.tostring.php
     */
    public function __toString(): string
    {
    }

    /**
     * Checks if function is disabled
     *
     * @link https://php.net/manual/en/reflectionfunction.isdisabled.php
     * @return bool {@see true} if it's disable, otherwise {@see false}
     */
    #[Deprecated(since: '8.0')]
    #[Pure]
    public function isDisabled(): bool
    {
    }

    /**
     * Invokes function
     *
     * @link https://www.php.net/manual/en/reflectionfunction.invoke.php
     * @param mixed ...$args [optional] The passed in argument list. It accepts a
     * variable number of arguments which are passed to the function much
     * like {@see call_user_func} is.
     * @return mixed Returns the result of the invoked function call.
     */
    public function invoke(mixed ...$args): mixed
    {
    }

    /**
     * Invokes function args
     *
     * @link https://php.net/manual/en/reflectionfunction.invokeargs.php
     * @param array $args The passed arguments to the function as an array, much
     * like {@see call_user_func_array} works.
     * @return mixed the result of the invoked function
     */
    public function invokeArgs(array $args): mixed
    {
    }

    /**
     * Returns a dynamically created closure for the function
     *
     * @link https://php.net/manual/en/reflectionfunction.getclosure.php
     * @return Closure|null Returns {@see Closure} or {@see null} in case of an error.
     */
    #[Pure]
    public function getClosure(): Closure
    {
    }

    #[PhpStormStubsElementAvailable(from: '8.2')]
    public function isAnonymous(): bool
    {
    }
}
