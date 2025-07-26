<?php

function gearman_version(): string
{
}

function gearman_bugreport(): string
{
}

function gearman_verbose_name(int $verbose): null|string
{
}

function gearman_client_create(): GearmanClient|false
{
}

function gearman_worker_create(): GearmanWorker|false
{
}

class GearmanClient
{
    public function __construct() {}

    public function __destruct()
    {
    }

    public function returnCode(): int
    {
    }

    public function error(): string|false
    {
    }

    public function getErrno(): int
    {
    }

    public function options(): int
    {
    }

    public function setOptions(int $option): bool
    {
    }

    public function addOptions(int $option): bool
    {
    }

    public function removeOptions(int $option): bool
    {
    }

    public function timeout(): int
    {
    }

    public function setTimeout(int $timeout): bool
    {
    }

    public function addServer(null|string $host = null, int $port = 0, bool $setupExceptionHandler = true): bool
    {
    }

    public function addServers(null|string $servers = null, bool $setupExceptionHandler = true): bool
    {
    }

    public function wait(): bool
    {
    }

    public function doNormal(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doHigh(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doLow(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doBackground(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doHighBackground(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doLowBackground(string $function, string $workload, null|string $unique = null): string
    {
    }

    public function doJobHandle(): string
    {
    }

    public function doStatus(): array
    {
    }

    public function jobStatus(string $job_handle): array
    {
    }

    public function jobStatusByUniqueKey(string $unique_key): array
    {
    }

    public function ping(string $workload): bool
    {
    }

    public function addTask(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function addTaskHigh(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function addTaskLow(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function addTaskBackground(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function addTaskHighBackground(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function addTaskLowBackground(
        string $function_name,
        string|int|float $workload,
        mixed $context = null,
        null|string $unique_key = null,
    ): GearmanTask|false {
    }

    public function runTasks(): bool
    {
    }

    public function addTaskStatus(string $job_handle, mixed $context = null): GearmanTask|false
    {
    }

    public function setWorkloadCallback(callable $function): bool
    {
    }

    public function setCreatedCallback(callable $function): bool
    {
    }

    public function setDataCallback(callable $function): bool
    {
    }

    public function setWarningCallback(callable $function): bool
    {
    }

    public function setStatusCallback(callable $function): bool
    {
    }

    public function setCompleteCallback(callable $function): bool
    {
    }

    public function setExceptionCallback(callable $function): bool
    {
    }

    public function setFailCallback(callable $function): bool
    {
    }

    public function clearCallbacks(): bool
    {
    }

    public function context(): string
    {
    }

    public function setContext(string $data): bool
    {
    }

    public function enableExceptionHandler(): bool
    {
    }
}

function gearman_client_return_code(GearmanClient $obj): int
{
}

function gearman_client_error(GearmanClient $obj): string|false
{
}

function gearman_client_get_errno(GearmanClient $obj): int
{
}

function gearman_client_options(GearmanClient $obj): int
{
}

function gearman_client_set_options(GearmanClient $obj, int $option): bool
{
}

function gearman_client_add_options(GearmanClient $obj, int $option): bool
{
}

function gearman_client_remove_options(GearmanClient $obj, int $option): bool
{
}

function gearman_client_timeout(GearmanClient $obj): null|int
{
}

function gearman_client_set_timeout(GearmanClient $obj, int $timeout): bool
{
}

function gearman_client_add_server(
    GearmanClient $obj,
    null|string $host = null,
    int $port = 0,
    bool $setupExceptionHandler = true,
): bool {
}

function gearman_client_add_servers(
    GearmanClient $obj,
    null|string $servers = null,
    bool $setupExceptionHandler = true,
): bool {
}

function gearman_client_wait(GearmanClient $obj): bool
{
}

function gearman_client_do_normal(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_high(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_low(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_background(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_high_background(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_low_background(
    GearmanClient $obj,
    string $function,
    string $workload,
    null|string $unique = null,
): string {
}

function gearman_client_do_job_handle(GearmanClient $obj): string
{
}

function gearman_client_do_status(GearmanClient $obj): array
{
}

function gearman_client_job_status(GearmanClient $obj, string $job_handle): array
{
}

function gearman_client_job_status_by_unique_key(GearmanClient $obj, string $unique_key): array
{
}

function gearman_client_ping(GearmanClient $obj, string $workload): bool
{
}

function gearman_client_add_task(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_add_task_high(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_add_task_low(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_add_task_background(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_add_task_high_background(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_add_task_low_background(
    GearmanClient $obj,
    string $function_name,
    string|int|float $workload,
    mixed $context = null,
    null|string $unique_key = null,
): GearmanTask|false {
}

function gearman_client_run_tasks(GearmanClient $obj): bool
{
}

function gearman_client_add_task_status(
    GearmanClient $obj,
    string $job_handle,
    mixed $context = null,
): GearmanTask|false {
}

function gearman_client_set_workload_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_created_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_data_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_warning_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_status_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_complete_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_exception_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_set_fail_callback(GearmanClient $obj, callable $function): bool
{
}

function gearman_client_clear_callbacks(GearmanClient $obj): bool
{
}

function gearman_client_context(GearmanClient $obj): string
{
}

function gearman_client_set_context(GearmanClient $obj, string $data): bool
{
}

function gearman_client_enable_exception_handler(GearmanClient $obj): bool
{
}

class GearmanJob
{
    public function __destruct()
    {
    }

    public function returnCode(): int
    {
    }

    public function setReturn(int $gearman_return_t): bool
    {
    }

    public function sendData(string $data): bool
    {
    }

    public function sendWarning(string $warning): bool
    {
    }

    public function sendStatus(int $numerator, int $denominator): bool
    {
    }

    public function sendComplete(string $result): bool
    {
    }

    public function sendException(string $exception): bool
    {
    }

    public function sendFail(): bool
    {
    }

    public function handle(): false|string
    {
    }

    public function functionName(): false|string
    {
    }

    public function unique(): false|string
    {
    }

    public function workload(): string
    {
    }

    public function workloadSize(): int
    {
    }
}

function gearman_job_return_code(GearmanJob $obj): int
{
}

function gearman_job_set_return(GearmanJob $obj, int $gearman_return_t): bool
{
}

function gearman_job_send_data(GearmanJob $obj, string $data): bool
{
}

function gearman_job_send_warning(GearmanJob $obj, string $warning): bool
{
}

function gearman_job_send_status(GearmanJob $obj, int $numerator, int $denominator): bool
{
}

function gearman_job_send_complete(GearmanJob $obj, string $result): bool
{
}

function gearman_job_send_exception(GearmanJob $obj, string $exception): bool
{
}

function gearman_job_send_fail(GearmanJob $obj): bool
{
}

function gearman_job_handle(GearmanJob $obj): false|string
{
}

function gearman_job_function_name(GearmanJob $obj): false|string
{
}

function gearman_job_unique(GearmanJob $obj): false|string
{
}

function gearman_job_workload(GearmanJob $obj): string
{
}

function gearman_job_workload_size(GearmanJob $obj): int
{
}

class GearmanTask
{
    public function __construct() {}

    public function returnCode(): int
    {
    }

    public function functionName(): false|string
    {
    }

    public function unique(): false|string
    {
    }

    public function jobHandle(): false|string
    {
    }

    public function isKnown(): bool
    {
    }

    public function isRunning(): bool
    {
    }

    public function taskNumerator(): false|int
    {
    }

    public function taskDenominator(): false|int
    {
    }

    public function data(): false|string
    {
    }

    public function dataSize(): int|false
    {
    }

    public function sendWorkload(string $data): int|false
    {
    }

    public function recvData(int $data_len): false|array
    {
    }
}

function gearman_task_return_code(GearmanTask $obj): int
{
}

function gearman_task_function_name(GearmanTask $obj): false|string
{
}

function gearman_task_unique(GearmanTask $obj): false|string
{
}

function gearman_task_job_handle(GearmanTask $obj): false|string
{
}

function gearman_task_is_known(GearmanTask $obj): bool
{
}

function gearman_task_is_running(GearmanTask $obj): bool
{
}

function gearman_task_numerator(GearmanTask $obj): false|int
{
}

function gearman_task_denominator(GearmanTask $obj): false|int
{
}

function gearman_task_data(GearmanTask $obj): false|string
{
}

function gearman_task_data_size(GearmanTask $obj): int|false
{
}

function gearman_task_send_workload(GearmanTask $obj, string $data): int|false
{
}

function gearman_task_recv_data(GearmanTask $obj, int $data_len): false|array
{
}

class GearmanWorker
{
    public function __construct() {}

    public function __destruct()
    {
    }

    public function returnCode(): int
    {
    }

    public function error(): string|false
    {
    }

    public function getErrno(): int
    {
    }

    public function options(): int
    {
    }

    public function setOptions(int $option): true
    {
    }

    public function addOptions(int $option): true
    {
    }

    public function removeOptions(int $option): true
    {
    }

    public function timeout(): int
    {
    }

    public function setTimeout(int $timeout): true
    {
    }

    public function setId(string $id): bool
    {
    }

    public function addServer(null|string $host = null, int $port = 0, bool $setupExceptionHandler = true): bool
    {
    }

    public function addServers(null|string $servers = null, bool $setupExceptionHandler = true): bool
    {
    }

    public function wait(): bool
    {
    }

    public function register(string $function_name, int $timeout = 0): bool
    {
    }

    public function unregister(string $function_name): bool
    {
    }

    public function unregisterAll(): bool
    {
    }

    public function grabJob(): GearmanWorker|false
    {
    }

    public function addFunction(
        string $function_name,
        callable $function,
        mixed $context = null,
        int $timeout = 0,
    ): bool {
    }

    public function work(): bool
    {
    }

    public function ping(string $data): bool
    {
    }

    public function enableExceptionHandler(): bool
    {
    }
}

function gearman_worker_return_code(GearmanWorker $obj): int
{
}

function gearman_worker_error(GearmanWorker $obj): string|false
{
}

function gearman_worker_errno(GearmanWorker $obj): int
{
}

function gearman_worker_options(GearmanWorker $obj): int
{
}

function gearman_worker_set_options(GearmanWorker $obj, int $option): true
{
}

function gearman_worker_add_options(GearmanWorker $obj, int $option): true
{
}

function gearman_worker_remove_options(GearmanWorker $obj, int $option): true
{
}

function gearman_worker_timeout(GearmanWorker $obj): int
{
}

function gearman_worker_set_timeout(GearmanWorker $obj, int $timeout): true
{
}

function gearman_worker_set_id(GearmanWorker $obj, string $id): bool
{
}

function gearman_worker_add_server(
    GearmanWorker $obj,
    null|string $host = null,
    int $port = 0,
    bool $setupExceptionHandler = true,
): bool {
}

function gearman_worker_add_servers(
    GearmanWorker $obj,
    null|string $servers = null,
    bool $setupExceptionHandler = true,
): bool {
}

function gearman_worker_wait(GearmanWorker $obj): bool
{
}

function gearman_worker_register(GearmanWorker $obj, string $function_name, int $timeout = 0): bool
{
}

function gearman_worker_unregister(GearmanWorker $obj, string $function_name): bool
{
}

function gearman_worker_unregister_all(GearmanWorker $obj): bool
{
}

function gearman_worker_grab_job(GearmanWorker $obj): GearmanWorker|false
{
}

function gearman_worker_add_function(
    GearmanWorker $obj,
    string $function_name,
    callable $function,
    mixed $context = null,
    int $timeout = 0,
): bool {
}

function gearman_worker_work(GearmanWorker $obj): bool
{
}

function gearman_worker_ping(GearmanWorker $obj, string $data): bool
{
}

function gearman_worker_enable_exception_handler(GearmanWorker $obj): bool
{
}

class GearmanException extends Exception
{
}
