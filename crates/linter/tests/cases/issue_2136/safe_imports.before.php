<?php

namespace Other;

function takes(SomeInterface $value): void {}

namespace A;

function SomeInterface(): void {}
const SomeInterface = 1;

SomeInterface();
SomeInterface(...);
echo SomeInterface;
new namespace\SomeInterface();
new \B\SomeInterface();

namespace Current;

new SomeInterface();
new \Current\SomeInterface();

namespace Aliased;

use B\SomeInterface as ImportedInterface;

new SomeInterface();
new \B\SomeInterface();
