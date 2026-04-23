<?php

class Period
{
    public function __call($method, $parameters)
    {
        switch ($method) {
            case 'hour':
            case 'minute':
            case 'second':
            case 'millisecond':
            case 'microsecond':
                return $this->setDateInterval(
                    // Override default P1D when instantiating via fluent setters.
                    [$this->isDefaultInterval ? new CarbonInterval('PT0S') : $this->dateInterval, $method](
                        ...$parameters,
                    ),
                );
        }

        return null;
    }
}
