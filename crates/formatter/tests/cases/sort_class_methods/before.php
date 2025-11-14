<?php

class Example
{
    private const string CONSTANT = 'value';

    // This should come last (magic method)
    public function __toString(): string
    {
        return 'Example';
    }

    // This should come in instance methods section
    private function privateMethod(): void
    {
    }

    // This should come in static methods section
    public static function publicStaticMethod(): void
    {
    }

    // This should come first
    public function __construct(
        private string $name,
    ) {}

    // This should come in instance methods section
    protected function protectedMethod(): void
    {
    }

    // This should come in static methods section
    private static function privateStaticMethod(): void
    {
    }

    // This should come in instance methods section
    public function publicMethod(): void
    {
    }

    // This should come last (magic method)
    public function __destruct()
    {
    }

    // This should come in static methods section
    protected static function protectedStaticMethod(): void
    {
    }
}

abstract class AbstractExample
{
    // Regular instance method
    public function concretePublic(): void
    {
    }

    // Abstract method should come before concrete with same visibility
    abstract public function abstractPublic(): void;

    // Private method
    private function concretePrivate(): void
    {
    }

    // Abstract static should be in static section
    abstract public static function abstractPublicStatic(): void;

    // Concrete static
    public static function concretePublicStatic(): void
    {
    }

    // Protected abstract
    abstract protected function abstractProtected(): void;

    // Protected concrete
    protected function concreteProtected(): void
    {
    }
}

trait ExampleTrait
{
    // Traits should also be sorted
    private function traitPrivate(): void
    {
    }

    public function traitPublic(): void
    {
    }

    protected function traitProtected(): void
    {
    }
}

interface ExampleInterface
{
    // Interfaces should be sorted alphabetically
    public function zebra(): void;

    public function apple(): void;

    public function banana(): void;
}
