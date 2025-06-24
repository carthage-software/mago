<?php

namespace {
    function newrelic_add_custom_parameter(string $key, bool|float|int|string $value): bool
    {
    }

    function newrelic_add_custom_tracer(string $functionName): bool
    {
    }

    function newrelic_background_job(bool $flag = true): void
    {
    }

    function newrelic_capture_params(bool $enable_flag = true): void
    {
    }

    function newrelic_custom_metric(string $metric_name, float $value): bool
    {
    }

    function newrelic_disable_autorum(): null|bool
    {
    }

    /**
     * @deprecated
     */
    function newrelic_enable_params()
    {
    }

    function newrelic_end_of_transaction(): void
    {
    }

    function newrelic_end_transaction(bool $ignore = false): bool
    {
    }

    function newrelic_get_browser_timing_footer(bool $includeTags = true): string
    {
    }

    function newrelic_get_browser_timing_header(bool $includeTags = true): string
    {
    }

    function newrelic_ignore_apdex(): void
    {
    }

    function newrelic_ignore_transaction(): void
    {
    }

    function newrelic_name_transaction(string $name): bool
    {
    }

    function newrelic_notice_error(
        string|Throwable|Exception|int $messageOrExceptionOrCode,
        null|string|Throwable|Exception $errstrOrException = null,
        null|string $errfile = null,
        null|int $errline = null,
        null|string $errcontext = null,
    ): null {
    }

    function newrelic_record_custom_event(string $name, array $attributes): void
    {
    }

    function newrelic_set_appname(string $name, string $license, bool $xmit = false): bool
    {
    }

    function newrelic_set_user_attributes(string $user_value, string $account_value, string $product_value): bool
    {
    }

    function newrelic_set_user_id(string $user_id): bool
    {
    }

    function newrelic_start_transaction(string $appname, null|string $license = null): bool
    {
    }

    function newrelic_record_datastore_segment(callable $func, array $parameters): mixed
    {
    }

    function newrelic_accept_distributed_trace_headers(array $headers, string $transport_type = 'HTTP'): bool
    {
    }

    /**
     * @deprecated
     */
    function newrelic_accept_distributed_trace_payload(string $payload): void
    {
    }

    /**
     * @deprecated
     */
    function newrelic_accept_distributed_trace_payload_httpsafe(
        string $httpsafe_payload,
        string $transport_type = 'HTTP',
    ): bool {
    }

    function newrelic_add_custom_span_parameter(string $key, bool|float|int|string $value): bool
    {
    }

    /**
     * @deprecated
     */
    function newrelic_create_distributed_trace_payload(): newrelic\DistributedTracePayload
    {
    }

    function newrelic_get_linking_metadata(): array
    {
    }

    function newrelic_get_trace_metadata(): array
    {
    }

    function newrelic_insert_distributed_trace_headers(array $headers): bool
    {
    }

    function newrelic_is_sampled(): bool
    {
    }
}

namespace newrelic {
    class DistributedTracePayload
    {
        public function text(): string
        {
        }

        public function httpSafe(): string
        {
        }
    }
}
