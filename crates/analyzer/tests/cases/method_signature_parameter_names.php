<?php

// Test 1: Parameter name change in interface implementation (WARNING)
// PHP: No error, but breaks named arguments at runtime
interface Sizer
{
    public function getSize(string $document): int;
}

class SizerImpl implements Sizer
{
    // @mago-expect analysis:incompatible-parameter-name
    public function getSize(string $file): int
    {
        return 100;
    }
}

// Test 2: Single parameter at different position (WARNING)
interface Processor
{
    public function process(string $input, array $options): mixed;
}

class ProcessorImpl implements Processor
{
    // @mago-expect analysis:incompatible-parameter-name
    public function process(string $input, array $config): mixed
    {
        return null;
    }
}

// Test 3: Parameter names match (OK)
interface Calculator
{
    public function calculate(int $a, int $b): int;
}

class CalculatorImpl implements Calculator
{
    public function calculate(int $a, int $b): int
    {
        return $a + $b;
    }
}

// Test 4: Parameter name change in trait (WARNING)
trait Formatter
{
    abstract public function format(string $text): string;
}

class TextFormatter
{
    use Formatter;

    // @mago-expect analysis:incompatible-parameter-name
    public function format(string $content): string
    {
        return $content;
    }
}

// Test 5: Parameter name change in abstract class (WARNING)
abstract class BaseValidator
{
    abstract public function validate(mixed $value): bool;
}

class StringValidator extends BaseValidator
{
    // @mago-expect analysis:incompatible-parameter-name
    public function validate(mixed $input): bool
    {
        return is_string($input);
    }
}

// Test 6: Parameter name change with diamond inheritance (WARNING)
interface Logger
{
    public function log(string $message): void;
}

interface FileLogger extends Logger {}
interface DatabaseLogger extends Logger {}

class CompositeLogger implements FileLogger, DatabaseLogger
{
    // @mago-expect analysis:incompatible-parameter-name
    public function log(string $entry): void {}
}

// Test 7: Parameter name change with indirect interface (WARNING)
interface Serializer
{
    public function serialize(mixed $data): string;
}

interface JsonSerializer extends Serializer {}

class DefaultJsonSerializer implements JsonSerializer
{
    // @mago-expect analysis:incompatible-parameter-name
    public function serialize(mixed $payload): string
    {
        return '';
    }
}
