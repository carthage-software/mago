<?php

declare(strict_types=1);

/**
 * @template T
 *
 */
interface GenIfaceSingle {}

/**
 *
 * @implements GenIfaceSingle<int, string>
 */
final class GenIfaceSingleImpl implements GenIfaceSingle {}
