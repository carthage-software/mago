<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 *
 */
interface GenIfaceTwo {}

/**
 *
 * @implements GenIfaceTwo<int>
 */
final class GenIfaceTwoImpl implements GenIfaceTwo {}
