<?php

function test()
{
    return (
        $this->timeoutSeconds > 0
        && $this->startedAt !== null
        && ($this->clock->now()->getTimestamp() - $this->startedAt) > $this->timeoutSeconds
    );
}
