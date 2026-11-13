<?php

declare(strict_types=1);

final class Model
{
    /** @ignore-nullable-return */
    public function name(): ?string
    {
        return mt_rand(1, 100) > 1 ? 'foo' : null;
    }

    /** @ignore-falsable-return */
    public function id(): int|false
    {
        return mt_rand(1, 100) > 1 ? 1 : false;
    }

    public function strictName(): ?string
    {
        return mt_rand(1, 100) > 1 ? 'foo' : null;
    }

    public function strictId(): int|false
    {
        return mt_rand(1, 100) > 1 ? 1 : false;
    }
}

/** @return non-empty-array<string, int> */
function lenient_name(Model $model): array
{
    return [$model->name() => 1];
}

/** @return non-empty-array<int, int> */
function lenient_id(Model $model): array
{
    return [$model->id() => 1];
}

/** @return non-empty-array<array-key, int> */
function strict_name(Model $model): array
{
    // @mago-expect analysis:invalid-array-element-key
    return [$model->strictName() => 1];
}

/** @return non-empty-array<array-key, int> */
function strict_id(Model $model): array
{
    // @mago-expect analysis:invalid-array-element-key
    return [$model->strictId() => 1];
}
