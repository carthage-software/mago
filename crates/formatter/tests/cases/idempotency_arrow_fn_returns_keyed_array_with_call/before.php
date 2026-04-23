<?php

class Model
{
    protected function getOriginalWithoutRewindingModel($key = null, $default = null)
    {
        if ($key) {
            return $this->transformModelValue(
                $key, Arr::get($this->original, $key, $default)
            );
        }

        return (new Collection($this->original))
            ->mapWithKeys(fn ($value, $key) => [$key => $this->transformModelValue($key, $value)])
            ->all();
    }
}
