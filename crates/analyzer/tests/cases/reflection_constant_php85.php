<?php

declare(strict_types=1);

final class NamedConstant extends ReflectionConstant {}

$constant = new NamedConstant('PHP_VERSION');
echo $constant->getName(), "\n";
$_ = $constant->getFileName();

// @mago-expect analysis:unavailable-method
$_ = $constant->inNamespace();
