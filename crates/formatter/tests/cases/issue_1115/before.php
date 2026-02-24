<?php

$simple = new #[Attribute]
class {
    public function foo(): void {}
};

$multiple = new #[Attribute1]
#[Attribute2]
class {
    public function bar(): void {}
};

$alreadyFormatted = new #[Attribute]
class {
    public function baz(): void {}
};

$withArgs =
    new #[Attribute]
    class($foo, $bar) {
        public function __construct(
            private readonly mixed $foo,
            private readonly mixed $bar,
        ) {}
    };

$withInheritance = new #[SomeAttribute]
#[AnotherAttribute]
class extends BaseClass implements SomeInterface {
    public function doSomething(): void {}
};

$readonly =
    new #[Immutable]
    readonly class {
        public function __construct(
            public string $value,
        ) {}
    };

$noAttributes = new class {
    public function test(): void {}
};

$noAttributesWithArgs = new class($dependency) {
    public function __construct(
        private mixed $dependency,
    ) {}
};

$attributeWithArgs = new #[Route('/api/users', methods: ['GET', 'POST'])]
class {
    public function handle(): void {}
};

$multipleAttributeLists = new #[Attribute1]
#[Attribute2]
#[Attribute3]
class implements FirstInterface, SecondInterface {
    public function method(): void {}
};