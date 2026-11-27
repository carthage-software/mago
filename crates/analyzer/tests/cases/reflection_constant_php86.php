<?php

declare(strict_types=1);

$constant = new ReflectionConstant('PHP_VERSION');
echo $constant->inNamespace() ? "namespaced\n" : "global\n";
