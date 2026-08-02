<?php

declare(strict_types=1);

/** @param 64 $value */
function acceptsTargetConstant(int $value): void {}

/** @param 127 $value */
function acceptsAllTargets(int $value): void {}

/** @param 128 $value */
function acceptsRepeatable(int $value): void {}

acceptsTargetConstant(Attribute::TARGET_CONSTANT);
acceptsAllTargets(Attribute::TARGET_ALL);
acceptsRepeatable(Attribute::IS_REPEATABLE);

#[Attribute(Attribute::TARGET_CONSTANT)]
final class ConstantAttribute {}

#[ConstantAttribute]
const ATTRIBUTED_CONSTANT = 1;
