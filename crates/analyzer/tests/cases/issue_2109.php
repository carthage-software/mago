<?php

declare(strict_types=1);

$value = random_int(3, 12);

/**
 * @mago-expect analysis:redundant-type-comparison(1),analysis:redundant-logical-operation(2),analysis:redundant-comparison(2)
 */
assert($value >= 3 && $value <= 12);

/**
 * @mago-expect analysis:redundant-type-comparison(1), redundant-logical-operation(2), redundant-comparison(2)
 */
assert($value >= 3 && $value <= 12);

/** @mago-expect analysis:redundant-type-comparison(1),redundant-logical-operation(2),redundant-comparison(2) */
assert($value >= 3 && $value <= 12);

/** @mago-expect analysis:redundant-type-comparison(1), analysis:redundant-logical-operation(2), analysis:redundant-comparison(2) */
assert($value >= 3 && $value <= 12);

// @mago-ignore analysis:redundant-type-comparison, analysis:redundant-logical-operation(2), redundant-comparison(2)
assert($value >= 3 && $value <= 12);
