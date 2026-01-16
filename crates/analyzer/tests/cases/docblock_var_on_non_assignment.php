<?php

declare(strict_types=1);

/**
 * Test that @var annotations work for variables other than the assignment target.
 *
 * This tests the fix for an issue where @var annotations for loop variables
 * were being ignored when appearing before assignment statements.
 */

/** @var object[] $objs */
$objs = [new DateTime()];

foreach ($objs as $obj) {
    /** @var DateTime $obj */
    $c = $obj->getTimestamp() > 1000;
}

// Also test with multiple @var annotations
/** @var array<int|string, object> $map */
$map = ['key' => new DateTime()];

function want_string(string $x): string {
  return $x;
}

foreach ($map as $key => $value) {
    /** @var string $key */
    /** @var DateTime $value */
    $result = $value->format('Y-m-d');
    want_string($key);
}
