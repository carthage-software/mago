<?php

final class PriorUnsetOperandFailure extends RuntimeException
{
}

final class PriorUnsetOperandCallback
{
    /** @throws RuntimeException */
    public static function stored(): void
    {
        throw new RuntimeException();
    }
}

final class PriorUnsetOperandProvider
{
    /**
     * @return 'other'
     * @throws PriorUnsetOperandFailure
     */
    public function key(): string
    {
        throw new PriorUnsetOperandFailure();
    }
}

function unset_prior_operand_throw_preserves_later_callable_storage(): void
{
    $callbacks = [];
    $callbacks['job'] = PriorUnsetOperandCallback::stored(...);
    $other = ['other' => null];
    $provider = new PriorUnsetOperandProvider();

    try {
        unset($other[$provider->key()], $callbacks['job']);
    } catch (PriorUnsetOperandFailure) {
    }

    $callbacks['job']();
}
