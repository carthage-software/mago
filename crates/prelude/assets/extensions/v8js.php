<?php

final class V8JsTimeLimitException extends Exception {}

final class V8JsMemoryLimitException extends Exception {}

class V8Js
{
    public const V8_VERSION = '';
    public const FLAG_NONE = 1;
    public const FLAG_FORCE_ARRAY = 2;
    public const FLAG_PROPAGATE_PHP_EXCEPTIONS = 4;

    /**
     * @param string $object_name
     * @param bool $report_uncaught_exceptions
     * @param string $snapshot_blob
     */
    public function __construct(
        $object_name = 'PHP',
        array $variables = [],
        array $extensions = [],
        $report_uncaught_exceptions = true,
        $snapshot_blob = null,
    ) {}

    public function setModuleLoader(callable $loader) {}

    public function setModuleNormaliser(callable $normaliser) {}

    /**
     * @param string $script
     * @param string $identifier
     * @param int $flags
     * @param int $time_limit
     * @param int $memory_limit
     */
    public function executeString(
        $script,
        $identifier = '',
        $flags = V8Js::FLAG_NONE,
        $time_limit = 0,
        $memory_limit = 0,
    ) {}

    /**
     * @param string $script
     * @param string $identifier
     *
     * @return resource
     */
    public function compileString($script, $identifier = '') {}

    /**
     * @param resource $script
     * @param int $flags
     * @param int $time_limit
     * @param int $memory_limit
     */
    public function executeScript($script, $flags = V8Js::FLAG_NONE, $time_limit = 0, $memory_limit = 0) {}

    /**
     * @param int $limit
     */
    public function setTimeLimit($limit) {}

    /**
     * @param int $limit
     */
    public function setMemoryLimit($limit) {}

    /**
     * @param int $average_object_size
     */
    public function setAverageObjectSize($average_object_size) {}

    /**
     * @return V8JsScriptException|null
     */
    public function getPendingException() {}

    public function clearPendingException() {}

    /**
     * @param string $extension_name
     * @param string $code
     * @param bool $auto_enable
     * @return bool
     */
    public static function registerExtension($extension_name, $code, array $dependencies, $auto_enable = false) {}

    /**
     * @return list<string>
     */
    public static function getExtensions() {}

    /**
     * @param string $embed_source
     *
     * @return string|false
     */
    public static function createSnapshot($embed_source) {}
}

final class V8JsScriptException extends Exception
{
    /**
     * @return string
     */
    final public function getJsFileName() {}

    /**
     * @return int
     */
    final public function getJsLineNumber() {}

    /**
     * @return int
     */
    final public function getJsStartColumn() {}

    /**
     * @return int
     */
    final public function getJsEndColumn() {}

    /**
     * @return string
     */
    final public function getJsSourceLine() {}

    /**
     * @return string
     */
    final public function getJsTrace() {}
}
