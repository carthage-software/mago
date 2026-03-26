<?php

class Example
{
    private const string CONSTANT = 'value';

    // This should come first
    public function __construct(
        private string $name,
    ) {}

    // This should come in static methods section
    public static function publicStaticMethod(): void {}

    // This should come in static methods section
    protected static function protectedStaticMethod(): void {}

    // This should come in static methods section
    private static function privateStaticMethod(): void {}

    // This should come in instance methods section
    public function publicMethod(): void {}

    // This should come in instance methods section
    protected function protectedMethod(): void {}

    // This should come in instance methods section
    private function privateMethod(): void {}

    // This should come last (magic method)
    public function __toString(): string
    {
        return 'Example';
    }

    // This should come last (magic method)
    public function __destruct() {}
}

abstract class AbstractExample
{
    // Abstract static should be in static section
    abstract public static function abstractPublicStatic(): void;

    // Concrete static
    public static function concretePublicStatic(): void {}

    // Abstract method should come before concrete with same visibility
    abstract public function abstractPublic(): void;

    // Regular instance method
    public function concretePublic(): void {}

    // Protected abstract
    abstract protected function abstractProtected(): void;

    // Protected concrete
    protected function concreteProtected(): void {}

    // Private method
    private function concretePrivate(): void {}
}

trait ExampleTrait
{
    public function traitPublic(): void {}

    protected function traitProtected(): void {}

    // Traits should also be sorted
    private function traitPrivate(): void {}
}

interface ExampleInterface
{
    public function apple(): void;

    public function banana(): void;

    // Interfaces should be sorted alphabetically
    public function zebra(): void;
}
