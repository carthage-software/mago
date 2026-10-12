<?php

declare(strict_types=1);

final class DeprScopeRegistry
{
    /** @deprecated Use constructor injection instead. */
    public function service(): int
    {
        return 1;
    }

    /** @deprecated Use name instead. */
    public string $label = '';

    /** @deprecated Use NEW_LIMIT instead. */
    public const OLD_LIMIT = 10;

    public const NEW_LIMIT = 30;
}

/** @deprecated */
trait DeprScopeHelper
{
}

/** @deprecated */
class DeprScopeBase
{
}

/** @deprecated */
final class DeprScopeLegacyController
{
    public function handle(DeprScopeRegistry $registry): int
    {
        return $registry->service() + strlen($registry->label) + DeprScopeRegistry::OLD_LIMIT;
    }
}

/** @deprecated */
function depr_scope_bootstrap(DeprScopeRegistry $registry): int
{
    return $registry->service();
}

/** @deprecated */
final class DeprScopeTraitUser
{
    use DeprScopeHelper;
}

/** @deprecated */
final class DeprScopeChild extends DeprScopeBase
{
}

final class DeprScopeCurrent
{
    /** @deprecated Use runNew(). */
    public function runOld(DeprScopeRegistry $registry): int
    {
        return $registry->service();
    }

    public function runNew(DeprScopeRegistry $registry): int
    {
        /** @mago-expect analysis:deprecated-method */
        return $registry->service();
    }

    public function readNew(DeprScopeRegistry $registry): int
    {
        /** @mago-expect analysis:deprecated-property */
        $label = $registry->label;
        /** @mago-expect analysis:deprecated-class-constant */
        return strlen($label) + DeprScopeRegistry::OLD_LIMIT;
    }
}

/** @mago-expect analysis:deprecated-trait */
final class DeprScopeCurrentTraitUser
{
    use DeprScopeHelper;
}
