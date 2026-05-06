<?php

declare(strict_types=1);

class Dependency {}

class OtherDependency {}

// =============================================================================
// VALID CASES - These should pass without issues
// =============================================================================

// Correct argument types
$valid1 = new class('hello', 42) {
    public function __construct(
        public string $name,
        public int $age,
    ) {}
};

// No arguments when no constructor parameters
$valid2 = new class() {
    public function __construct() {}
};

// No arguments and no constructor
$valid3 = new class() {};

// Object argument with correct type
$valid4 = new class(new Dependency()) {
    public function __construct(
        public Dependency $dep,
    ) {}
};

// Nullable argument with null
$valid5 = new class(null) {
    public function __construct(
        public ?string $name,
    ) {}
};

// Nullable argument with value
$valid6 = new class('value') {
    public function __construct(
        public ?string $name,
    ) {}
};

// Default parameter - no arg provided
$valid7 = new class() {
    public function __construct(
        public string $name = 'default',
    ) {}
};

// Default parameter - arg provided
$valid8 = new class('custom') {
    public function __construct(
        public string $name = 'default',
    ) {}
};

// =============================================================================
// VARIADIC CASES
// =============================================================================

// Variadic with correct types
$variadic1 = new class('a', 'b', 'c') {
    public function __construct(string ...$_items) {}
};

// Variadic with no args (valid - variadic can be empty)
$variadic2 = new class() {
    public function __construct(string ...$_items) {}
};

// Mixed required and variadic
$variadic3 = new class('required', 'extra1', 'extra2') {
    public function __construct(
        public string $first,
        string ...$_rest,
    ) {}
};

// =============================================================================
// INVALID ARGUMENT TYPES
// =============================================================================

$invalid1 = new class(123) {
    public function __construct(
        public string $name,
    ) {}
};

$invalid2 = new class('not an int') {
    public function __construct(
        public int $count,
    ) {}
};

$invalid3 = new class(new OtherDependency()) {
    public function __construct(
        public Dependency $dep,
    ) {}
};

$invalid4 = new class('wrong', 'also wrong') {
    public function __construct(
        public int $a,
        public int $b,
    ) {}
};

$invalid5 = new class(null) {
    public function __construct(
        public string $name,
    ) {} // not nullable!
};

// =============================================================================
// TOO MANY ARGUMENTS
// =============================================================================

$tooMany1 = new class('a', 'b', 'c') {
    public function __construct(
        public string $name,
    ) {}
};

$tooMany2 = new class(1, 2, 3) {
    public function __construct() {}
};

$tooMany3 = new class('extra') {
    // no constructor at all
};

// =============================================================================
// TOO FEW ARGUMENTS
// =============================================================================

$tooFew1 = new class() {
    public function __construct(
        public string $required,
    ) {}
};

$tooFew2 = new class('only one') {
    public function __construct(
        public string $a,
        public int $b,
    ) {}
};

$tooFew3 = new class('first') {
    public function __construct(
        public string $a,
        public string $b,
        public string $c,
    ) {}
};

// =============================================================================
// VARIADIC INVALID CASES
// =============================================================================

$variadicInvalid1 = new class(1, 2, 3) {
    public function __construct(string ...$_items) {}
};

$variadicInvalid2 = new class('valid', 123, 'valid') {
    public function __construct(string ...$_items) {}
};

$variadicInvalid3 = new class() {
    public function __construct(
        public string $required,
        string ...$_rest,
    ) {}
};

// =============================================================================
// COMBINED ISSUES
// =============================================================================

$combined1 = new class('wrong type', 'extra') {
    public function __construct(
        public int $num,
    ) {}
};
