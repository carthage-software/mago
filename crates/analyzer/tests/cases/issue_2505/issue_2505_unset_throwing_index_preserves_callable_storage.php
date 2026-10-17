<?php

final class ThrowingUnsetIndexFailure extends RuntimeException
{
}

final class DeferredUnsetIndexFailure extends Exception
{
}

final class DeferredUnsetIndexCallback
{
    /** @throws DeferredUnsetIndexFailure */
    public static function stored(): void
    {
        throw new DeferredUnsetIndexFailure();
    }
}

final class UnsetIndexProvider
{
    public bool $throws = false;

    /** @throws ThrowingUnsetIndexFailure */
    public function key(): string
    {
        if ($this->throws) {
            throw new ThrowingUnsetIndexFailure();
        }

        return 'job';
    }
}

function unset_throwing_index_preserves_callable_storage(): void
{
    $callbacks = [];
    $provider = new UnsetIndexProvider();

    try {
        $callbacks[$provider->key()] = DeferredUnsetIndexCallback::stored(...);
    } catch (ThrowingUnsetIndexFailure) {
    }

    $provider->throws = true;

    try {
        unset($callbacks[$provider->key()]);
    } catch (ThrowingUnsetIndexFailure) {
    }
}
