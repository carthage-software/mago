<?php

namespace Test;

class Model {}

/**
 * @template T of Model
 */
interface HasEmojiReactionsContract
{
    /** @param T $_model */
    public function doSomething(Model $_model): void;
}

/**
 * @implements HasEmojiReactionsContract<$this>
 */
class Article extends Model implements HasEmojiReactionsContract
{
    /** @param $this $_model */
    public function doSomething(Model $_model): void
    {
        return;
    }
}

/**
 * @template T of Model
 *
 * @param HasEmojiReactionsContract<T> $herc
 */
function someFunction(HasEmojiReactionsContract $herc) {}

someFunction(new Article());
