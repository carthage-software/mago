<?php

/** @param null|array-key $_value */
function take_nullable_value(mixed $_value): void {}

/** @param array-key $_value */
function take_non_null_value(mixed $_value): void {}


class Iterate
{
    public function iterate(mixed $iterator): void
    {
        $key = null;
        $value = null;

        if (\is_iterable($iterator)) {
            try {
                /**
                 * @var array-key $key
                 * @var array-key $value
                 */
                foreach ($iterator as $key => $value) {
                    take_non_null_value($key);
                    take_non_null_value($value);
                }
            } catch (\Throwable) {
                take_nullable_value($key);
                take_nullable_value($value);
            }
        }
    }
}
