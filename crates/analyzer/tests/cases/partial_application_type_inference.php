<?php

/** @param string $_ */
function accept_string(string $_): void {}

/** @param int $_ */
function accept_int(int $_): void {}

/** @param float $_ */
function accept_float(float $_): void {}

/** @param ?string $_ */
function accept_nullable_string(?string $_): void {}

/** @param array<int, string> $_ */
function accept_string_array(array $_): void {}

/** @param Builder $_ */
function accept_builder(Builder $_): void {}

/** @param Fluent $_ */
function accept_fluent(Fluent $_): void {}

$upper_closure = strtoupper(?);
$upper_result = $upper_closure("hello");
accept_string($upper_result);

$lower_closure = strtolower(?);
$lower_result = $lower_closure("HELLO");
accept_string($lower_result);

class StringProcessor {
    public function uppercase(string $input): string {
        return strtoupper($input);
    }

    public function concat(string $a, string $b): string {
        return $a . $b;
    }

    public function length(string $input): int {
        return strlen($input);
    }
}

$processor = new StringProcessor();
$upper_closure = $processor->uppercase(?);
$result = $upper_closure("hello");
accept_string($result);

$concat_hello = $processor->concat("hello", ?);
$greeting = $concat_hello(" world");
accept_string($greeting);

$length_closure = $processor->length(?);
$len = $length_closure("test");
accept_int($len);

class Math {
    public static function add(int $a, int $b): int {
        return $a + $b;
    }

    public static function multiply(int $a, int $b, int $c): int {
        return $a * $b * $c;
    }
}

$add_five = Math::add(5, ?);
$result_ten = $add_five(5);
accept_int($result_ten);

$multiply_by_two = Math::multiply(?, 2, ?);
$result_twelve = $multiply_by_two(2, 3);
accept_int($result_twelve);

class Calculator {
    public function double(int $x): int {
        return $x * 2;
    }

    public function addTen(int $x): int {
        return $x + 10;
    }
}

$calc = new Calculator();
$double_fn = $calc->double(?);
$add_ten_fn = $calc->addTen(?);

$value = 5;
$doubled_value = $double_fn($value);
accept_int($doubled_value);
$final_value = $add_ten_fn($doubled_value);
accept_int($final_value);

class NullableProcessor {
    public function process(?string $input): ?string {
        return $input;
    }

    public function processInt(?int $value): ?int {
        return $value;
    }
}

$nullable_proc = new NullableProcessor();
$process_closure = $nullable_proc->process(?);
$maybe_string = $process_closure("test");
accept_nullable_string($maybe_string);

$process_int_closure = $nullable_proc->processInt(?);
$maybe_int = $process_int_closure(42);
if ($maybe_int !== null) {
    accept_int($maybe_int);
}

class UnionProcessor {
    public function format(string|int $value): string {
        return (string)$value;
    }
}

$union_proc = new UnionProcessor();
$format_closure = $union_proc->format(?);
$formatted_string = $format_closure(123);
accept_string($formatted_string);

$formatted_from_string = $format_closure("abc");
accept_string($formatted_from_string);

class Combiner {
    public function combine(string $prefix, int $number, string $suffix): string {
        return $prefix . $number . $suffix;
    }
}

$combiner = new Combiner();
$combine_with_hello = $combiner->combine("hello_", ?, "_world");
$combined = $combine_with_hello(42);
accept_string($combined);

class Logger {
    public function log(string $message, string ...$tags): string {
        return $message . implode(",", $tags);
    }
}

$logger = new Logger();
$log_error = $logger->log("Error", ...);
$logged = $log_error("warning", "critical");
accept_string($logged);

class Divider {
    public function divide(int $numerator, int $denominator): float {
        return $numerator / $denominator;
    }
}

$divider = new Divider();
$reciprocal = $divider->divide(numerator: 1, denominator: ?);
$half = $reciprocal(2);
accept_float($half);

class VoidProcessor {
    public function execute(string $command): void {
        echo $command;
    }
}

$void_proc = new VoidProcessor();
$exec_closure = $void_proc->execute(?);
$exec_closure("test");

class ArrayProcessor {
    /**
     * @return array<int, string>
     */
    public function split(string $input, string $delimiter): array {
        return explode($delimiter, $input);
    }
}

$array_proc = new ArrayProcessor();
$split_by_comma = $array_proc->split(?, ",");
$parts = $split_by_comma("a,b,c");
accept_string_array($parts);
foreach ($parts as $part) {
    accept_string($part);
}

final class Builder {
    public static function create(string $name): self {
        return new self();
    }

    public function getName(): string {
        return "builder";
    }
}

$create_closure = Builder::create(?);
$instance = $create_closure("test");
accept_builder($instance);
$name = $instance->getName();
accept_string($name);

class Fluent {
    public function setValue(int $value): self {
        return $this;
    }

    public function getValue(): int {
        return 42;
    }
}

$fluent = new Fluent();
$set_value_closure = $fluent->setValue(?);
$result_fluent = $set_value_closure(10);
accept_fluent($result_fluent);
$retrieved_value = $result_fluent->getValue();
accept_int($retrieved_value);

class MultiUse {
    public function transform(string $input, string $prefix): string {
        return $prefix . $input;
    }
}

$multi = new MultiUse();
$add_hello = $multi->transform(?, "hello_");
$add_goodbye = $multi->transform(?, "goodbye_");

$greeting1 = $add_hello("world");
$greeting2 = $add_goodbye("world");
accept_string($greeting1);
accept_string($greeting2);

strlen(?)(1); // @mago-expect analysis:invalid-argument
strlen(?)("f", 2); // @mago-expect analysis:too-many-arguments
strlen(?)(); // @mago-expect analysis:too-few-arguments
$processor->uppercase(?)(123); // @mago-expect analysis:invalid-argument
$processor->concat(?, ?)(); // @mago-expect analysis:too-few-arguments
Math::add(?)(1); // @mago-expect analysis:too-few-arguments
Math::multiply(?, ?, ?)("a", "b", "c"); // @mago-expect analysis:invalid-argument,invalid-argument,invalid-argument
