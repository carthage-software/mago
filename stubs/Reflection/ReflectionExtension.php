<?php

use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Immutable;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\TentativeType;
use JetBrains\PhpStorm\Pure;

class ReflectionExtension implements Reflector
{
    /**
     * @var string Name of the extension, same as calling the {@see ReflectionExtension::getName()} method
     */
    #[Immutable]
    #[LanguageLevelTypeAware(['8.1' => 'string'], default: '')]
    public $name;

    /**
     * Constructs a ReflectionExtension
     *
     * @link https://php.net/manual/en/reflectionextension.construct.php
     * @param string $name Name of the extension.
     * @throws ReflectionException if the extension does not exist.
     */
    public function __construct(string  $name) {}

    /**
     * To string
     *
     * @link https://php.net/manual/en/reflectionextension.tostring.php
     * @return string the exported extension as a string, in the same way as
     * the {@see ReflectionExtension::export()}.
     */
    public function __toString(): string
    {
    }

    /**
     * Gets extension name
     *
     * @link https://php.net/manual/en/reflectionextension.getname.php
     * @return string The extensions name.
     */
    #[Pure]
    public function getName(): string
    {
    }

    /**
     * Gets extension version
     *
     * @link https://php.net/manual/en/reflectionextension.getversion.php
     * @return string|null The version of the extension.
     */
    #[Pure]
    public function getVersion(): null|string
    {
    }

    /**
     * Gets extension functions
     *
     * @link https://php.net/manual/en/reflectionextension.getfunctions.php
     * @return ReflectionFunction[] An associative array of {@see ReflectionFunction} objects,
     * for each function defined in the extension with the keys being the function
     * names. If no function are defined, an empty array is returned.
     */
    #[Pure]
    public function getFunctions(): array
    {
    }

    /**
     * Gets constants
     *
     * @link https://php.net/manual/en/reflectionextension.getconstants.php
     * @return array An associative array with constant names as keys.
     */
    #[Pure]
    public function getConstants(): array
    {
    }

    /**
     * Gets extension ini entries
     *
     * @link https://php.net/manual/en/reflectionextension.getinientries.php
     * @return array An associative array with the ini entries as keys,
     * with their defined values as values.
     */
    #[Pure]
    public function getINIEntries(): array
    {
    }

    /**
     * Gets classes
     *
     * @link https://php.net/manual/en/reflectionextension.getclasses.php
     * @return ReflectionClass[] An array of {@see ReflectionClass} objects, one
     * for each class within the extension. If no classes are defined,
     * an empty array is returned.
     */
    #[Pure]
    public function getClasses(): array
    {
    }

    /**
     * Gets class names
     *
     * @link https://php.net/manual/en/reflectionextension.getclassnames.php
     * @return string[] An array of class names, as defined in the extension.
     * If no classes are defined, an empty array is returned.
     */
    #[Pure]
    public function getClassNames(): array
    {
    }

    /**
     * Gets dependencies
     *
     * @link https://php.net/manual/en/reflectionextension.getdependencies.php
     * @return string[] An associative array with dependencies as keys and
     * either Required, Optional or Conflicts as the values.
     */
    #[Pure]
    public function getDependencies(): array
    {
    }

    /**
     * Print extension info
     *
     * @link https://php.net/manual/en/reflectionextension.info.php
     * @return void Print extension info
     */
    public function info(): void
    {
    }

    /**
     * Returns whether this extension is persistent
     *
     * @link https://php.net/manual/en/reflectionextension.ispersistent.php
     * @return bool Returns {@see true} for extensions loaded by extension, {@see false} otherwise.
     * @since 5.4
     */
    #[Pure]
    public function isPersistent(): bool
    {
    }

    /**
     * Returns whether this extension is temporary
     *
     * @link https://php.net/manual/en/reflectionextension.istemporary.php
     * @return bool Returns {@see true} for extensions loaded by {@see dl()}, {@see false} otherwise.
     * @since 5.4
     */
    #[Pure]
    public function isTemporary(): bool
    {
    }

    /**
     * Clones
     *
     * @link https://php.net/manual/en/reflectionextension.clone.php
     * @return void No value is returned, if called a fatal error will occur.
     */
    #[PhpStormStubsElementAvailable(from: '5.4', to: '8.0')]
    final private function __clone(): void
    {
    }

    /**
     * Clones
     *
     * @link https://php.net/manual/en/reflectionextension.clone.php
     * @return void No value is returned, if called a fatal error will occur.
     */
    #[PhpStormStubsElementAvailable(from: '8.1')]
    private function __clone(): void
    {
    }
}
