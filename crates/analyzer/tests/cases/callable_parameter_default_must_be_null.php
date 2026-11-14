<?php

declare(strict_types=1);

// @mago-expect analysis:invalid-parameter-default-value
function a(callable $cb = 'strlen'): void {}

// @mago-expect analysis:invalid-parameter-default-value
function b(?callable $cb = 'strlen'): void {}

function c(?callable $cb = null): void {}

function d(callable|string $cb = 'strlen'): void {}
