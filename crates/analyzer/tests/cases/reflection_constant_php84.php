<?php

declare(strict_types=1);

$constant = new ReflectionConstant('PHP_VERSION');
echo $constant->getName(), "\n";

// @mago-expect analysis:unavailable-method
$_ = $constant->getFileName();

// @mago-expect analysis:unavailable-method
$_ = $constant->inNamespace();
