<?php

declare(strict_types=1);

/**
 * @param callable $_x
 */
function bad($_x): void {}

/**
 * @param callable(array{x: string}) $_x
 */
function good($_x): void {}

/**
 * @param callable(int) $_x
 */
function weird($_x): void {}

good(static fn(array $p): mixed => $p);
bad(static fn(array $p): mixed => $p);
weird(static fn(array $p): mixed => $p); // @mago-expect analysis:invalid-argument
