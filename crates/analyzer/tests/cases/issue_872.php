<?php

declare(strict_types = 1);

class A {}

/** @mago-expect analysis: invalid-extend */
readonly class B extends A {}
