<?php

declare(strict_types=1);

trait Doing
{
    public static function doit(): void {}
    protected static function doit_prot(): void {}
}

class BaseTask
{
    use Doing;
}

/**
 * @method static void doit()
 * @method static void doit_prot()
 */
final class SubTask extends BaseTask
{
}

// `SubTask` re-declares the `@method` tag but inherits the real trait method from `BaseTask`.
// The documented method is backed by a real implementation reachable through the ancestry, so the
// call must not be flagged as a missing magic method (regression test for #1184).
SubTask::doit();

// `doit_prot()` is protected: from the global scope the inherited implementation is inaccessible,
// so PHP routes the call through `__callStatic`, which `SubTask` lacks.
SubTask::doit_prot(); // @mago-expect analysis:missing-magic-method

/**
 * @method static void doit()
 */
class DocumentedBase
{
    use Doing;
}

/**
 * @method static void doit()
 */
final class DocumentedSub extends DocumentedBase
{
}

// Both the parent and the subclass re-declare the `@method` tag while the real implementation
// comes from a trait used by the parent. The parent's method slot holds a pseudo-method as well,
// so the real implementation is only found by consulting the used traits directly.
DocumentedSub::doit();
