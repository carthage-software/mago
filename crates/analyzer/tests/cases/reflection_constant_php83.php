<?php

declare(strict_types=1);

// @mago-expect analysis:unavailable-class-like
$_ = new ReflectionConstant('PHP_VERSION');
