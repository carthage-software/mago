<?php

class Example
{
    private const string CONSTANT = 'value';

    // This should come last (magic method)
    // This should come in instance methods section
    // This should come in static methods section
    // This should come first
    public function __construct(
        private string $name,
    ) {}

    public static function publicStaticMethod(): void
    {
    }

    // This should come in instance methods section
    // This should come in static methods section
    // This should come in instance methods section
    // This should come last (magic method)
    // This should come in static methods section
    protected static function protectedStaticMethod(): void
    {
    }

    private static function privateStaticMethod(): void
    {
    }

    public function publicMethod(): void
    {
    }

    protected function protectedMethod(): void
    {
    }

    private function privateMethod(): void
    {
    }

    public function __toString(): string
    {
        return 'Example';
    }

    public function __destruct()
    {
    }
}

abstract class AbstractExample
{
    // Regular instance method
    // Abstract method should come before concrete with same visibility
    // Private method
    // Abstract static should be in static section
    abstract public static function abstractPublicStatic(): void;

    // Concrete static
    public static function concretePublicStatic(): void
    {
    }

    abstract public function abstractPublic(): void;

    public function concretePublic(): void
    {
    }

    // Protected abstract
    abstract protected function abstractProtected(): void;

    // Protected concrete
    protected function concreteProtected(): void
    {
    }

    private function concretePrivate(): void
    {
    }
}

trait ExampleTrait
{
    // Traits should also be sorted
    public function traitPublic(): void
    {
    }

    protected function traitProtected(): void
    {
    }

    private function traitPrivate(): void
    {
    }
}

interface ExampleInterface
{
    // Interfaces should be sorted alphabetically
    public function apple(): void;

    public function banana(): void;

    public function zebra(): void;
}
