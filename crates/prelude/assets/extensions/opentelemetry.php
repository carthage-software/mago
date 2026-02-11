<?php

namespace OpenTelemetry\Instrumentation;

use Closure;

function hook(?string $class, string $function, ?Closure $pre = null, ?Closure $post = null): bool {}
