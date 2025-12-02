<?php

class ZendAPI_Queue
{
    public $_jobqueue_url;

    /**
     * @param string $password
     * @param int $application_id
     *
     * @return bool
     */
    public function login($password, $application_id = null)
    {
    }

    /**
     * @param ZendAPI_Job $job
     *
     * @return int
     */
    public function addJob($job)
    {
    }

    /**
     * @param int $job_id
     *
     * @return ZendAPI_Job
     */
    public function getJob($job_id)
    {
    }

    /**
     * @param ZendAPI_Job $job
     *
     * @return int
     */
    public function updateJob($job)
    {
    }

    /**
     * @param int|list<int> $job_id
     *
     * @return bool
     */
    public function removeJob($job_id)
    {
    }

    /**
     * @param int|list<int> $job_id
     *
     * @return bool
     */
    public function suspendJob($job_id)
    {
    }

    /**
     * @param int|list<int> $job_id
     *
     * @return bool
     */
    public function resumeJob($job_id)
    {
    }

    /**
     * @param Job $job
     *
     * @return bool
     */
    public function requeueJob($job)
    {
    }

    /**
     * @return array
     */
    public function getStatistics()
    {
    }

    /**
     * @param string $path
     *
     * @return bool
     */
    public function isScriptExists($path)
    {
    }

    /**
     * @return bool
     */
    public function isSuspend()
    {
    }

    /**
     * @param array $filter_options
     * @param int $max_jobs
     * @param bool $with_globals_and_output
     *
     * @return array
     */
    public function getJobsInQueue($filter_options = null, $max_jobs = -1, $with_globals_and_output = false)
    {
    }

    /**
     * @param array $filter_options
     *
     * @return int
     */
    public function getNumOfJobsInQueue($filter_options = null)
    {
    }

    /**
     * @return array
     */
    public function getAllhosts()
    {
    }

    /**
     * @return array
     */
    public function getAllApplicationIDs()
    {
    }

    /**
     * @param int $status
     * @param int $start_time
     * @param int $end_time
     * @param int $index
     * @param int $count
     * @param int $total
     *
     * @return array
     */
    public function getHistoricJobs($status, $start_time, $end_time, $index, $count, &$total)
    {
    }

    /**
     * @return bool
     */
    public function suspendQueue()
    {
    }

    /**
     * @return bool
     */
    public function resumeQueue()
    {
    }

    /**
     * @return string
     */
    public function getLastError()
    {
    }

    /**
     * @return bool
     */
    public function setMaxHistoryTime()
    {
    }
}

class ZendAPI_Job
{
    /**
     * @var int
     */
    public $_id;

    /**
     * @var string
     */
    public $_script;

    /**
     * @var string
     */
    public $_host;

    /**
     * @var string
     */
    public $_name;

    /**
     * @var string
     */
    public $_output;

    /**
     * @var int
     */
    public $_status = JOB_QUEUE_STATUS_WAITING;

    /**
     * @var string
     */
    public $_application_id = null;

    /**
     * @var int
     */
    public $_priority = JOB_QUEUE_PRIORITY_NORMAL;

    /**
     * @var array
     */
    public $_user_variables = [];

    /**
     * @var int
     */
    public $_global_variables = 0;

    /**
     * @var int|array
     */
    public $_predecessor = null;

    /**
     * @var int
     */
    public $_scheduled_time = 0;

    /**
     * @var int
     */
    public $_interval = 0;

    /**
     * @var int
     */
    public $_end_time = null;

    /**
     * @var int
     */
    public $_preserved = 0;

    /**
     * @param string $jobqueue_url
     * @param string $password
     *
     * @return int|false
     */
    public function addJobToQueue($jobqueue_url, $password)
    {
    }

    /**
     * @param int $priority
     */
    public function setJobPriority($priority)
    {
    }

    public function setJobName($name)
    {
    }

    public function setScript($script)
    {
    }

    public function setApplicationID($app_id)
    {
    }

    public function setUserVariables($vars)
    {
    }

    public function setGlobalVariables($vars)
    {
    }

    public function setJobDependency($job_id)
    {
    }

    public function setScheduledTime($timestamp)
    {
    }

    public function setRecurrenceData($interval, $end_time = null)
    {
    }

    public function setPreserved($preserved)
    {
    }

    /**
     * @return array
     */
    public function getProperties()
    {
    }

    public function getOutput()
    {
    }

    public function getID()
    {
    }

    public function getHost()
    {
    }

    public function getScript()
    {
    }

    public function getJobPriority()
    {
    }

    public function getJobName()
    {
    }

    public function getApplicationID()
    {
    }

    public function getUserVariables()
    {
    }

    public function getGlobalVariables()
    {
    }

    public function getJobDependency()
    {
    }

    public function getScheduledTime()
    {
    }

    public function getInterval()
    {
    }

    public function getEndTime()
    {
    }

    public function getPreserved()
    {
    }

    /**
     * Get the current status of the job
     * If this job was created and not returned from a queue (using the JobQueue::GetJob() function),
     *  the function will return false
     * The status is one of the constants with the "JOB_QUEUE_STATUS_" prefix.
     * E.g. job was performed and failed, job is waiting etc.
     *
     * @return int|false
     */
    public function getJobStatus()
    {
    }

    /**
     * Get how much seconds there are until the next time the job will run.
     * If the job is not recurrence or it past its end time, then return 0.
     *
     * @return int
     */
    public function getTimeToNextRepeat()
    {
    }

    /**
     * For recurring job get the status of the last execution. For simple job,
     * getLastPerformedStatus is equivalent to getJobStatus.
     * jobs that haven't been executed yet will return STATUS_WAITING
     * @return int
     */
    public function getLastPerformedStatus()
    {
    }
}

class java
{
}

class JavaException
{
    /**
     * @return object
     */
    public function getCause()
    {
    }
}

/**
 * @param bool $status
 *
 * @return void
 */
function accelerator_set_status($status)
{
}

/**
 * @return void
 */
function output_cache_disable()
{
}

/**
 * @return void
 */
function output_cache_disable_compression()
{
}

/**
 * @param string $key
 * @param string $function
 * @param int $lifetime
 *
 * @return string
 */
function output_cache_fetch($key, $function, $lifetime)
{
}

/**
 * @param string $key
 * @param string $function
 * @param int $lifetime
 *
 * @return mixed
 */
function output_cache_output($key, $function, $lifetime)
{
}

/**
 * @param string $filename
 *
 * @return bool
 */
function output_cache_remove($filename)
{
}

/**
 * @param string $url
 *
 * @return bool
 */
function output_cache_remove_url($url)
{
}

/**
 * @param string $key
 *
 * @return bool
 */
function output_cache_remove_key($key)
{
}

/**
 * @param string $key
 *
 * @return bool
 */
function output_cache_put($key, $data)
{
}

/**
 * @param string $key
 * @param int $lifetime
 */
function output_cache_get($key, $lifetime)
{
}

/**
 * @param string $key
 * @param int $lifetime
 *
 * @return bool
 */
function output_cache_exists($key, $lifetime)
{
}

/**
 * @return void
 */
function output_cache_stop()
{
}

/**
 * @param int $errno
 * @param string $errstr
 * @param string $errfile
 * @param int $errline
 *
 * @return void
 */
function monitor_pass_error($errno, $errstr, $errfile, $errline)
{
}

/**
 * @param string $hint
 *
 * @return void
 */
function monitor_set_aggregation_hint($hint)
{
}

/**
 * @param string $class
 * @param string $text
 * @param int $severe
 *
 * @return void
 */
function monitor_custom_event($class, $text, $severe = null, $user_data = null)
{
}

/**
 * @param int $error_code
 * @param string $url
 * @param int $severe
 *
 * @return void
 */
function monitor_httperror_event($error_code, $url, $severe = null)
{
}

/**
 * @return array
 */
function monitor_license_info()
{
}

/**
 * @param string $event_handler_func
 * @param string $handler_register_name
 * @param int $event_type_mask
 *
 * @return bool
 */
function register_event_handler($event_handler_func, $handler_register_name, $event_type_mask)
{
}

/**
 * @param string $handler_name
 *
 * @return bool
 */
function unregister_event_handler($handler_name)
{
}

/**
 * @param string $filename
 * @param string $mime_type
 * @param string $custom_headers
 *
 * @return null|false
 */
function zend_send_file($filename, $mime_type, $custom_headers)
{
}

/**
 * @param string $buffer
 * @param string $mime_type
 * @param string $custom_headers
 *
 * @return null|false
 */
function zend_send_buffer($buffer, $mime_type, $custom_headers)
{
}

/**
 * @param string $error_string
 */
function set_job_failed($error_string)
{
}

/**
 * @return array{license_ok: bool, expires: string|int}
 */
function jobqueue_license_info()
{
}

/**
 * @param string $class
 *
 * @return object
 */
function java($class)
{
}

/**
 * @return object|false
 */
function java_last_exception_get()
{
}

/**
 * @return void
 */
function java_last_exception_clear()
{
}

/**
 * @param bool $ignore
 *
 * @return void
 */
function java_set_ignore_case($ignore)
{
}

/**
 * @param string $encoding
 *
 * @return array
 */
function java_set_encoding($encoding)
{
}

/**
 * @param bool $throw
 *
 * @return void
 */
function java_throw_exceptions($throw)
{
}

/**
 * @param string $new_jarpath
 *
 * @return array
 */
function java_reload($new_jarpath)
{
}

/**
 * @param string $new_classpath
 *
 * @return array
 */
function java_require($new_classpath)
{
}

/**
 * @return bool
 */
function zend_loader_enabled()
{
}

/**
 * @return bool
 */
function zend_loader_file_encoded()
{
}

/**
 * @return array
 */
function zend_loader_file_licensed()
{
}

/**
 * @return string
 */
function zend_loader_current_file()
{
}

/**
 * @param string $license_file
 * @param bool $override
 *
 * @return bool
 */
function zend_loader_install_license($license_file, $override)
{
}

/**
 * @param string $function_name
 *
 * @return string
 */
function zend_obfuscate_function_name($function_name)
{
}

/**
 * @param string $class_name
 *
 * @return string
 */
function zend_obfuscate_class_name($class_name)
{
}

/**
 * @return int
 */
function zend_current_obfuscation_level()
{
}

/**
 * @return void
 */
function zend_runtime_obfuscate()
{
}

/**
 * @param bool $all_ids
 *
 * @return array
 */
function zend_get_id($all_ids = false)
{
}

/**
 * @return string
 */
function zend_optimizer_version()
{
}

const JOB_QUEUE_STATUS_SUCCESS = 1;

const JOB_QUEUE_STATUS_WAITING = 2;

const JOB_QUEUE_STATUS_SUSPENDED = 3;

const JOB_QUEUE_STATUS_SCHEDULED = 4;

const JOB_QUEUE_STATUS_WAITING_PREDECESSOR = 5;

const JOB_QUEUE_STATUS_IN_PROCESS = 6;

const JOB_QUEUE_STATUS_EXECUTION_FAILED = 7;

const JOB_QUEUE_STATUS_LOGICALLY_FAILED = 8;

const JOB_QUEUE_PRIORITY_LOW = 0;

const JOB_QUEUE_PRIORITY_NORMAL = 1;

const JOB_QUEUE_PRIORITY_HIGH = 2;

const JOB_QUEUE_PRIORITY_URGENT = 3;

const JOB_QUEUE_SAVE_POST = 1;

const JOB_QUEUE_SAVE_GET = 2;

const JOB_QUEUE_SAVE_COOKIE = 4;

const JOB_QUEUE_SAVE_SESSION = 8;

const JOB_QUEUE_SAVE_RAW_POST = 16;

const JOB_QUEUE_SAVE_SERVER = 32;

const JOB_QUEUE_SAVE_FILES = 64;

const JOB_QUEUE_SAVE_ENV = 128;
