<?php

namespace Other;

function takes(SomeInterface $value): void {}

namespace A;

use B\SomeInterface;

function SomeInterface(): void {}
const SomeInterface = 1;

SomeInterface();
SomeInterface(...);
echo SomeInterface;
new namespace\SomeInterface();
new SomeInterface();

namespace Current;

new SomeInterface();
new SomeInterface();

namespace Aliased;

use B\SomeInterface as ImportedInterface;

new SomeInterface();
new ImportedInterface();
